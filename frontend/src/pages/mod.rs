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
    Client,
    #[at("/send")]
    Host,
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <home::Home /> },
        Route::Client => html! { <client::Client /> },
        Route::ReceiveCode { code } => html! { <client::Client {code} /> },
        Route::Host => html! { <host::Host /> },
        Route::NotFound => html! { <home::Home /> },
    }
}