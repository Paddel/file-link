use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::{console, File, HtmlInputElement};
use yew::platform::spawn_local;
use yew::{Html, html, Context, Component, NodeRef};

use drop_files::DropFiles;

use crate::file_tag::{FileState, FileTag, convert_bytes_to_readable_format};
use crate::pages::host::slider::Slider;
use crate::wrtc_protocol::{FilesUpdate, FileInfo, FileRequest};
use crate::services::web_rtc::{State, ConnectionState, WebRtcMessage, WebRTCManager};
use crate::services::api_service::{api_service, ApiServiceMessage};

mod drop_files;
mod slider;

const COMPRESSION_DEFAULT: u8 = 9;

#[derive(Clone)]
pub struct FileItem {
    state: FileState,
    pub tag: FileTag,
    js_file: File,
    progress: f64,
}

pub enum Msg {
    SessionStart,
    CopyShareLink,
    Update(Vec<File>),
    CompressionUpdate(u8),
    TransferUpdate((FileTag, f64)),
    FileRemove(FileTag),

    CallbackWebRtc(WebRtcMessage),
    CallbackApi(ApiServiceMessage),
}

pub struct Host {
    web_rtc_manager: Rc<RefCell<WebRTCManager>>,
    web_rtc_state: ConnectionState,
    files: HashMap<Uuid, FileItem>,
    origin: String,
    code: String,
    compression_level: u8,
    password: String,
    node_password: NodeRef,
    node_share: NodeRef,
}

