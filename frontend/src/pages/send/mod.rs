use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use yew::{Callback, Context, ContextProvider, Component, Html, html};
use web_sys::console;

use file_item::Indicator;
use drop_files::DropFiles;
use drop_receiver::DropReceiver;

use crate::file_tag::FileTag;
use crate::components::file_list::{FileList, FileListItem};
use crate::web_rtc_manager::{CallbackType, NetworkManager, WebRtcManager};

mod file_item;
mod drop_files;
mod drop_receiver;

#[derive(Clone, Debug, PartialEq)]
pub struct PageState {
    is_dragging: bool,
    on_drag: Callback<bool>,
}

pub enum Msg {
    Drag(bool),
    Drop(String),
    AddFile(FileListItem),
}

pub struct Send {
    state: Rc<PageState>,
    files: HashMap<String, FileListItem>,
    web_rtc_manager: Rc<RefCell<WebRtcManager>>,
}

impl Component for Send {
    type Message = Msg;
    type Properties = ();
     
    
    fn create(ctx: &Context<Self>) -> Self {
        let on_drag = ctx.link().callback(Msg::Drag);
        let state = Rc::new(PageState{is_dragging: false, on_drag});
        let callback: Arc<Box<dyn Fn(CallbackType)>> = Arc::new(Box::new(|_type: CallbackType| {
            // Your logic here
        }));
        
        Send {
            state,
            files: HashMap::new(),
            web_rtc_manager: WebRtcManager::new(callback),
        }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Drag(value) => {
                Rc::make_mut(&mut self.state).is_dragging = value;
            }
            Msg::Drop(id) => {
                console::log_1(&format!("dropped: {}", id).into());
            }
            Msg::AddFile(file) => {
                self.files.insert(file.tag.uuid().to_string(), file);
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {

        let page_state = self.state.clone();
        html! {
            <>
                <ContextProvider<Rc<PageState>> context={page_state}>
                    <div class="container">
                        <div class="row">
                            <div class="col-md-6">
                                <DropFiles />
                            </div>
                            <div class="col-md-6">
                                <div class="row">
                                    <div class="col-md-12">
                                        <input type="text" readonly={true} value="Loading.." />
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
                </ContextProvider<Rc<PageState>>>
            </>
        }
    }
}