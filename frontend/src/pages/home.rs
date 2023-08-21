use yew::prelude::{html, Callback, function_component, Html};
use yew_router::prelude::use_navigator;

use super::Route;

#[function_component]
pub fn Home() -> Html {
    let navigator = use_navigator().unwrap();
    let nav_clone = navigator.clone();
    html! {
        <div>
            <button onclick={Callback::from(move |_| navigator.push(&Route::Send))}>{ "send" }</button>
            <button onclick={Callback::from(move |_| nav_clone.push(&Route::Receive))}>{ "receive" }</button>
        </div>
    }
}