use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{HtmlInputElement, console};
use yew::prelude::*;

use download_manager::DownloadManager;

use crate::file_tag::{FileTag, FileState};
use crate::services::web_rtc::{WebRTCManager, WebRtcMessage, State, ConnectionState};
use crate::services::web_socket::{WsConnection, WebSocketMessage};
use crate::wrtc_protocol::{FilesUpdate, FileInfo, FileRequest};

mod download_manager;

include!(concat!(env!("OUT_DIR"), "/config.rs"));
include!("../../../../shared/ws_protocol.rs");

pub struct FileItem {
    state: FileState,
    tag: FileTag,
    queued: bool,
    progress: f64,
}

pub enum Msg {
    SessionConnect(String),
    FileAccept(FileTag),

    CallbackWebRtc(WebRtcMessage),
    CallbackWebsocket(WebSocketMessage),
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
    web_socket: Option<WsConnection>,
    session_details: SessionFetchOffer,
    password_needed: bool,
    session_host: Option<SessionHost>,
    files: HashMap<Uuid, FileItem>,
    input_code: NodeRef,
    fetchin_file: Option<FileTag>,
}

impl Component for Client {
    type Message = Msg;
    type Properties = ReceiveProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            download_manager: DownloadManager::new(),
            web_rtc_manager: WebRTCManager::new(ctx.link().callback(Msg::CallbackWebRtc)),
            web_rtc_state: ConnectionState::new(),
            web_socket: None,
            session_details: SessionFetchOffer {code: String::new(), password: String::new()},
            password_needed: false,
            session_host: None,
            files: HashMap::new(),
            input_code: NodeRef::default(),
            fetchin_file: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SessionConnect(password) => {
                self.session_details.code = self.input_code.cast::<HtmlInputElement>().unwrap().value();
                self.session_details.password = password;
                self.ws_connect(ctx);
            }
            Msg::FileAccept(tag) => {
                return self.web_rtc_request_file(tag);
            }
            Msg::CallbackWebRtc(msg) => {
                self.update_web_rtc(ctx, msg);
            }
            Msg::CallbackWebsocket(msg) => {
                self.update_web_socket(ctx, msg);
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let section_websocket = if self.web_socket.is_none() {
            self.view_input_code(ctx)
        }
        else {
            html! {
                <div>
                    <p>{ctx.props().code.clone()}</p>
                </div>
            }
        };

        let section_webrtc = if self.web_rtc_connected() {
            html! {
                <table class="table">
                <tbody>
                {
                    for self.files.iter().enumerate().map(|(index, (_, file))| {
                        html! {
                            {self.view_file_row(ctx, index, file)}
                        }
                    })
                }
                </tbody>
                </table>
            }
        }
        else {
            html! {
                <>
                    <div class="col-md-12">
                        <p>{ "Not Connected" }</p>
                    </div>
                </>
            }
        };

        html! {
            <div>
                {section_websocket}
                {section_webrtc}
            </div>
        }
    }
}

