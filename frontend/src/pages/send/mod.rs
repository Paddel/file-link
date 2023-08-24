use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{DerefMut, Deref};
use std::rc::Rc;
use std::sync::Arc;

use anyhow::Error;
use serde::{Deserialize, Serialize};
use web_sys::console;
use yew::{Html, html, Callback, Context, ContextProvider, Component, function_component, HtmlResult, Suspense, use_effect_with_deps, use_prepared_state, use_state};

use file_item::Indicator;
use drop_files::DropFiles;
use drop_receiver::DropReceiver;
use serde_json::json;
use yew_websocket::websocket::{WebSocketTask, WebSocketService, WebSocketStatus};

use crate::components::file_list::{FileList, FileListItem};
use crate::services::web_rtc::{NetworkManager, State, ConnectionState, ConnectionString, WebRtcMessage};
use crate::ws_macros::{Json, WsResponse, WsRequest};

mod file_item;
mod drop_files;
mod drop_receiver;

include!(concat!(env!("OUT_DIR"), "/config.rs"));
include!("../../../../shared/ws_protocol.rs");

#[derive(Serialize, Deserialize)]
struct ConnectionLinkResponse {
    code: String,
}

async fn fetch_uuid() -> String {
    console::log_1(&format!("fetch_uuid").into());
    // reqwest works for both non-wasm and wasm targets.
    let resp = reqwest::get("https://httpbin.org/uuid").await.unwrap();
    let link_resp = resp.json::<ConnectionLinkResponse>().await.unwrap();

    link_resp.code
}

// async fn fetch_uuid() -> String {
//     let resp: reqwest::Response = reqwest::get("/robots.txt").await.unwrap();
//     // let resp: reqwest::Response = reqwest::get("https://raw.githubusercontent.com/serkanyersen/jsonplus/master/package.json").await.unwrap();
//     // let resp: reqwest::Response = reqwest::get("https://httpbin.org/uuid").await.unwrap();
//     let id_resp = resp.json::<UuidResponse>().await.unwrap();
//     id_resp.id
// }

#[function_component]
fn Content() -> HtmlResult {
    let code = use_prepared_state!(async move |_| -> String { fetch_uuid().await }, ())?.unwrap();

    Ok(html! {
        <div>{"Fetched code: "}{code}</div>
    })
}


async fn fetch_connect_link() -> Result<String, bool> {
    let body = reqwest::get("/robots.txt").await
    // let body = reqwest::get("https://raw.githubusercontent.com/Apuju/Public/master/Test.txt").await
        .map_err(|_| false)?
        .text().await
        .map_err(|_| false)?;
    Ok(body)
}

#[function_component(ConnectLink)]
fn connect_link() -> HtmlResult {
    // console::log_1(&format!("connect_link").into());
    let result: Option<Rc<Result<String, bool>>> = use_prepared_state!(async move |_| -> Result<String, bool> { fetch_connect_link().await }, ())?;
    match result {
        Some(link) => {
            Ok(html! {
                <div>{"Random UUID: "}{match &*link {
                    Ok(val) => val.clone(),
                    Err(_) => String::from("Error b"),
                }}</div>
                
            })
        }
        None => Ok(html! { 
            <div>{"Failed a"}</div>
        })
    }
}

// match result {
    // Ok(link) => {
    //     Ok(html! {
    //         <div>{"Random UUID: "}{link}</div>
    //     })
    // }
//     Err(error) => {
//         Ok(html! {
//             <div>{"Oops! Something went wrong. Please refresh the page"}</div>
//         })
//     }
// }


// let content = match result {
//     Ok(link) => link,
//     Err(_) => "Oops! Something went wrong. Please refresh the page".to_string(),
// };

#[derive(Clone, Debug, PartialEq)]
pub struct PageContext {
    is_dragging: bool,
    on_drag: Callback<bool>,
    session_start: Callback<SessionDetails>,
}

pub enum Msg {
    Drag(bool),
    Drop(String),
    AddFile(FileListItem),
    
    WsConnect(),
    WsOpened(),
    WsReady(Result<SessionCode, Error>),
    WsDisconnect,
    WsLost,

    CallbackWebRTC(WebRtcMessage),
}

pub struct Send<T: NetworkManager + 'static> {
    context: Rc<PageContext>,
    files: HashMap<String, FileListItem>,
    web_rtc_manager: Rc<RefCell<T>>,

    ws: Option<WebSocketTask>,
    pub ws_fetching: bool,
    code: String,
}

