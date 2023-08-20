use yew::{html, Callback, Context, Component, Html};

use crate::components::drop_field::DropField;
use crate::components::draggable::{Draggable, Indicator};

pub enum Msg {
    Drag(bool),
}

pub struct Send {
    is_dragging: bool,
}

impl Component for Send {
    type Message = Msg;
    type Properties = ();
     
    
    fn create(_: &Context<Self>) -> Self {

        Send {
            is_dragging: false,
        }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Drag(value) => {
                self.is_dragging = value;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {

        let scope = ctx.link().clone();
        let drag: Callback<bool> = Callback::from(move |value: bool| {
            scope.send_message(Msg::Drag(value));
        });
        
        // ctx.link().send_message(Msg::Drag(value));
        html! {
            <div>
                <Indicator dragging={self.is_dragging}>
                    <DropField />
                </Indicator>
                <Draggable {drag} />
            </div>
        }
    }
}