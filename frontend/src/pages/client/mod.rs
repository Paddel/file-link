use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{HtmlInputElement, console};
use yew::prelude::*;

use crate::components::file_list::{FileList, FileListItem};
use crate::file_tag::FileTag;
use crate::services::web_rtc::{WebRTCManager, WebRtcMessage, State, ConnectionState};
use crate::services::web_socket::{WsConnection, WebSocketMessage};

include!(concat!(env!("OUT_DIR"), "/config.rs"));
include!("../../../../shared/ws_protocol.rs");

pub enum Msg {
    InputCode(String),
    SessionConnect(String),
    AddFile(FileListItem),

    CallbackWebRtc(WebRtcMessage),
    CallbackWebsocket(WebSocketMessage),
}

#[derive(Properties, PartialEq)]
pub struct ReceiveProps {
    #[prop_or(String::new())]
    pub code: String,
}

pub struct Receive {
    web_rtc_manager: Rc<RefCell<WebRTCManager>>,
    web_rtc_state: ConnectionState,
    web_socket: Option<WsConnection>,
    code: String,
    password: String,
    password_needed: bool,
    session_host: Option<SessionHost>,
}

impl Component for Receive {
    type Message = Msg;
    type Properties = ReceiveProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            web_rtc_manager: WebRTCManager::new(ctx.link().callback(Msg::CallbackWebRtc)),
            web_rtc_state: ConnectionState::new(),
            web_socket: None,
            code: ctx.props().code.clone(),
            password: String::new(),
            password_needed: false,
            session_host: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AddFile(file) => {
                // console::log_1(&format!("added file: {}", file.tag.name()).into());
            }
            Msg::InputCode(code) => {
                self.code = code;
            }
            Msg::SessionConnect(password) => {
                self.password = password;
                self.ws_connect(ctx);
            }
            Msg::CallbackWebRtc(msg) => {
                match msg {
                    WebRtcMessage::Message(data) => {
                        console::log_1(&format!("WebRtcMessage::Message {}", data).into());
                    }
                    WebRtcMessage::UpdateState(state) => {
                        if let State::Client(connection_state) = state.clone() {
                            if connection_state.ice_gathering_state != self.web_rtc_state.ice_gathering_state {
                                if let Some(state) = connection_state.ice_gathering_state {
                                    if state == web_sys::RtcIceGatheringState::Complete {
                                        let answer = self.web_rtc_manager.deref().borrow_mut().create_encoded_offer();
                                        let data = SessionDetails::SessionClient(SessionClient::SessionAnswer(SessionAnswer{
                                            code: self.code.clone(),
                                            password: self.password.clone(),
                                            answer,
                                        }));
                                        
                                        self.ws_send(data);
                                    }
                                }
                            }

                            self.web_rtc_state = connection_state;
                        };
                    }
                    WebRtcMessage::Reset => {
                        console::log_1(&format!("WebRtcMessage::Reset").into());
                    }
                }
            }
            Msg::CallbackWebsocket(msg) => {
                match msg {
                    WebSocketMessage::Text(data) => {
                        if self.web_rtc_state.data_channel_state.is_some() &&
                            self.web_rtc_state.data_channel_state.unwrap() == web_sys::RtcDataChannelState::Open {
                            return false;
                        }

                        let session_check: Result<SessionCheck, serde_json::Error> = serde_json::from_str(&data);
                        if session_check.is_ok() {
                            match session_check.unwrap().result {
                                SessionCheckResult::Success(session_host) => {
                                    self.session_host = Some(session_host.clone());

                                    self.web_rtc_manager.deref().borrow_mut().set_state(State::Client(ConnectionState::new()));
                                    let result: Result<(), wasm_bindgen::JsValue> = WebRTCManager::start_web_rtc(self.web_rtc_manager.clone());
                                    if result.is_ok() {
                                        let result = WebRTCManager::validate_offer(self.web_rtc_manager.clone(), &session_host.offer);
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
                        let data = SessionDetails::SessionClient(SessionClient::SessionFetchOffer(SessionFetchOffer{
                            code: self.code.clone(),
                            password: self.password.clone(),
                        }));
                        
                        self.ws_send(data);
                    }
                    WebSocketMessage::Close => {
                        self.ws_disconnect();
                    }
                    WebSocketMessage::Err => {
                        self.ws_disconnect();
                    }
                }
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

        let section_webrtc = if self.web_rtc_state.data_channel_state.is_some() &&
            self.web_rtc_state.data_channel_state.unwrap() == web_sys::RtcDataChannelState::Open {
            html! {
                <>
                    <div class="col-md-12">
                        <p>{ "Connected" }</p>
                    </div>
                </>
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

impl Receive {
    fn view_input_code(&self, ctx: &Context<Self>) -> Html {

        let callback = ctx.link().callback(|value| Msg::InputCode(value));
        let oninput = Callback::from(move |input_event: InputEvent| {
            if let Some(target) = input_event.target() {
                let input: HtmlInputElement = target.dyn_into().unwrap_throw();
                let value = input.value();
                callback.emit(value);
            }
        });

        html! {
            <>
            <input type="text" {oninput} />
            <button onclick={ctx.link().callback(|_| Msg::SessionConnect("".to_string()))}>{ "connect" }</button>
            </>
        }
    }

    fn ws_connect(&mut self, ctx: &Context<Self>) {
        let callback = ctx.link().callback(Msg::CallbackWebsocket);
        self.web_socket = WsConnection::new(WEBSOCKET_ADDRESS, callback).ok();
    }

    fn ws_disconnect(&mut self) {
        self.web_socket = None;
    }

    fn ws_send(&mut self, data: SessionDetails) {
        // self.ws.borrow_mut().unwrap().send(serde_json::to_string(&data).unwrap());
        self.web_socket
            .as_mut()
            .unwrap()
            .send_text(serde_json::to_string(&data).unwrap());
    }
}