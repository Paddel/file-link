use std::convert::TryInto;

use yew::prelude::*;
use web_sys::{FileList, HtmlInputElement, File};

pub enum Msg {
    DragOver(DragEvent),
    DragLeave(DragEvent),
    Drop(DragEvent),
    PickFile,
    FilesSelected,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub onupdate: Callback<Vec<File>>,
}

pub struct DropFiles {
    hovering: bool,
    file_input_ref: NodeRef,
}

impl Component for DropFiles {
    type Message = Msg;
    type Properties = Props;

    fn create(_: &Context<Self>) -> Self {
        Self {
            hovering: false,
            file_input_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DragOver(_) | Msg::DragLeave(_) => {
                if !Self::accepting(&msg) { return false; }
                self.hovering = matches!(msg, Msg::DragOver(_));
                return true;
            }
            Msg::Drop(_) => {
                if !Self::accepting(&msg) { return false; }
                self.hovering = false;
                
                if let Msg::Drop(event) = &msg {
                    let file_list: FileList = event.data_transfer().unwrap().files().unwrap();
                    self.handle_files(ctx, file_list);
                }
                return true;
            }
            Msg::PickFile => {
                if let Some(input) = self.file_input_ref.cast::<HtmlInputElement>() {
                    input.click();
                }
            }
            Msg::FilesSelected => {
                if let Some(input) = self.file_input_ref.cast::<HtmlInputElement>() {
                    if let Some(file_list) = input.files() {
                        self.handle_files(ctx, file_list);
                    }
                }
            }
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let class = format!(
            "drop-field d-flex drop-field-files flex-column justify-content-center{}",
            if self.hovering { " drop-field-hovering" } else { "" }
        );

        html! {
            <span>
                <div
                    class={class}
                    ondragover={ctx.link().callback(|event: DragEvent| {
                        event.prevent_default();
                        Msg::DragOver(event)
                    })}
                    ondragleave={ctx.link().callback(Msg::DragLeave)}
                    ondrop={ctx.link().callback(|event: DragEvent| {
                        event.prevent_default();
                        Msg::Drop(event)
                    })}
                    onclick={ctx.link().callback(|_| Msg::PickFile)}
                    >
                    <span class="font-weight-bold text-secondary">{"Click to select or simply drop files here."}</span>
                </div>
                <input
                    type="file"
                    ref={self.file_input_ref.clone()}
                    style="display: none"
                    multiple=true
                    onchange={ctx.link().callback(|_| Msg::FilesSelected)}
                />
            </span>
        }
    }
}

impl DropFiles {
    fn handle_files(&self, ctx: &Context<Self>, file_list: FileList) {
        let files: Vec<File> = (0..file_list.length())
            .filter_map(|i| file_list.item(i))
            .collect();
        ctx.props().onupdate.emit(files);
    }
    fn accepting(msg: &Msg) -> bool {
        if let Msg::DragOver(event) | Msg::DragLeave(event) | Msg::Drop(event) = msg {
            let types = event.data_transfer().map(|dt| dt.types());
            types.map_or(false, |types| (0..types.length()).any(|i| types.at(i.try_into().unwrap_or(0)) == "Files"))
        } else {
            false
        }
    }    
}