impl Component for Host {
    type Message = Msg;
    type Properties = ();
     
    
    fn create(ctx: &Context<Self>) -> Self {
        let origin = web_sys::window()
            .expect("no global `window` exists")
            .location()
            .origin()  // This gets the origin
            .unwrap_or_else(|_| "Error getting origin".to_string());


        Host {
            web_rtc_manager: WebRTCManager::new(ctx.link().callback(Msg::CallbackWebRtc)),
            web_rtc_state: ConnectionState::new(),
            files: HashMap::new(),
            origin,
            code: String::new(),
            compression_level: COMPRESSION_DEFAULT,
            password: String::new(),
            node_password: NodeRef::default(),
            node_share: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Update(files) => {
                self.handle_files(files);
                true
            }
            Msg::CompressionUpdate(value) => {
                self.compression_level = value;
                true
            }
            Msg::SessionStart => {
                self.password = if let Some(input) = self.node_password.cast::<HtmlInputElement>() {
                    input.value()
                } else {
                    "".to_string()
                };
                
                if let Some(state) = self.web_rtc_state.ice_gathering_state {
                    if state == web_sys::RtcIceGatheringState::Complete {
                        self.create_session(ctx);
                        return true;
                    }
                }
                
                self.web_rtc_manager.deref().borrow_mut().set_state(State::Server(ConnectionState::new()));
                let result: Result<(), wasm_bindgen::JsValue> = WebRTCManager::start_web_rtc(&self.web_rtc_manager);
                if result.is_err() {
                    console::log_1(&result.err().unwrap());
                }
                true
            }
            Msg::TransferUpdate((file_tag, progress)) => {
                let file = self.files.get_mut(&file_tag.uuid);
                if let Some(file) = file {
                    file.progress = progress;
                    if progress >= 1.0 {
                        file.state = FileState::Done;
                    }
                }
                true
            }
            Msg::FileRemove(tag) => {
                self.files.remove(&tag.uuid);
                self.web_rtc_send_update();
                true
            }
            Msg::CallbackWebRtc(msg) => {
                self.update_web_rtc(ctx, msg)
            }
            Msg::CallbackApi(msg) => {
                self.update_api_service(ctx, msg)
            }
            Msg::CopyShareLink => {
                if let Some(input) = self.node_share.cast::<web_sys::HtmlInputElement>() {
                    input.select();
                    // let _ = web_sys::window().unwrap().document().unwrap().exec_command("copy");
                    let document = web_sys::window().unwrap().document().unwrap();
                    let document = document.dyn_into::<web_sys::HtmlDocument>().unwrap();
                    let _ = document.exec_command("copy");
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if self.web_rtc_connected() {
            self.view_session_handle(ctx)
        }
        else {
            self.view_session_create(ctx)
        }
    }
}

impl Host {
    fn create_session(&self, ctx: &Context<Self>) {
        let callback = ctx.link().callback(Msg::CallbackApi);
        let answer = self.web_rtc_manager.deref().borrow().create_encoded_offer();
        api_service::create_session(callback, answer, self.password.clone(), self.compression_level);
    }

    fn handle_files(&mut self, files: Vec<File>) {
        files.into_iter().for_each(|file| {
            let item = FileItem {
                state: FileState::Pending,
                tag: FileTag::from(file.clone()),
                js_file: file.clone(),
                progress: 0.0,
            };
            self.files.insert(item.tag.uuid, item);
        });

        self.web_rtc_send_update();
    }

    fn web_rtc_send_update(&self) {
        let files: Vec<FileInfo> = self.files.iter().map(|(_, file)| {
            FileInfo {
                uuid: file.tag.uuid(),
                name: file.tag.name().to_string(),
                size: file.tag.size(),
            }
        }).collect();

        let update = FilesUpdate{files};
        let message = serde_json::to_string(&update).unwrap();
        self.web_rtc_manager
        .deref()
        .borrow()
        .send_message(&message);
    }

    fn web_rtc_connected(&self) -> bool {
        matches!(self.web_rtc_state.ice_connection_state, Some(web_sys::RtcIceConnectionState::Connected))
    }
    
    fn web_rtc_send_file(&mut self, ctx: &Context<Self>, uuid: Uuid) {
        // let file = self.files.iter().find(|(_, file)| file.tag.uuid() == uuid);	
        let file = self.files.get_mut(&uuid);
        let file = match file {
            Some(file) => file,
            None => return,
        };

        file.state = FileState::Transferring;
        
        let callback_update = ctx.link().callback(|(tag, progress)| Msg::TransferUpdate((tag, progress)));
        let mut file = file.clone();
        let web_rtc_manager = self.web_rtc_manager.clone();
        spawn_local(async move {
            let blob = file.js_file.deref();

            const CHUNK_SIZE: f64 = 64.0 * 1024.0;
            let mut offset = 0.0;

            while offset < blob.size() {
                let end = (offset + CHUNK_SIZE).min(blob.size());

                let chunk = blob.slice_with_i32_and_f64(offset as i32, end.into());
                if chunk.is_err() {
                    console::log_1(&format!("Failed to slice chunk").into());
                    return;
                }
                
                let chunk = chunk.unwrap();
                let sent_success = web_rtc_manager
                    .deref()
                    .borrow()
                    .send_data(&chunk, 10)
                    .await;

                if !sent_success {
                    console::log_1(&format!("Failed to send chunk").into());
                    return;
                }
                file.progress = end / blob.size();
                offset += CHUNK_SIZE;
                callback_update.emit((file.tag.clone(), file.progress));
            }
        });
    }

    fn update_web_rtc(&mut self, ctx: &Context<Self>, msg: WebRtcMessage) -> bool {
        match msg {
            WebRtcMessage::Message(data) => {
                let update: Result<FileRequest, serde_json::Error> = serde_json::from_str(&data);
                if update.is_err() {
                    return false;
                }

                self.web_rtc_send_file(ctx, update.unwrap().uuid);
                true
            }
            WebRtcMessage::Data(_, _) => {
                //Host should not receive data
                false
            }
            WebRtcMessage::UpdateState(state) => {
                let mut update = false;
                if let State::Server(connection_state) = state.clone() {
                    if connection_state.ice_gathering_state != self.web_rtc_state.ice_gathering_state {
                        if let Some(state) = connection_state.ice_gathering_state {
                            if state == web_sys::RtcIceGatheringState::Complete {
                                self.create_session(ctx);
                            }
                        }
                        update = true
                    }
                    
                    if connection_state.ice_connection_state != self.web_rtc_state.ice_connection_state {
                        if let Some(state) = connection_state.ice_connection_state {
                            if state == web_sys::RtcIceConnectionState::Connected ||
                            state == web_sys::RtcIceConnectionState::Disconnected {

                                update = true;
                            }
                        }
                    }
                    
                    self.web_rtc_state = connection_state;
                };
                update
            }
            WebRtcMessage::Reset => {
                self.web_rtc_manager = WebRTCManager::new(ctx.link().callback(Msg::CallbackWebRtc));
                self.web_rtc_state = ConnectionState::new();
                self.files = HashMap::new();
                self.code = String::new();
                self.compression_level = COMPRESSION_DEFAULT;
                
                false
            }
        }
    }

    fn update_api_service(&mut self, _ctx: &Context<Self>, msg: ApiServiceMessage) -> bool {
        match msg {
            ApiServiceMessage::HostCreate(result) => {
                if result.is_err() {
                    return false;
                }
                let result = result.unwrap();
                self.code = result.code;

                api_service::poll_session(_ctx.link().callback(Msg::CallbackApi), self.code.clone());
                true
            },
            ApiServiceMessage::HostPoll(result) => {
                if result.is_err() {
                    return false;
                }
                let result = result.unwrap();
                
                let _ = WebRTCManager::validate_answer(&self.web_rtc_manager, &result.connection_details);
                true
            },
            _ => false,
        }
    }

    fn view_session_create (&self, ctx: &Context<Self>) -> Html {
        if !self.code.is_empty() {
            let url = format!("{}/receive/{}", self.origin, self.code);

            html! {
                <div class="container mt-5">
                    <div class="row justify-content-center">
                        <div class="col-md-6">
                            <h2 class="text-center mb-4">{"Share the link"}</h2>
                            <div class="input-group">
                                <input type="text" ref={self.node_share.clone()} class="form-control" value={url} readonly={true} />
                                <div class="input-group-append">
                                    <button onclick={ctx.link().callback(|_| Msg::CopyShareLink)} class="btn btn-outline-secondary" type="button">{"Copy"}</button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            }
        }
        else {
            let creation_disabled = self.web_rtc_state.ice_gathering_state == Some(web_sys::RtcIceGatheringState::Gathering);
            html! {
                <div class="container mt-5">
                    <div class="row justify-content-center">
                        <div class="col-md-6">
                            <h2 class="text-center mb-4">{"Create a Session"}</h2>
                            <div class="mb-3">
                                <label for="password" class="form-label">{"Enter Password (Optional):"}</label>
                                <input type="password" class="form-control" ref={self.node_password.clone()} placeholder="Enter Password" />
                            </div>
                            <div class="mb-3">
                                <Slider label="Compression Level"
                                    min={0} max={10}
                                    onchange={ctx.link().callback(|value| {Msg::CompressionUpdate(value as u8)})}
                                    value={self.compression_level as i32}
                                />
                            </div>
                            <div class="mb-3">
                                <button onclick={ctx.link().callback(|_| Msg::SessionStart)} class="btn btn-primary btn-block" disabled={creation_disabled}>{"Create Session"}</button>
                            </div>
                        </div>
                    </div>
                </div>
            }
        }
    }

    fn view_session_handle(&self, ctx: &Context<Self>) -> Html {
        let section_table = {
            html! {
                <div class="table-wrapper table-responsive">
                    <table class="table custom-table table-bordered">
                        <thead>
                            <tr>
                                <th>{"#"}</th>
                                <th>{"Name"}</th>
                                <th>{"Size"}</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody>
                            {
                                for self.files.iter().enumerate().map(|(index, (_, file))| {
                                    let tag = file.tag.clone();
                                    html! {
                                        <tr>
                                            <td>{index}</td>
                                            <td class="table-name">{&tag.name()}</td>
                                            <td>{convert_bytes_to_readable_format(tag.size() as u64)}</td>
                                            <td>{Self::view_control_pannel(ctx, file)}</td>
                                        </tr>
                                    }
                                })
                            }
                        </tbody>
                    </table>
                </div>
            }
        };

        html! {
            <div class="container mt-5 d-flex flex-column justify-content-center align-items-center">
                <div class="col-md-9 info-panel bg-light p-3 rounded text-center mb-3">
                    <div class="d-flex justify-content-around align-items-center w-100">
                        <p class="d-flex align-items-center mb-0">
                            <span class="pl-3 pr-1 font-weight-bold">{"Connection:"}</span>
                            <span class="text-success">{"🟢"}</span>//todo: Add timeout indicator
                        </p>
                        <p class="d-flex align-items-center mb-0">
                            <span class="pl-3 pr-1 font-weight-bold">{"Password:"}</span> 
                            <span>{format!("{}", if self.password.is_empty() {"🔓"} else {"🔒"})}</span>
                        </p>
                        <p class="d-flex align-items-center mb-0">
                            <span class="pl-3 pr-1 font-weight-bold">{"Compression:"}</span> 
                            <span>{self.compression_level}</span>
                        </p>
                    </div>
                    <div class="mt-2">
                        <DropFiles onupdate={ctx.link().callback(Msg::Update)} />
                    </div>
                </div>
                {if self.files.len() > 0 {section_table} else {html!{}}}
            </div>
        }        
    }

    fn view_control_pannel(ctx: &Context<Self>, file: &FileItem) -> Html {
        match file.state {
            FileState::Pending => {
                let tag = file.tag.clone();
                html! {
                    <button class="btn btn-outline-secondary" onclick={ctx.link().callback(move |_| Msg::FileRemove(tag.clone()))}>{ "Remove" }</button>
                }
            }
            FileState::Transferring => {
                html! {
                    <div class="progress" style="height: 25px;">
                        <div class="progress-bar" role="progressbar" style={format!("width: {}%", (file.progress*100.0) as u32)} aria-valuenow={format!("{}%", (file.progress*100.0) as u32)} aria-valuemin="0" aria-valuemax="100">
                            <span style="color: white; text-shadow: 1px 1px 3px rgba(0, 0, 0, 0.6);">{format!("Progress: {}%", (file.progress*100.0) as u32)}</span>
                        </div>
                    </div>
                }
            }
            FileState::Done => {
                html! {
                    <p>{ "Done" }</p>
                }
            }
            _ => {
                html! {
                }
            }
        }
    }
}