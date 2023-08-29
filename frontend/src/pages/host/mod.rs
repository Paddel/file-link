use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use web_sys::{console, File, HtmlInputElement};
use yew::platform::spawn_local;
use yew::{Html, html, Context, Component, NodeRef};

use drop_files::DropFiles;

use crate::file_tag::{FileState, FileTag};
use crate::pages::host::slider::Slider;
use crate::wrtc_protocol::{FilesUpdate, FileInfo, FileRequest};
use crate::services::web_rtc::{State, ConnectionState, WebRtcMessage, WebRTCManager};
use crate::services::web_socket::{WsConnection, WebSocketMessage};

mod drop_files;
mod slider;

include!(concat!(env!("OUT_DIR"), "/config.rs"));
include!("../../../../shared/ws_protocol.rs");

const COMPRESSION_DEFAULT: i32 = 9;

#[derive(Clone)]
pub struct FileItem {
    state: FileState,
    pub tag: FileTag,
    js_file: File,
    progress: f64,
}

pub enum Msg {
    Update(Vec<File>),
    CompressionUpdate(i32),
    SessionStart,
    TransferUpdate((FileTag, f64)),
    FileRemove(FileTag),

    CallbackWebRtc(WebRtcMessage),
    CallbackWebsocket(WebSocketMessage),
}

pub struct Host {
    web_rtc_manager: Rc<RefCell<WebRTCManager>>,
    web_rtc_state: ConnectionState,
    web_socket: Option<WsConnection>,
    files: HashMap<Uuid, FileItem>,
    code: String,
    compression_level: i32,
    node_compression: NodeRef,
    node_password: NodeRef,
}

