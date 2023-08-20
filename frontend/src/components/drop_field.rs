use std::collections::HashMap;

use yew::prelude::*;
use web_sys::{console, FileList, File, HtmlInputElement};

#[derive(Properties, PartialEq)]
pub struct Props {
    // pub drop: Callback<bool>,
    pub accepting: Callback<DragEvent, bool>,
}

pub enum Msg {
    DragOver(DragEvent),
    DragLeave(DragEvent),
    Drop(DragEvent),
    PickFile,
    FilesSelected,
}
pub struct DropField {
    hovering: bool,
    files: HashMap<String, File>,
    file_input_ref: NodeRef,
}

impl Component for DropField {
    type Message = Msg;
    type Properties = Props;

    fn create(_: &Context<Self>) -> Self {
        Self {
            hovering: false,
            files: HashMap::new(),
            file_input_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DragOver(event) => {
                if ctx.props().accepting.emit(event) {return false;}
                self.hovering = true;
            }
            Msg::DragLeave(event) => {
                if ctx.props().accepting.emit(event) {return false;}
                self.hovering = false;
            }
            Msg::Drop(event) => {
                if ctx.props().accepting.emit(event.clone()) {return false;}
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
            <span>
                <div
                    class={if self.hovering { "drop-field hovering" } else { "drop-field" }}
                    ondragover={drag_over}
                    ondragleave={drag_leave}
                    ondrop={drop}
                    onclick={ctx.link().callback(|_| Msg::PickFile)}
                >
                    { "Drop here " }{self.files.len()}
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

impl DropField {
    fn handle_files(&mut self, files: FileList) {
        for i in 0..files.length() {
            if let Some(file) = files.item(i) {
                let file_name = file.name();
    
                if !self.files.contains_key(&file_name) {
                    self.files.insert(file_name.clone(), file);
                    web_sys::console::log_1(&format!("File name: {}", file_name).into());
                } else {
                    web_sys::console::log_1(&format!("File name: {} already exists", file_name).into());
                }
            }
        }
    }
}