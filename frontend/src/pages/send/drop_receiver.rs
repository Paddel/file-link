use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub drop: Callback<String>,
}

pub enum Msg {
    DragOver(DragEvent),
    DragLeave(DragEvent),
    Drop(DragEvent),
}
pub struct DropReceiver {
    hovering: bool,
}

impl Component for DropReceiver {
    type Message = Msg;
    type Properties = Props;

    fn create(_: &Context<Self>) -> Self {
        Self {
            hovering: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DragOver(event) => {
                if !Self::accepting(event) {return false;}
                self.hovering = true;
            }
            Msg::DragLeave(event) => {
                if !Self::accepting(event) {return false;}
                self.hovering = false;
            }
            Msg::Drop(event) => {
                if !Self::accepting(event.clone()) {return false;}
                self.hovering = false;

                let text = event.data_transfer().unwrap().get_data("text").unwrap();
                ctx.props().drop.emit(text);
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut class = "drop-field".to_string();
        if self.hovering {
            class.push_str(" drop-field-hovering");
        }

        let drag_over: Callback<DragEvent> = ctx.link().callback(|event: DragEvent| {
            event.prevent_default();
            Msg::DragOver(event)
        });
        let drag_leave = ctx.link().callback(|event: DragEvent| Msg::DragLeave(event));

        let drop = ctx.link().callback(|event: DragEvent| {
            event.prevent_default();
            Msg::Drop(event)
        });

        html! {
                <div
                    {class}
                    ondragover={drag_over}
                    ondragleave={drag_leave}
                    ondrop={drop}
                >
                    
                </div>
        }
    }
}

impl DropReceiver {
    fn accepting(event: DragEvent) -> bool {
        let types = event.data_transfer().unwrap().types();
        let mut accepted = false;
        for i in 0..types.length() {
            if types.at(i.try_into().unwrap()) == "text/plain" {
                accepted = true;
                break;
            }
        }
        accepted
    }
}