use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use uuid::Uuid;
use web_sys::console;
use yew::prelude::*;

use download_manager::DownloadManager;

use crate::file_tag::{convert_bytes_to_readable_format, FileState, FileTag};
use crate::pages::client::connect::Connect;
use crate::pages::client::password::Password;
use crate::services::api_service::{api_service, ApiServiceMessage};
use crate::services::web_rtc::{ConnectionState, State, WebRTCManager, WebRtcMessage};
use crate::shared::ClientJoinResult;
use crate::wrtc_protocol::{FileRequest, FilesUpdate};

mod connect;
mod download_manager;
mod password;

pub struct FileItem {
    state: FileState,
    tag: FileTag,
    progress: f64,
}

pub enum ClientState {
    Connect,
    Password,
    Connected,
}

pub enum Msg {
    SessionConnect(String, Option<String>),
    FileAccept(FileTag),
    FileDownload(FileTag),

    CallbackWebRtc(WebRtcMessage),
    CallbackApi(ApiServiceMessage),
}

#[derive(Properties, PartialEq)]
pub struct ReceiveProps {
    #[prop_or(String::new())]
    pub code: String,
}

pub struct Client {
    download_manager: DownloadManager,
    web_rtc_manager: Rc<RefCell<WebRTCManager>>,
    web_rtc_state: ConnectionState,
    files: HashMap<Uuid, FileItem>,
    session_details: Option<ClientJoinResult>,
    session_code: Option<String>,
    password: Option<String>,
    password_needed: bool,
    fetchin_file: Option<FileTag>,
}

impl Component for Client {
    type Message = Msg;
    type Properties = ReceiveProps;

    fn create(ctx: &Context<Self>) -> Self {
        //Direct connect if code is provided

        let code = if !ctx.props().code.is_empty() {
            //TODO: join
            Some(ctx.props().code.clone())
        } else {
            None
        };

        Self {
            download_manager: DownloadManager::new(),
            web_rtc_manager: WebRTCManager::new(ctx.link().callback(Msg::CallbackWebRtc)),
            web_rtc_state: ConnectionState::new(),
            files: HashMap::new(),
            session_details: None,
            session_code: code,
            password: None,
            password_needed: false,
            fetchin_file: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SessionConnect(code, password) => {
                let callback: Callback<ApiServiceMessage> = ctx.link().callback(Msg::CallbackApi);
                api_service::get_session_details(callback, &code, password.clone());
                self.session_code = Some(code);
                self.password = password;
                true
            }
            Msg::FileAccept(tag) => self.handle_file_accept(tag),
            Msg::FileDownload(tag) => {
                self.download_manager.download(tag);
                true
            }
            Msg::CallbackWebRtc(msg) => self.update_web_rtc(ctx, msg),
            Msg::CallbackApi(msg) => self.update_api(ctx, msg),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match self.current_state() {
            ClientState::Connect => {
                html! { <Connect on_connect={ctx.link().callback(|code: String| Msg::SessionConnect(code.clone(), None))} /> }
            }
            ClientState::Password => {
                let session_code = self.session_code.clone().expect("Session code is not set");
                html! { <Password on_connect={ctx.link().callback(move |password: String| Msg::SessionConnect(session_code.clone(), Some(password.clone())))} /> }
            }
            ClientState::Connected => self.view_connected(ctx),
        }
    }
}

impl Client {
    fn handle_file_accept(&mut self, file_tag: FileTag) -> bool {
        let file_item: Option<&mut FileItem> = self.files.get_mut(&file_tag.uuid());
        let file_item = match file_item {
            Some(item) => item,
            None => return false,
        };

        if self.download_manager.active() {
            file_item.state = FileState::Queued;
            return true;
        }

        self.download_manager.new_file(file_tag.clone());

        file_item.state = FileState::Transferring;
        self.fetchin_file = Some(file_tag.clone());

        let file_request = FileRequest {
            uuid: file_tag.uuid(),
        };
        self.web_rtc_manager
            .deref()
            .borrow()
            .send_message(&serde_json::to_string(&file_request).unwrap());
        true
    }

    fn update_web_rtc(&mut self, ctx: &Context<Self>, msg: WebRtcMessage) -> bool {
        match msg {
            WebRtcMessage::Message(data) => {
                let update: Result<FilesUpdate, serde_json::Error> = serde_json::from_str(&data);
                if update.is_err() {
                    return false;
                }
                return self.on_files_updates(update.unwrap());
            }
            WebRtcMessage::Data(data, size) => {
                let result = self.download_manager.save_chunk(&data, size);

                if result.is_err() {
                    console::log_1(
                        &format!("Error saving chunk: {:?}", result.clone().err().unwrap()).into(),
                    );
                    return false;
                }

                let file_tag = self.download_manager.get_file_tag();
                let file = self.files.get_mut(&file_tag.unwrap().uuid());
                let file = match file {
                    Some(file) => file,
                    None => return false,
                };

                if result.unwrap() {
                    file.state = FileState::Done;
                    file.progress = 100.0;
                    for (_, file) in self.files.iter() {
                        if file.state == FileState::Queued {
                            self.handle_file_accept(file.tag.clone());
                            break;
                        }
                    }
                } else {
                    file.state = FileState::Transferring;
                    file.progress = self.download_manager.get_progress();
                    return true;
                }
                return true;
            }
            WebRtcMessage::UpdateState(state) => {
                if let State::Client(connection_state) = state.clone() {
                    self.on_state_update(ctx, &connection_state);
                    self.web_rtc_state = connection_state;
                };
            }
            WebRtcMessage::Reset => {
                self.web_rtc_manager = WebRTCManager::new(ctx.link().callback(Msg::CallbackWebRtc));
                self.download_manager = DownloadManager::new();
                self.files.clear();
                self.fetchin_file = None;
                self.web_rtc_state = ConnectionState::new();
                self.session_details = None;
                self.session_code = None;
                self.password_needed = false;
            }
        }
        true
    }

