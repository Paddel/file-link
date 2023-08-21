use wasm_bindgen::prelude::{JsValue, wasm_bindgen};
use yew::prelude::{html, function_component, Html};
use yew_router::prelude::{BrowserRouter, Switch};

mod file_tag;
mod pages;
mod web_rtc_manager;

use pages::Route;


#[function_component]
fn App() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={pages::switch} />
        </BrowserRouter>
    }
}

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    yew::Renderer::<App>::new().render();
    Ok(())
}