impl Client {
    fn web_rtc_request_file(&mut self, tag: FileTag) -> bool {
        let file_item = self.files.get_mut(&tag.uuid());
        if file_item.is_none() {
            return false;
        }
        self.download_manager.new_file(tag.clone());

        let file_item = file_item.unwrap();
        file_item.state = FileState::Transferring;
        self.fetchin_file = Some(tag.clone());

        let file_request = FileRequest {uuid: file_item.tag.uuid()};
        self.web_rtc_manager
            .deref()
            .borrow()
            .send_message(&serde_json::to_string(&file_request).unwrap());
        true
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

    fn update_web_rtc(&mut self, _: &Context<Self>, msg: WebRtcMessage) -> bool {
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
                    console::log_1(&format!("Error saving chunk: {:?}", result.err().unwrap()).into());
                }
                return true;
            }
            WebRtcMessage::UpdateState(state) => {
                if let State::Client(connection_state) = state.clone() {
                    self.on_state_update(&connection_state);
                    self.web_rtc_state = connection_state;
                };
            }
            WebRtcMessage::Reset => {
                console::log_1(&format!("WebRtcMessage::Reset").into());
            }
        }
        true
    }

    fn update_web_socket(&mut self, _: &Context<Self>, msg: WebSocketMessage) -> bool {
        match msg {
            WebSocketMessage::Text(data) => {
                if self.web_rtc_state.ice_connection_state.is_some() &&
                    self.web_rtc_state.ice_connection_state.unwrap() == web_sys::RtcIceConnectionState::Connected {
                    return false;
                }

                let session_check: Result<SessionCheck, serde_json::Error> = serde_json::from_str(&data);
                if session_check.is_ok() {
                    match session_check.unwrap().result {
                        SessionCheckResult::Success(session_host) => {
                            self.session_host = Some(session_host.clone());

                            self.web_rtc_manager.deref().borrow_mut().set_state(State::Client(ConnectionState::new()));
                            let result: Result<(), wasm_bindgen::JsValue> = WebRTCManager::start_web_rtc(&self.web_rtc_manager);
                            if result.is_ok() {
                                let result = WebRTCManager::validate_offer(&self.web_rtc_manager, &session_host.offer);
                                if result.is_err() {
                                    web_sys::Window::alert_with_message(
                                        &web_sys::window().unwrap(),
                                        &format!(
                                            "Cannot use offer. Failure reason: {:?}",
                                            result.err().unwrap()
                                        ),
                                    )
                                    .expect("alert should work");
                                }
                            }
                        }
                        SessionCheckResult::WrongPassword => {
                            self.password_needed = true;
                            console::log_1(&format!("WrongPassword").into());
                        }
                        SessionCheckResult::NotFound => {
                            self.password_needed = false;
                            self.ws_disconnect();
                            console::log_1(&format!("NotFound").into());
                        }
                    }
                }
            }
            WebSocketMessage::Open => {
                let data = SessionDetails::SessionClient(SessionClient::SessionFetchOffer(self.session_details.clone()));
                self.ws_send(data);
            }
            WebSocketMessage::Close => {
                self.ws_disconnect();
            }
            WebSocketMessage::Err => {
                self.ws_disconnect();
            }
        }
        true
    }

    fn view_input_code(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
            <input type="text" ref={self.input_code.clone()} />
            <button onclick={ctx.link().callback(|_| Msg::SessionConnect("".to_string()))}>{ "connect" }</button>
            </>
        }
    }

    fn view_file_row(&self, ctx: &Context<Self>, index: usize, file: &FileItem) -> Html {
        let control_pannel = {
            match file.state {
                FileState::Pending => {
                    let tag = file.tag.clone();
                    html! {
                        <button onclick={ctx.link().callback(move |_| Msg::FileAccept(tag.clone()))}>{ "Accept" }</button>
                    }
                }
                FileState::Transferring => {
                    html! {
                        <p>{format!("Progress: {}%", file.progress)}</p>
                    }
                }
                FileState::Done => {
                    html! {
                        <button onclick={ctx.link().callback(|_| Msg::SessionConnect("".to_string()))}>{ "connect" }</button>
                    }
                }
                FileState::Failed => {
                    html! {
                        <button onclick={ctx.link().callback(|_| Msg::SessionConnect("".to_string()))}>{ "connect" }</button>
                    }
                }
            }
        };

        html! {
            <tr>
                <td>{index}</td>
                <td>{&file.tag.name()}</td>
                <td>{file.tag.size()}</td>
                <td>{format!("{:?}", file.state)}</td>
                <td>{control_pannel}</td>
            </tr>
        }
    }

    fn on_state_update(&mut self, connection_state: &ConnectionState) {
        console::log_1(&format!("UpdateState {:?}", connection_state).into());
        if connection_state.ice_gathering_state != self.web_rtc_state.ice_gathering_state {
            if let Some(state) = connection_state.ice_gathering_state {
                if state == web_sys::RtcIceGatheringState::Complete {
                    let answer = self.web_rtc_manager.deref().borrow().create_encoded_offer();
                    let data = SessionDetails::SessionClient(SessionClient::SessionAnswer(SessionAnswer{
                        code: self.session_details.code.clone(),
                        password: self.session_details.password.clone(),
                        answer,
                    }));
                    
                    self.ws_send(data);
                }
            }
        }
    }

    fn on_files_updates(&mut self, files_update: FilesUpdate) -> bool {
        self.files.clear();
        for file in files_update.files {
            let file_tag = FileTag::new(file.name, file.size, file.uuid);
            self.files.insert(file_tag.uuid(), FileItem {
                state: FileState::Pending,
                tag: file_tag,
                queued: false,
                progress: 0.0,
            });
        }
        true
    }

    fn web_rtc_connected(&self) -> bool {
        matches!(self.web_rtc_state.ice_connection_state, Some(web_sys::RtcIceConnectionState::Connected))
    }
}