    fn update_api(&mut self, _: &Context<Self>, msg: ApiServiceMessage) -> bool {
        match msg {
            ApiServiceMessage::ClientDetails(result) => {
                if result.is_err() {
                    let status = result.unwrap_err();
                    if status == 401 {
                        //Unauthorized
                        self.password_needed = true;
                        return true;
                    } else {
                        console::log_1(
                            &format!("Error getting detail session: {:?}", status).into(),
                        );
                        return false;
                    }
                }
                let result = result.unwrap();
                let details = result.connection_details;

                self.web_rtc_manager
                    .deref()
                    .borrow_mut()
                    .set_state(State::Client(ConnectionState::new()));
                let result: Result<(), wasm_bindgen::JsValue> =
                    WebRTCManager::start_web_rtc(&self.web_rtc_manager);
                if result.is_ok() {
                    let result = WebRTCManager::validate_offer(&self.web_rtc_manager, &details);
                    if result.is_err() {
                        console::log_1(
                            &format!("Error validating offer: {:?}", result.clone().err()).into(),
                        );
                    }
                }
                false
            }
            ApiServiceMessage::ClientJoin(result) => {
                if result.is_err() {
                    let status = result.unwrap_err();
                    console::log_1(&format!("Error joining session: {:?}", status).into());
                    return false;
                }
                let result = result.unwrap();
                self.session_details = Some(result);
                true
            }
            _ => false,
        }
    }

    fn view_connected(&self, ctx: &Context<Self>) -> Html {
        let session_details = self.session_details.as_ref().unwrap();
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
                                    html! {{Self::view_file_row(ctx, index, file)}}
                                })
                            }
                        </tbody>
                    </table>
                </div>
            }
        };

        html! {
            <div class="container mt-5">
                <div class="row mb-3">
                    <div class="info-panel bg-light p-3 rounded text-center d-flex justify-content-around align-items-center w-100">
                        <p class="d-flex align-items-center mb-0">
                            <span class="pl-3 pr-1 font-weight-bold">{"Connection:"}</span>
                            <span class="text-success">{"🟢"}</span>//todo: Add timeout indicator
                        </p>
                        <p class="d-flex align-items-center mb-0">
                            <span class="pl-3 pr-1 font-weight-bold">{"Password:"}</span>
                            <span>{format!("{}", if session_details.has_password {"🔓"} else {"🔒"})}</span>
                        </p>
                        <p class="d-flex align-items-center mb-0">
                            <span class="pl-3 pr-1 font-weight-bold">{"Compression:"}</span>
                            <span>{session_details.compression_level}</span>
                        </p>
                    </div>
                </div>
                {if self.files.len() > 0 {section_table} else {html!{}}}
            </div>
        }
    }

    fn view_file_row(ctx: &Context<Self>, index: usize, file: &FileItem) -> Html {
        let file_tag = file.tag.clone();
        let control_pannel = {
            match file.state {
                FileState::Pending => {
                    let tag = file.tag.clone();
                    html! {
                        <button class="btn btn-outline-primary" onclick={ctx.link().callback(move |_| Msg::FileAccept(tag.clone()))}>{ "Accept" }</button>
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
                        <button class="btn btn-outline-primary" onclick={ctx.link().callback(move |_| Msg::FileDownload(file_tag.clone()))}>{ "Download" }</button>
                    }
                }
                FileState::Queued => {
                    html! {
                        <p>{ "Queued" }</p>
                    }
                }
            }
        };

        html! {
            <tr>
                <td>{index}</td>
                <td class="table-name">{&file.tag.name()}</td>
                <td>{convert_bytes_to_readable_format(file.tag.size() as u64)}</td>
                <td>{control_pannel}</td>
            </tr>
        }
    }

    fn on_state_update(&mut self, ctx: &Context<Self>, connection_state: &ConnectionState) {
        // console::log_1(&format!("UpdateState {:?}", connection_state).into());
        if connection_state.ice_gathering_state != self.web_rtc_state.ice_gathering_state {
            if let Some(state) = connection_state.ice_gathering_state {
                if state == web_sys::RtcIceGatheringState::Complete {
                    if self.session_code.is_none() {
                        console::log_1(&"Session code is not set".into());
                        return;
                    }
                    let session_code = self.session_code.clone().unwrap();
                    let answer = self.web_rtc_manager.deref().borrow().create_encoded_offer();

                    let callback: Callback<ApiServiceMessage> =
                        ctx.link().callback(Msg::CallbackApi);
                    api_service::join_session(
                        callback,
                        session_code,
                        self.password.clone(),
                        answer,
                    );
                }
            }
        }
    }

    fn on_files_updates(&mut self, files_update: FilesUpdate) -> bool {
        for file in files_update.files {
            let file_tag = FileTag::new(file.name, file.size, file.uuid);
            if !self.files.contains_key(&file_tag.uuid()) {
                self.files.insert(
                    file_tag.uuid(),
                    FileItem {
                        state: FileState::Pending,
                        tag: file_tag,
                        progress: 0.0,
                    },
                );
            }
        }
        true
    }

    fn current_state(&self) -> ClientState {
        if self.web_rtc_connected() && self.session_details.is_some() {
            ClientState::Connected
        } else if !self.password_needed || self.session_code.is_none() {
            ClientState::Connect
        } else {
            ClientState::Password
        }
    }

    fn web_rtc_connected(&self) -> bool {
        matches!(
            self.web_rtc_state.ice_connection_state,
            Some(web_sys::RtcIceConnectionState::Connected)
        )
    }
}
