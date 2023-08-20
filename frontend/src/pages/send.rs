use yew::{html, Callback, Context, Component, DragEvent, Html};

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

        let dropfield_files_accepted = Callback::from(move |event: DragEvent| {
            let types = event.data_transfer().unwrap().types();
            let mut accepted = false;
            for i in 0..types.length() {
                if types.at(i.try_into().unwrap()) == "Files" {
                    accepted = true;
                    break;
                }
            }
            accepted
        });

        let dropfield_strings_accepted = Callback::from(move |event: DragEvent| {
            let types = event.data_transfer().unwrap().types();
            let mut accepted = false;
            for i in 0..types.length() {
                if types.at(i.try_into().unwrap()) == "text/plain" {
                    accepted = true;
                    break;
                }
            }
            accepted
        });

        html! {
            <>
            <div class="container">
                <div class="row">
                    <div class="col-md-6">
                        <DropField accepting={dropfield_strings_accepted} />
                        <Draggable {drag} />
                    </div>  
                    <div class="col-md-6">
                        <Indicator dragging={self.is_dragging}>
                            <DropField accepting={dropfield_files_accepted} />
                        </Indicator>
                    </div>
                </div>
                <div class="row">
                    <div class="col-md-12">
                        <p>{"This is the third div with full width"}</p>
                    </div>
                </div>
            </div>
            </>
        }
    }
}