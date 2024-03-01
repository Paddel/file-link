use yew::prelude::*;

pub enum Msg {
    SessionConnect,
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub on_connect: Callback<String>,
}

pub struct Connect {
    input_code: NodeRef,
}

impl Component for Connect {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Connect {
            input_code: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SessionConnect => {
                let input = self.input_code.cast::<web_sys::HtmlInputElement>().unwrap().value();
                let code = Connect::extract_code_from_link(&input).unwrap_or(&input);
                ctx.props().on_connect.emit(code.to_string());
            }
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="container mt-5">
                <div class="row justify-content-center">
                    <div class="col-md-6">
                    <h2 class="text-center mb-4">{"Enter the initiator's link"}</h2>
                        <div class="input-group">
                            <input type="text" ref={self.input_code.clone()} class="form-control" placeholder="Enter Session Link" />
                            <div class="input-group-append">
                                <button onclick={ctx.link().callback(|_| Msg::SessionConnect)} class="btn btn-outline-secondary" type="button">{"Connect"}</button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}

impl Connect {
    fn extract_code_from_link(link: &str) -> Option<&str> {
        if link.contains("/receive/") {
            let parts: Vec<&str> = link.split('/').collect();
            return parts.last().cloned();
        }
        None
    }
}