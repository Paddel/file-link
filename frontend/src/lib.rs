use wasm_bindgen::prelude::{JsValue, wasm_bindgen};
use yew::prelude::{html, function_component, Callback, Html};
use yew_router::prelude::{BrowserRouter, Switch, use_navigator};

use pages::Route;

mod file_tag;
mod pages;
mod services;
mod wrtc_protocol;
mod constants;

pub mod shared {
    use serde::{Deserialize, Serialize};
    use toml;
    include!("../../shared/api_protocol.rs");
    include!("../../shared/config.rs");
}

#[function_component(Header)]
pub fn header() -> Html {
    let navigator = use_navigator().unwrap();

    html! {
        <div class="header">
            <img
            src="/public/assets/logo.svg"
            class="logo"
            alt="Logo"
            onclick={Callback::from(move |_| navigator.push(&Route::Home))}
            />
        </div>
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Header />
            <div class="container p-3">
                <Switch<Route> render={pages::switch} />
            </div>
        </BrowserRouter>
    }
}

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    yew::Renderer::<App>::new().render();
    Ok(())
}