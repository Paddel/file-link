use yew::prelude::*;

use crate::file_tag::FileTag;
use crate::components::file_list::{FileList, FileListItem};

pub enum Msg {
    AddFile(FileListItem),
}
pub struct Receive {
    
}

impl Component for Receive {
    type Message = Msg;
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self {
        }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AddFile(file) => {
                // console::log_1(&format!("added file: {}", file.tag.name()).into());
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <FileList>
                <p>{"Hello"}</p>
            </FileList>
        }
    }
}