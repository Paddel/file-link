use std::rc::Rc;

use yew::prelude::*;

use crate::file_tag::FileTag;
use super::PageState;

pub enum IndicatorMsg {
    ContextChanged(Rc<PageState>),
}

#[derive(PartialEq, Properties)]
pub struct IndicatorProps {
    pub children: Children,
}

pub struct Indicator {
    state: Rc<PageState>,
    _listener: ContextHandle<Rc<PageState>>,
}

impl Component for Indicator {
    type Message = IndicatorMsg;
    type Properties = IndicatorProps;

    fn create(ctx: &Context<Self>) -> Self {
        let (state, _listener) = ctx
            .link()
            .context::<Rc<PageState>>(ctx.link().callback(IndicatorMsg::ContextChanged))
            .expect("context to be set");

        Self { state, _listener }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            IndicatorMsg::ContextChanged(state) => {
                self.state = state;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut class = "drag-indicator".to_string();
        class += if self.state.is_dragging { " glow" } else { "" };

        html! {
            <div {class}>
                { ctx.props().children.clone() }
            </div>
        }
    }
}

#[derive(PartialEq, Properties)]
pub struct FileItemProps {
    pub children: Children,
    pub tag: FileTag,
}

pub struct FileItem {
    state: Rc<PageState>,
    _listener: ContextHandle<Rc<PageState>>,
}

pub enum FileItemMsg {
    ContextChanged(Rc<PageState>),
    DragStart(DragEvent),
    DragEnd,
}

impl Component for FileItem {
    type Message = FileItemMsg;
    type Properties = FileItemProps;

    fn create(ctx: &Context<Self>) -> Self {
        let (state, _listener) = ctx
            .link()
            .context::<Rc<PageState>>(ctx.link().callback(FileItemMsg::ContextChanged))
            .expect("context to be set");

        Self { state, _listener }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            FileItemMsg::ContextChanged(state) => {
                self.state = state;
            }
            FileItemMsg::DragStart(event) => {
                self.state.on_drag.emit(true);
                event.data_transfer()
                    .unwrap()
                    .set_data("text", &ctx.props().tag.uuid().to_string())
                    .unwrap();
            }
            FileItemMsg::DragEnd => {
                self.state.on_drag.emit(false);
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {

        html! {
            <div
                class="draggable"
                draggable="true"
                ondragstart={ctx.link().callback(FileItemMsg::DragStart)}
                ondragend={ctx.link().callback(|_| FileItemMsg::DragEnd)}
            >
                { ctx.props().children.clone() }
            </div>
        }
    }
}