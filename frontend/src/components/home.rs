use yew::prelude::{html, Callback, function_component, Html};
use yew_router::prelude::use_navigator;

use super::Route;

#[function_component]
pub fn Home() -> Html {
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

    let navigator = use_navigator().unwrap();
    let onclick = Callback::from(move |_| navigator.push(&Route::Send));

    html! {
        <div>
            // <button {onclick}>{ "+1" }</button>
            // <p>{ *counter }</p>
            // <br/>
            <button {onclick}>{ "send" }</button>
        </div>
    }
}