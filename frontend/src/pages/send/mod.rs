use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::Error;
use serde::{Deserialize, Serialize};
use web_sys::console;
use yew::{Html, html, Callback, Context, ContextProvider, Component, function_component, HtmlResult, Suspense, use_effect_with_deps, use_prepared_state, use_state};

use file_item::Indicator;
use drop_files::DropFiles;
use drop_receiver::DropReceiver;
use yew_websocket::websocket::{WebSocketTask, WebSocketService, WebSocketStatus};

use crate::components::file_list::{FileList, FileListItem};
use crate::services::web_rtc::{CallbackType, NetworkManager, WebRtcManager};
use crate::ws_macros::{Json, WsResponse, WsRequest};

mod file_item;
mod drop_files;
mod drop_receiver;


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

struct WsSessionDetails(String, bool, u8);

#[derive(Clone, Debug, PartialEq)]
pub struct PageContext {
    is_dragging: bool,
    on_drag: Callback<bool>,
    session_start: Callback<WsSessionDetails>,
}

pub enum Msg {
    Drag(bool),
    Drop(String),
    AddFile(FileListItem),
    
    WsConnect(WsSessionDetails),
    WsReady(Result<WsResponse, Error>),
    WsSendData(bool),
    WsDisconnect,
    WsLost,
}

pub struct Send {
    context: Rc<PageContext>,
    files: HashMap<String, FileListItem>,
    web_rtc_manager: Rc<RefCell<WebRtcManager>>,

    ws: Option<WebSocketTask>,
    pub ws_fetching: bool,
    pub ws_data: Option<u32>,
}

impl Component for Send {
    type Message = Msg;
    type Properties = ();
     
    
    fn create(ctx: &Context<Self>) -> Self {
        let on_drag = ctx.link().callback(Msg::Drag);
        let session_start = ctx.link().callback(|details: WsSessionDetails| Msg::WsConnect(details));
        let context = Rc::new(PageContext{is_dragging: false, on_drag, session_start});
        let callback: Arc<Box<dyn Fn(CallbackType)>> = Arc::new(Box::new(|_type: CallbackType| {
            // Your logic here
        }));
        
        Send {
            context,
            files: HashMap::new(),
            web_rtc_manager: WebRtcManager::new(callback),
            ws: None,
            ws_fetching: false,
            ws_data: None,
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
            Msg::WsConnect(details) => {
                let callback = ctx.link().callback(|Json(data)| Msg::WsReady(data));
                let notification = ctx.link().batch_callback(|status| match status {
                    WebSocketStatus::Opened => None,
                    WebSocketStatus::Closed | WebSocketStatus::Error => {
                        Some(Msg::WsLost.into())
                    }
                });
                let task = match WebSocketService::connect(
                    // "wss://echo.websocket.events/",
                    "ws://127.0.0.1:9000/",
                    callback,
                    notification,
                ) {
                    Ok(task) => Some(task),
                    Err(_) => None,
                };
                self.ws = task;

                true
            }
            Msg::WsSendData(binary) => {
                let request = WsRequest { value: 321 };
                if binary {
                    self.ws
                        .as_mut()
                        .unwrap()
                        .send_binary(serde_json::to_string(&request).unwrap().into_bytes());
                } else {
                    self.ws
                        .as_mut()
                        .unwrap()
                        .send(serde_json::to_string(&request).unwrap());
                }
                false
            }
            Msg::WsDisconnect => {
                self.ws = None;
                true
            }
            Msg::WsLost => {
                self.ws = None;
                true
            }
            Msg::WsReady(response) => {
                self.ws_data = response.map(|data| data.value).ok();
                console::log_1(&format!("data: {:?}", self.ws_data).into());
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
                                    <button onclick={ctx.link().callback(|_| Msg::WsConnect(WsSessionDetails("".to_string(), true, 9)))}>{ "connect" }</button>
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