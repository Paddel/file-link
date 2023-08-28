use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use web_sys::{console, File, Blob};
use yew::platform::spawn_local;
use yew::{Html, html, Context, Component};

use drop_files::DropFiles;

use crate::file_tag::{FileState, FileTag};
use crate::wrtc_protocol::{FilesUpdate, FileInfo, FileRequest};
use crate::services::web_rtc::{State, ConnectionState, WebRtcMessage, WebRTCManager};
use crate::services::web_socket::{WsConnection, WebSocketMessage};

mod drop_files;

include!(concat!(env!("OUT_DIR"), "/config.rs"));
include!("../../../../shared/ws_protocol.rs");

const COMPRESSION_DEFAULT: u8 = 9;

#[derive(Clone)]
pub struct FileItem {
    state: FileState,
    pub tag: FileTag,
    js_file: File,
    progress: f64,
}

pub enum Msg {
    Update(Vec<File>),
    SessionStart,

    CallbackWebRTC(WebRtcMessage),
    CallbackWebsocket(WebSocketMessage),
}

pub struct Host {
    web_rtc_manager: Rc<RefCell<WebRTCManager>>,
    web_rtc_state: ConnectionState,
    web_socket: Option<WsConnection>,
    files: HashMap<Uuid, FileItem>,
    code: String,
}

impl Component for Host {
    type Message = Msg;
    type Properties = ();
     
    
    fn create(ctx: &Context<Self>) -> Self {
        Host {
            web_rtc_manager: WebRTCManager::new(ctx.link().callback(Msg::CallbackWebRTC)),
            web_rtc_state: ConnectionState::new(),
            web_socket: None,
            files: HashMap::new(),
            code: String::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Update(files) => {
                self.handle_files(files);
                true
            }
            Msg::SessionStart => {
                self.web_rtc_manager.deref().borrow_mut().set_state(State::Server(ConnectionState::new()));
                let _: Result<(), wasm_bindgen::JsValue> = WebRTCManager::start_web_rtc(&self.web_rtc_manager);
                true
            }
            Msg::CallbackWebRTC(msg) => {
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
            let uuid = Uuid::new_v4();
            let item = FileItem {
                state: FileState::Pending,
                tag: FileTag::from(file.clone()),
                js_file: file.clone(),
                progress: 0.0,
            };
            self.files.insert(uuid, item);
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

        let update = FilesUpdate {files};
        let message = serde_json::to_string(&update).unwrap();
        self.web_rtc_manager
        .deref()
        .borrow()
        .send_message(&message);
    }

    fn web_rtc_connected(&self) -> bool {
        !matches!(self.web_rtc_state.ice_connection_state, Some(web_sys::RtcIceConnectionState::Connected))
    }
    
    fn web_rtc_send_file(&self, uuid: Uuid) {
        let file = self.files.iter().find(|(_, file)| file.tag.uuid() == uuid);	
        if file.is_none() {
            return;
        }
        
        let file = file.unwrap().1.clone();
        let web_rtc_manager = self.web_rtc_manager.clone();
        console::log_1(&format!("Sending file {}", file.tag.name()).into());
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
                    .send_data(&chunk)
                    .await;

                if !sent_success {
                    console::log_1(&format!("Failed to send chunk").into());
                    return;
                }
                console::log_1(&format!("Sent chunk x").into());

                offset += CHUNK_SIZE;
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
                console::log_1(&format!("WebRtcMessage::Message {}", data).into());
                let update: Result<FileRequest, serde_json::Error> = serde_json::from_str(&data);
                if update.is_err() {
                    return false;
                }

                self.web_rtc_send_file(update.unwrap().uuid);
                true
            }
            WebRtcMessage::Data(_, _) => {
                //Host should not receive data
                false
            }
            WebRtcMessage::UpdateState(state) => {
                let mut update = false;
                if let State::Server(connection_state) = state.clone() {
                    console::log_1(&format!("UpdateState {:?}", connection_state).into());
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
                self.web_rtc_manager = WebRTCManager::new(ctx.link().callback(Msg::CallbackWebRTC));
                console::log_1(&format!("Client Reset").into());
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
                            let _ = WebRTCManager::validate_answer(&self.web_rtc_manager, &answer);
                        }
                    }
                }
                update
            }
            WebSocketMessage::Open => {
                let offer = self.web_rtc_manager.deref().borrow_mut().create_encoded_offer();
                let data = SessionDetails::SessionHost(SessionHost{
                    offer,
                    password: "".to_string(),
                    compression: COMPRESSION_DEFAULT,
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
                <>
                <div class="col-md-12">
                    <button onclick={ctx.link().callback(|_| Msg::SessionStart)}>{"connect"}</button>
                </div>
                </>
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

    fn view_session_handle (&self, ctx: &Context<Self>) -> Html {
        html! {
                <table class="table">
                    <tbody>
                        <DropFiles onupdate={ctx.link().callback(Msg::Update)} />
                        {
                            for self.files.iter().enumerate().map(|(index, (_, file))| {
                                html! {
                                    <tr>
                                        <td>{index}</td>
                                        <td>{&file.tag.name()}</td>
                                        <td>{file.tag.size()}</td>
                                        <td>{format!("{:?}", file.state)}</td>
                                        <td>{"Waiting for Receiver"}</td>
                                    </tr>
                                }
                            })
                         }
                </tbody>
            </table>
        }
    }
}