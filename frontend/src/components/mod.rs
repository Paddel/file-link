use yew::prelude::{html, Html};
use yew_router::prelude::Routable;

mod chat_model;
mod home;

use crate::web_rtc_manager::WebRTCManager;
use chat_model::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/send")]
    Send,
    // #[at("/receive")]
    // Receive,
    // #[not_found]
    // #[at("/404")]
    // NotFound,
}

pub fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <home::Home /> },
        Route::Send => html! { <ChatModel<WebRTCManager> /> },
    }
}