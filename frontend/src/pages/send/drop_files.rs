use std::collections::HashMap;

use yew::prelude::*;
use web_sys::{FileList, HtmlInputElement};

use super::file_item::FileItem;
use crate::file_tag::FileTag;

pub enum Msg {
    DragOver(DragEvent),
    DragLeave(DragEvent),
    Drop(DragEvent),
    PickFile,
    FilesSelected,
}
pub struct DropFiles {
    hovering: bool,
    file_input_ref: NodeRef,
    files: HashMap<String, FileTag>,
}

impl Component for DropFiles {
    type Message = Msg;
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self {
            hovering: false,
            files: HashMap::new(),
            file_input_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
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

                let files: FileList = event.data_transfer().unwrap().files().unwrap();
                self.handle_files(files);
            }
            Msg::PickFile => {
                let input: HtmlInputElement = self.file_input_ref.cast::<HtmlInputElement>().unwrap();
                input.click();
            }
            Msg::FilesSelected => {
                if let Some(input) = self.file_input_ref.cast::<HtmlInputElement>() {
                    if let Some(files) = input.files() {
                        self.handle_files(files);
                    }
                }
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut class = "drop-field drop-field-files".to_string();
        class += if self.hovering { " drop-field-hovering" } else { "" };

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
            <>
            <span>
                <div
                    {class}
                    ondragover={drag_over}
                    ondragleave={drag_leave}
                    ondrop={drop}
                    onclick={ctx.link().callback(|_| Msg::PickFile)}
                >
                    <p>{"Drop files here"}</p>
                </div>
                <input
                    type="file"
                    ref={self.file_input_ref.clone()}
                    style="display: none"
                    multiple=true
                    onchange={ctx.link().callback(|_| Msg::FilesSelected)}
                />
                <div>
                    {
                        self.files.clone().into_iter().map(|item| {
                            let (key, tag) = item;
                            html! { 
                                    <div {key}>
                                        <FileItem tag={tag.clone()}>
                                            <div class="drop-field-file-item">
                                                <p>
                                                { format!("{} - {}", tag.name(), tag.blob().size()) }
                                                </p>
                                            </div>
                                        </FileItem>
                                    </div>
                                }
                        }).collect::<Html>()
                    }
                </div>
            </span>
            </>
        }
    }
}

impl DropFiles {
    fn handle_files(&mut self, files: FileList) {
        for i in 0..files.length() {
            if let Some(file) = files.item(i) {
                let file_name = file.name();
    
                if !self.files.contains_key(&file_name) {
                    self.files.insert(file_name.clone(), FileTag::new(file));
                    web_sys::console::log_1(&format!("File name: {}", file_name).into());
                } else {
                    web_sys::console::log_1(&format!("File name: {} already exists", file_name).into());
                }
            }
        }
    }

    fn accepting(event: DragEvent) -> bool {
        let types = event.data_transfer().unwrap().types();
        let mut accepted = false;
        for i in 0..types.length() {
            if types.at(i.try_into().unwrap()) == "Files" {
                accepted = true;
                break;
            }
        }
        accepted
    }
}