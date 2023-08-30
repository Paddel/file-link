use yew::prelude::{html, Callback, function_component, Html};
use yew_router::prelude::use_navigator;

use super::Route;

#[function_component]
pub fn Home() -> Html {
    let navigator = use_navigator().unwrap();
    let nav_clone = navigator.clone();
    html! {
        <div class="container mt-5">
            <h2 class="text-center mb-4">{"Choose Your Action!"}</h2>

            <div class="d-flex justify-content-center">
                <div class="m-2">
                    <button class="btn btn-light btn-block square-button bouncing-trigger" onclick={Callback::from(move |_| navigator.push(&Route::Host))}>
                        <div class="svg-container">
                            <img src="/public/static/icons/base.svg" class="svg-base"/>
                            <img src="/public/static/icons/upload.svg" class="svg-overlay bouncing"/>
                        </div>
                        <hr class="w-50 mx-auto my-2" />
                        {"Send"}
                    </button>
                </div>
                <div class="m-2">
                    <button class="btn btn-light btn-block square-button bouncing-trigger" onclick={Callback::from(move |_| nav_clone.push(&Route::Client))}>
                        <div class="svg-container">
                            <img src="/public/static/icons/base.svg" class="svg-base"/>
                            <img src="/public/static/icons/download.svg" class="svg-overlay bouncing"/>
                        </div>
                        <hr class="w-50 mx-auto my-2" />
                        {"Receive"}
                    </button>
                </div>
            </div>
        </div>
    }
}