impl Component for Host {
    type Message = Msg;
    type Properties = ();
     
    
    fn create(ctx: &Context<Self>) -> Self {
        Host {
            web_rtc_manager: WebRTCManager::new(ctx.link().callback(Msg::CallbackWebRtc)),
            web_rtc_state: ConnectionState::new(),
            web_socket: None,
            files: HashMap::new(),
            code: String::new(),
            compression_level: COMPRESSION_DEFAULT,
            node_compression: NodeRef::default(),
            node_password: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Update(files) => {
                self.handle_files(files);
                true
            }
            Msg::CompressionUpdate(value) => {
                // let event: Event = event.dyn_into().unwrap();
                // let event_target = event.target().unwrap();
                // let target: HtmlInputElement = event_target.dyn_into().unwrap();

                // self.compression_level = target.value().parse::<i32>().unwrap();
                self.compression_level = value;
                true
            }
            Msg::SessionStart => {
                self.web_rtc_manager.deref().borrow_mut().set_state(State::Server(ConnectionState::new()));
                let _: Result<(), wasm_bindgen::JsValue> = WebRTCManager::start_web_rtc(&self.web_rtc_manager);
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
            Msg::CallbackWebsocket(msg) => {
                self.update_web_socket(ctx, msg)
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if self.web_rtc_connected() {
            self.view_session_create(ctx)
        }
        else {
            self.view_session_handle(ctx)
        }
    }
}

impl Host {
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
        !matches!(self.web_rtc_state.ice_connection_state, Some(web_sys::RtcIceConnectionState::Connected))
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

    fn ws_connect(&mut self, ctx: &Context<Self>) {
        let callback = ctx.link().callback(Msg::CallbackWebsocket);
        self.web_socket = WsConnection::new(WEBSOCKET_ADDRESS, callback).ok();
    }

    fn ws_disconnect(&mut self) {
        self.web_socket = None;
    }

    fn ws_send(&mut self, data: SessionDetails) {
        self.web_socket
            .as_mut()
            .unwrap()
            .send_text(&serde_json::to_string(&data).unwrap());
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
                    // console::log_1(&format!("UpdateState {:?}", connection_state).into());
                    if connection_state.ice_gathering_state != self.web_rtc_state.ice_gathering_state {
                        if let Some(state) = connection_state.ice_gathering_state {
                            if state == web_sys::RtcIceGatheringState::Complete {
                                self.ws_connect(ctx);
                            }
                        }
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
                false
            }
        }
    }

    fn update_web_socket(&mut self, _: &Context<Self>, msg: WebSocketMessage) -> bool {
        match msg {
            WebSocketMessage::Text(data) => {
                let mut update = false;
                let session_host_result: Result<SessionHostResult, serde_json::Error> = serde_json::from_str(&data);

                if let Ok(session_host_result) = session_host_result {
                    match session_host_result {
                        SessionHostResult::SessionCode(session_code) => {
                            self.code = session_code.code;
                            update = true;
                        }
                        SessionHostResult::SessionAnswerForward(session_answer_forward) => {
                            let answer = session_answer_forward.answer;
                            let result = WebRTCManager::validate_answer(&self.web_rtc_manager, &answer);
                            if result.is_ok() {
                                self.ws_disconnect();
                            }
                        }
                    }
                }
                update
            }
            WebSocketMessage::Open => {
                let password = if let Some(input) = self.node_password.cast::<HtmlInputElement>() {
                    input.value()
                } else {
                    "".to_string()
                };

                let compression = if let Some(input) = self.node_compression.cast::<HtmlInputElement>() {
                    input.value().parse::<i32>().unwrap()
                } else {
                    COMPRESSION_DEFAULT
                };

                let offer = self.web_rtc_manager.deref().borrow_mut().create_encoded_offer();
                let data = SessionDetails::SessionHost(SessionHost{
                    offer,
                    password,
                    compression: compression as u8,
                });
                self.ws_send(data);
                false
            }
            WebSocketMessage::Close => {
                self.ws_disconnect();
                false
            }
            WebSocketMessage::Err => {
                self.ws_disconnect();
                false
            }
        }
    }

    fn view_session_create (&self, ctx: &Context<Self>) -> Html {
        let section_websocket = if self.web_socket.is_some()
        {
            html! {
                <>
                    <p>{&self.code}</p>
                </>
            }
        }
        else {
            html! {
                <div class="col-md-12">
                    <label for="password">{"Enter Password (Optional):"}</label>
                    <input type="password" ref={self.node_password.clone()} placeholder="Enter Password" />

                    <Slider label="Compression Level"
                        min={0} max={10}
                        onchange={ctx.link().callback(|value| {Msg::CompressionUpdate(value)})}
                        value={self.compression_level}
                    />
                    // <label for="compression-slider">{"Choose Compression:"}</label>
                    // <input type="range" oninput={ctx.link().callback(|event| Msg::CompressionUpdate(event))} min="0" max="10" step="1" value="{self.compression_level}"/>
                    // <span id="slider-value">{self.compression_level}</span>

                    <button onclick={ctx.link().callback(|_| Msg::SessionStart)}>{"connect"}</button>
                </div>
            }
        };

        html! {
            <>
                <div class="row">
                    {section_websocket}
                </div>
            </>
        }
    }

    fn view_session_handle(&self, ctx: &Context<Self>) -> Html {
        html! {
                <table class="table">
                    <tbody>
                        <DropFiles onupdate={ctx.link().callback(Msg::Update)} />
                        {
                            for self.files.iter().enumerate().map(|(index, (_, file))| {
                                let tag = file.tag.clone();
                                html! {
                                    <tr>
                                        <td>{index}</td>
                                        <td>{&tag.name()}</td>
                                        <td>{tag.size()}</td>
                                        <td>{format!("{:?}", file.state)}</td>
                                        <td>{Self::view_control_pannel(ctx, file)}</td>
                                        <td>{"Waiting for Receiver"}</td>
                                    </tr>
                                }
                            })
                         }
                </tbody>
            </table>
        }
    }

    fn view_control_pannel(ctx: &Context<Self>, file: &FileItem) -> Html {
        match file.state {
            FileState::Pending => {
                let tag = file.tag.clone();
                html! {
                    <button onclick={ctx.link().callback(move |_| Msg::FileRemove(tag.clone()))}>{ "Remove" }</button>
                }
            }
            FileState::Transferring => {
                html! {
                    <p>{format!("Progress: {}%", (file.progress*100.0) as u32)}</p>
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