impl<T: NetworkManager + 'static> Component for Send<T> {
    type Message = Msg;
    type Properties = ();
     
    
    fn create(ctx: &Context<Self>) -> Self {
        let callback_webrtc = ctx.link().callback(Msg::CallbackWebRTC);
        let on_drag = ctx.link().callback(Msg::Drag);
        let session_start = ctx.link().callback(|_| Msg::WsConnect());
        let context = Rc::new(PageContext{is_dragging: false, on_drag, session_start});
        
        Send {
            context,
            files: HashMap::new(),
            web_rtc_manager: T::new(callback_webrtc),
            ws: None,
            ws_fetching: false,
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
            Msg::WsConnect() => {
                self.web_rtc_manager.deref().borrow_mut().set_state(State::Server(ConnectionState::new()));
                let result: Result<(), wasm_bindgen::JsValue> = T::start_web_rtc(self.web_rtc_manager.clone());
                console::log_1(&format!("result: {:?}", result).into());
                true
            }
            Msg::WsOpened() => {
                console::log_1(&format!("ws opened").into());
                let offer = self.web_rtc_manager.deref().borrow_mut().create_offer();
                let data = SessionDetails::SessionHost(SessionHost{
                    mode: "host".to_string(),
                    offer, password: "".to_string(),
                    compression: 9,
                });
                self.ws_send_host(data);
                false
            }
            Msg::WsDisconnect => {
                self.ws_disconnect();
                true
            }
            Msg::WsLost => {
                self.ws_disconnect();
                true
            }
            Msg::WsReady(response) => {
                let data = response.map(|data| data).ok();
                if data.is_some() {
                    console::log_1(&format!("ws ready: {:?}", data).into());
                    self.code = data.unwrap().code;
                }
                true
            }
            Msg::CallbackWebRTC(msg) => {
                match msg {
                    WebRtcMessage::Message(blob) => {
                        console::log_1(&format!("Message: {:?}", blob).into());
                    }
                    WebRtcMessage::UpdateState(state) => {
                        if let State::Server(connection_state) = state {
                            if connection_state.ice_gathering_state.is_some() {
                                console::log_1(&format!("Server: {:?}", connection_state).into());
                                self.ws_connect(ctx);
                            }
                        };
                    }
                    WebRtcMessage::Reset => {
                        console::log_1(&format!("Reset").into());
                    }
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let fallback = html! {<div>{"Loading..."}</div>};
        let page_state = self.context.clone();
        html! {
            <ContextProvider<Rc<PageContext>> context={page_state}>
                <div class="container">
                    <div class="row">
                        <div class="col-md-6">
                            <DropFiles />
                        </div>
                        <div class="col-md-6">
                            <div class="row">
                                <div class="col-md-12">
                                // <Suspense {fallback}>
                                //     <Content />
                                // </Suspense>
                                    // <input type="text" readonly={true} value={self.invite_link} />
                                    <button onclick={ctx.link().callback(|_| Msg::WsConnect())}>{ "connect" }</button>
                                    <p>{&self.code}</p>
                                </div>
                            </div>
                            <div class="row">
                                <div class="col-md-12">
                                    <Indicator>
                                        <DropReceiver drop={ctx.link().callback(|id| Msg::Drop(id))} />
                                    </Indicator>
                                </div>
                            </div>
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

impl<T: NetworkManager + 'static> Send<T> {
    fn ws_connect(&mut self, ctx: &Context<Self>) {
        let callback = ctx.link().callback(|Json(data)| Msg::WsReady(data));
        let notification: Callback<WebSocketStatus> = ctx.link().batch_callback(move |status| match status {
            WebSocketStatus::Opened => {Some(WebRtcMessage::Reset)},
            WebSocketStatus::Closed | WebSocketStatus::Error => {
                console::log_1(&format!("ws close: {:?}", status).into());
                Some(WebRtcMessage::Reset)
            }
        });
        let task = match WebSocketService::connect(
            WEBSOCKET_ADDRESS,
            callback,
            notification,
        ) {
            Ok(task) => Some(task),
            Err(_) => None,
        };
        self.ws = task;
        console::log_1(&format!("ws: {:?}", self.ws).into());
    }

    fn ws_disconnect(&mut self) {
        self.ws = None;
    }

    fn ws_send_host(&mut self, data: SessionDetails) {
        // self.ws.borrow_mut().unwrap().send(serde_json::to_string(&data).unwrap());
        self.ws
            .as_mut()
            .unwrap()
            .send(serde_json::to_string(&data).unwrap());
        console::log_1(&format!("ws2: {:?}", self.ws).into());
    }
}