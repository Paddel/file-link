use wasm_bindgen::prelude::*;
use yew::prelude::*;

mod conn;
mod rtc;
mod comp;

#[function_component]
fn App() -> Html {
    let counter = use_state(|| 0);
    let onclick = {
        let counter = counter.clone();
        move |_| {
            let value = *counter + 1;
            counter.set(value);
        }
    };

    // let onrequestapi = {
    //     move |_| {
    //     }
    // };
            


    html! {
        <div>
            <button {onclick}>{ "+1" }</button>
            <p>{ *counter }</p>
            <br/>
            // <button {onrequestapi}>{ "send" }</button>
        </div>
    }
}

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    yew::Renderer::<App>::new().render();
    // comp::test();
    // rtc::run();
    Ok(())
}