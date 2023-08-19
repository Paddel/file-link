use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_router::prelude::*;

mod conn;
mod rtc;
mod components;

use components::Route;


#[function_component]
fn App() -> Html {
    // let counter = use_state(|| 0);
    // let onclick = {
    //     let counter = counter.clone();
    //     move |_| {
    //         let value = *counter + 1;
    //         counter.set(value);
    //     }
    // };

    // let onrequestapi = {
    //     move |_| {
    //     }
    // };

    // let navigator = use_navigator().unwrap();
    // let onclick = Callback::from(move |_| navigator.push(&Route::Send));
            


    // html! {
    //     <div>
    //         <button {onclick}>{ "+1" }</button>
    //         <p>{ *counter }</p>
    //         <br/>
    //         // <button {onrequestapi}>{ "send" }</button>
    //     </div>
    // }
    html! {
        // <>
        // <button {onclick}>{{"Send"}}</button>
        <BrowserRouter>
            <Switch<Route> render={components::switch} />
        </BrowserRouter>
        // </>
    }
}

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    yew::Renderer::<App>::new().render();
    Ok(())
}