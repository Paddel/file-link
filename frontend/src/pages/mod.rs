use yew::prelude::{html, Html};
use yew_router::prelude::Routable;

mod home;
pub mod client;
pub mod host;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/receive/:code")]
    ReceiveCode { code: String },
    #[at("/receive")]
    Receive,
    #[at("/send")]
    Send,
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <home::Home /> },
        Route::Receive => html! { <client::Client /> },
        Route::ReceiveCode { code } => html! { <client::Client {code} /> },
        Route::Send => html! { <host::Host /> },
        Route::NotFound => html! { <home::Home /> },
    }
}