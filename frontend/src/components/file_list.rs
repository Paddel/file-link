use yew::prelude::*;

use crate::file_tag::FileTag;


#[derive(Clone, Debug)]
pub enum FileState {
    Pending,
    Queued,
    Sending,
    Sent,
    Failed,
}

#[derive(Clone)]
pub struct FileListItem {
    pub tag: FileTag,
    pub progress: f64,
    pub state: FileState,
}

impl FileListItem {
    fn new(tag: FileTag) -> Self {
        Self {
            tag,
            progress: 0.0,
            state: FileState::Pending,
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct FileListProps {
    pub children: Children,
}

#[function_component]
pub fn FileList(props: &FileListProps) -> Html {
    html! {
        <table class="table">
            // <thead>
            //     <tr>
            //         <th scope="col">{"#"}</th>
            //         <th scope="col">{"Name"}</th>
            //         <th scope="col">{"Size"}</th>
            //         <th scope="col">{"State"}</th>
            //         <th scope="col">{""}</th>
            //     </tr>
            // </thead>
            <tbody>
                <tr>
                    // <th scope="row">{index}</th>
                    // <td>{file.tag.name()}</td>
                    // <td>{file.tag.blob().size()}</td>
                    // <td>{format!("{:?}", file.state)}</td>
                    // <td>{"Waiting for Receiver"}</td>
                    { props.children.clone() }
                </tr>
            </tbody>
        </table>
    }
}