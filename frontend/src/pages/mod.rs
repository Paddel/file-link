use yew::prelude::{html, Html};
use yew_router::prelude::Routable;

use crate::services::web_rtc::WebRTCManager;

mod home;
pub mod receive;
pub mod send;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/receive")]
    Receive,
    #[at("/send")]
    Send,
}

pub fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <home::Home /> },
        Route::Receive => html! { <receive::Receive /> },
        Route::Send => html! { <send::Send<WebRTCManager> /> },
    }
}