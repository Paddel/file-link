use yew::prelude::*;
use yew_router::prelude::*;

use crate::rtc::chat::web_rtc_manager::WebRTCManager;
use crate::rtc::chat::chat_model::*;
use crate::conn::Model;

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
        // Route::Home => html! { <Model /> },
        Route::Home => html! { <ChatModel<WebRTCManager> /> },
        Route::Send => html! { <p>{format!("You are looking at Post")}</p> },
        // Route::Send => html! { <ChatModel<WebRTCManager> /> },
        // Route::Post { id } => html! {<p>{format!("You are looking at Post {}", id)}</p>},
        // Route::Misc { path } => html! {<p>{format!("Matched some other path: {}", path)}</p>},
    }
}