use log::*;
use serde_derive::{Deserialize, Serialize};
use yew::prelude::*;
use yew_router::{prelude::*, Switch};
use yew_router::switch::Permissive;
use yew::virtual_dom::VNode;

use crate::app::views::{login::Login, main_view::MainView};

mod matrix;
mod views;

#[derive(Switch, Clone)]
pub enum AppRoute {
    #[to = "/"]
    Start,
    #[to = "/login"]
    Login,
    #[to = "/app"]
    MainView,
    #[to = "/page-not-found"]
    PageNotFound(Permissive<String>),
}

pub struct App {}

#[derive(Serialize, Deserialize)]
pub struct State {}

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _link: ComponentLink<Self>) -> Self {
        App {}
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> VNode {
        info!("rendered App!");
        html! {
            <div>
                <Router<AppRoute, ()>
                    render = Router::render(|switch: AppRoute| {
                        match switch {
                            AppRoute::Start => html!{<Login />},
                            AppRoute::Login => html!{<Login />},
                            AppRoute::MainView => html!{<MainView />},
                            AppRoute::PageNotFound(Permissive(None)) => html!{"Page not found"},
                            AppRoute::PageNotFound(Permissive(Some(missed_route))) => html!{format!("Page '{}' not found", missed_route)}
                        }
                    })
                    redirect = Router::redirect(|route: Route| {
                        AppRoute::PageNotFound(Permissive(Some(route.route)))
                    })
                />
            </div>
        }
    }
}
