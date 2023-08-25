use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use web_sys::console;
use yew::{Html, html, Callback, Context, ContextProvider, Component};

use file_item::Indicator;
use drop_files::DropFiles;
use drop_receiver::DropReceiver;

use crate::components::file_list::{FileList, FileListItem};
use crate::services::web_rtc::{State, ConnectionState, WebRtcMessage, WebRTCManager};
use crate::services::web_socket::{WsConnection, WebSocketMessage};

mod file_item;
mod drop_files;
mod drop_receiver;

include!(concat!(env!("OUT_DIR"), "/config.rs"));
include!("../../../../shared/ws_protocol.rs");

#[derive(Clone, Debug, PartialEq)]
pub struct PageContext {
    is_dragging: bool,
    on_drag: Callback<bool>,
}

pub enum Msg {
    Drag(bool),
    Drop(String),
    AddFile(FileListItem),
    SessionStart,

    CallbackWebRTC(WebRtcMessage),
    CallbackWebsocket(WebSocketMessage),
}

pub struct Send {
    context: Rc<PageContext>,
    files: HashMap<String, FileListItem>,
    web_rtc_manager: Rc<RefCell<WebRTCManager>>,
    web_rtc_state: ConnectionState,
    web_socket: Option<WsConnection>,
    code: String,
}

impl Component for Send {
    type Message = Msg;
    type Properties = ();
     
    
    fn create(ctx: &Context<Self>) -> Self {
        let on_drag = ctx.link().callback(Msg::Drag);
        let context = Rc::new(PageContext{is_dragging: false, on_drag});
        
        Send {
            context,
            files: HashMap::new(),
            web_rtc_manager: WebRTCManager::new(ctx.link().callback(Msg::CallbackWebRTC)),
            web_rtc_state: ConnectionState::new(),
            web_socket: None,
            code: String::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Drag(value) => {
                Rc::make_mut(&mut self.context).is_dragging = value;
                true
            }
            Msg::Drop(id) => {
                console::log_1(&format!("dropped: {}", id).into());
                true
            }
            Msg::AddFile(file) => {
                self.files.insert(file.tag.uuid().to_string(), file);
                true
            }
            Msg::SessionStart => {
                self.web_rtc_manager.deref().borrow_mut().set_state(State::Server(ConnectionState::new()));
                let result: Result<(), wasm_bindgen::JsValue> = WebRTCManager::start_web_rtc(self.web_rtc_manager.clone());
                console::log_1(&format!("result: {:?}", result).into());
                true
            }
            Msg::CallbackWebRTC(msg) => {
                match msg {
                    WebRtcMessage::Message(blob) => {
                        console::log_1(&format!("Webrtc Message:").into());
                    }
                    WebRtcMessage::UpdateState(state) => {
                        console::log_1(&format!("WebRtcMessage::UpdateState {:?}", state).into());

                        if let State::Server(connection_state) = state.clone() {
                            if connection_state.ice_gathering_state != self.web_rtc_state.ice_gathering_state {
                                console::log_1(&format!("Server ICE Gathering State: {:?}", state).into());
                                
                                if let Some(state) = connection_state.ice_gathering_state {
                                    if state == web_sys::RtcIceGatheringState::Complete {
                                        self.ws_connect(ctx);
                                    }
                                }
                            }
                            
                            if connection_state.data_channel_state != self.web_rtc_state.data_channel_state {
                                console::log_1(&format!("Server Data Channel State: {:?}", state).into());
                                
                                if let Some(state) = connection_state.data_channel_state {
                                    if state == web_sys::RtcDataChannelState::Open {
                                        console::log_1(&format!("Server Data Channel State: {:?}", state).into());
                                        self.web_rtc_manager.deref().borrow_mut().send_message("test");
                                    }
                                }
                            }

                            self.web_rtc_state = connection_state;
                        };
                    }
                    WebRtcMessage::Reset => {
                        console::log_1(&format!("Reset").into());
                    }
                }
                true
            }
            Msg::CallbackWebsocket(msg) => {
                match msg {
                    WebSocketMessage::Text(data) => {
                        console::log_1(&format!("Message: {:?}", data).into());
                        let session_host_result: Result<SessionHostResult, serde_json::Error> = serde_json::from_str(&data);

                        if let Ok(session_host_result) = session_host_result {
                            match session_host_result {
                                SessionHostResult::SessionCode(session_code) => {
                                    self.code = session_code.code;
                                    console::log_1(&format!("code: {}", self.code).into());
                                }
                                SessionHostResult::SessionAnswerForward(session_answer_forward) => {
                                    // console::log_1(&format!("answer: {}", session_answer_forward.answer).into());
                                    let answer = session_answer_forward.answer;
                                    let result = WebRTCManager::validate_answer(self.web_rtc_manager.clone(), &answer);
                                    console::log_1(&format!("result: {:?}", result).into());
                                    // if result.is_ok() {
                                    //     self.web_rtc_manager.deref().borrow_mut().send_message("test");
                                    // }
                                }
                            }
                        }
                        else {
                            console::log_1(&format!("Error:").into());
                        }
                    }
                    WebSocketMessage::Open => {
                        let offer = self.web_rtc_manager.deref().borrow_mut().create_encoded_offer();
                        let data = SessionDetails::SessionHost(SessionHost{
                            mode: "host".to_string(),
                            offer,
                            password: "".to_string(),
                            compression: 9,
                        });
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
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let page_state = self.context.clone();
        html! {
            <ContextProvider<Rc<PageContext>> context={page_state}>
                <div class="container">
                    <div class="row">
                        <div class="col-md-6">
                            <DropFiles />
                        </div>
                        <div class="col-md-6">
                            {self.view_networking(ctx)}
                        </div>
                    </div>
                    <FileList>
                    {
                        self.files.clone().into_iter().enumerate().map(|(index, (_, file))| {
                            html! {
                                <>
                                <td>{index}</td>
                                <td>{file.tag.name()}</td>
                                <td>{file.tag.blob().size()}</td>
                                <td>{format!("{:?}", file.state)}</td>
                                <td>{"Waiting for Receiver"}</td>
                                </>
                                }
                        }).collect::<Html>()
                    }
                    </FileList>
                </div>
            </ContextProvider<Rc<PageContext>>>
        }
    }
}

impl Send {
    fn ws_connect(&mut self, ctx: &Context<Self>) {
        let callback = ctx.link().callback(Msg::CallbackWebsocket);
        self.web_socket = WsConnection::new(WEBSOCKET_ADDRESS, callback).ok();
        console::log_1(&format!("ws: ").into());
    }

    fn ws_disconnect(&mut self) {
        self.web_socket = None;
    }

    fn ws_send(&mut self, data: SessionDetails) {
        console::log_1(&format!("sending").into());
        // self.ws.borrow_mut().unwrap().send(serde_json::to_string(&data).unwrap());
        self.web_socket
            .as_mut()
            .unwrap()
            .send_text(serde_json::to_string(&data).unwrap());
        console::log_1(&format!("ws2:").into());
    }

    fn view_networking(&self, ctx: &Context<Self>) -> Html {
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
                    <button onclick={ctx.link().callback(|_| Msg::SessionStart)}>{ "connect" }</button>
                </div>
                </>
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
            <>
                <div class="row">
                    {section_websocket}
                </div>
                <div class="row">
                    <div class="col-md-12">
                        {section_webrtc}
                        <Indicator>
                            <DropReceiver drop={ctx.link().callback(|id| Msg::Drop(id))} />
                        </Indicator>
                    </div>
                </div>
            </>
        }
    }
}