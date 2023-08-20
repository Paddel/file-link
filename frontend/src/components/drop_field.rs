// use stylist::yew::use_style;
use yew::prelude::*;
use web_sys::{console, FileList};

pub struct DropField {
    hovering: bool,
}

pub enum Msg {
    DragOver,
    DragLeave,
    Drop,   
}

impl Component for DropField {
    type Message = Msg;
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self {
            hovering: false,
        }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DragOver => {
                self.hovering = true;
            }
            Msg::DragLeave => {
                self.hovering = false;
            }
            Msg::Drop => {
                self.hovering = false;
            },

        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let drag_over = ctx.link().callback(|event: DragEvent| {
            event.prevent_default();
            Msg::DragOver
        });
        let drag_leave = ctx.link().callback(|_: DragEvent| Msg::DragLeave);
        let drop = ctx.link().callback(|event: DragEvent| {
            event.prevent_default();
            let data_transfer = event.data_transfer().unwrap();
            if let Ok(text) = data_transfer.get_data("text") {
                if !text.is_empty() {
                    web_sys::console::log_1(&format!("Dropped text: {}", text).into());
                }
            }
            // let files = event.data_transfer().unwrap().files().unwrap();
            // let len = files.length();
            // for i in 0..len {
            //     if let Some(file) = files.item(i) {
            //         let file_name = file.name();
            //         web_sys::console::log_1(&format!("File name: {}", file_name).into());
            //     }
            // }
            Msg::Drop
        });

        html! {
            <div
                class={if self.hovering { "drop-field hovering" } else { "drop-field" }}
                ondragover={drag_over}
                ondragleave={drag_leave}
                ondrop={drop}
            >
                { "Drop here" }
            </div>
        }
    }
}
