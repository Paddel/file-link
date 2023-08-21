use std::rc::Rc;

use yew::{Callback, Context, ContextProvider, Component, Html, html};
use web_sys::console;

use file_item::Indicator;
use drop_files::DropFiles;
use drop_receiver::DropReceiver;

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
}

pub struct Send {
    state: Rc<PageState>,
}

impl Component for Send {
    type Message = Msg;
    type Properties = ();
     
    
    fn create(ctx: &Context<Self>) -> Self {
        let on_drag = ctx.link().callback(Msg::Drag);
        let state = Rc::new(PageState{is_dragging: false, on_drag});
        Send {state}
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Drag(value) => {
                Rc::make_mut(&mut self.state).is_dragging = value;
            }
            Msg::Drop(id) => {
                console::log_1(&format!("dropped: {}", id).into());
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
                            <Indicator>
                                <DropReceiver drop={ctx.link().callback(|id| Msg::Drop(id))} />
                            </Indicator>
                        </div>
                    </div>
                    <div class="row">
                        <div class="col-md-12">
                            <p>{"This is the third div with full width"}</p>
                        </div>
                    </div>
                </div>
            </ContextProvider<Rc<PageState>>>
            </>
        }
    }
}