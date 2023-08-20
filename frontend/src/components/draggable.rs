// use stylist::yew::use_style;
use yew::prelude::*;


#[derive(PartialEq, Properties)]
pub struct IndicatorProps {
    pub dragging: bool,
    pub children: Children,
}

#[function_component]
pub fn Indicator(props: &IndicatorProps) -> Html {
    html! {
        <div class="drag-indicator">
            <p>{props.dragging}</p>
            { props.children.clone() }
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct DraggableProps {
    pub drag: Callback<bool>,
}

pub struct Draggable {
}

pub enum DraggableMsg {
    DragStart(DragEvent),
    DragEnd,
}

impl Component for Draggable {
    type Message = DraggableMsg;
    type Properties = DraggableProps;

    fn create(_: &Context<Self>) -> Self {
        Self {
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            DraggableMsg::DragStart(event) => {
                ctx.props().drag.emit(true);

                event.data_transfer()
                    .unwrap()
                    .set_data("text", "test")
                    .unwrap();
            }
            DraggableMsg::DragEnd => {
                ctx.props().drag.emit(false);
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {

        html! {
            <div
                class="draggable"
                draggable="true"
                ondragstart={ctx.link().callback(DraggableMsg::DragStart)}
                ondragend={ctx.link().callback(|_| DraggableMsg::DragEnd)}
            >
                { "Drag me!"}
            </div>
        }
    }
}