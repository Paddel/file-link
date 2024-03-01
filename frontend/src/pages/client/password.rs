use yew::prelude::*;

pub enum Msg {
    SessionConnect,
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub on_connect: Callback<String>,
}

pub struct Password {
    input_password: NodeRef,
}

impl Component for Password {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Password {
            input_password: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SessionConnect => {
                let input = self.input_password.cast::<web_sys::HtmlInputElement>().unwrap();
                let password = input.value();
                ctx.props().on_connect.emit(password);
            }
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="container mt-5">
                <div class="row justify-content-center">
                    <div class="col-md-6">
                        <h2 class="text-center mb-4">{"Wrong password"}</h2>
                        <div class="input-group">
                            <input type="password" ref={self.input_password.clone()} class="form-control" placeholder="Enter Password" />
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