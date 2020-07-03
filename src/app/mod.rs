use yew::{prelude::*, virtual_dom::VNode};
use yew_router::agent::RouteRequest::ChangeRoute;
use yew_router::{prelude::*, Switch};

use crate::app::matrix::{MatrixAgent, Response, SessionStore};
use crate::app::views::{login::Login, main_view::MainView};
use crate::constants::AUTH_KEY;
use log::*;
use std::sync::{Arc, Mutex};
use yew::format::Json;
use yew::services::storage::Area;
use yew::services::StorageService;

pub mod components;
pub mod matrix;
mod views;

#[derive(Switch, Clone)]
pub enum AppRoute {
    #[to = "/login"]
    Login,
    #[to = "/"]
    MainView,
}

#[allow(clippy::large_enum_variant)]
pub enum Msg {
    RouteChanged(Route<()>),
    ChangeRoute(AppRoute),
    NewMessage(Response),
}

pub struct App {
    // While unused this needs to stay :(
    _matrix_agent: Box<dyn Bridge<MatrixAgent>>,
    route: Option<Route<()>>,
    route_agent: Box<dyn Bridge<RouteAgent<()>>>,
    storage: Arc<Mutex<StorageService>>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = Arc::new(Mutex::new(
            StorageService::new(Area::Local).expect("storage was disabled by the user"),
        ));

        let session: Option<SessionStore> = {
            if let Json(Ok(restored_model)) = storage.lock().unwrap().restore(AUTH_KEY) {
                Some(restored_model)
            } else {
                None
            }
        };
        let route_agent = RouteAgent::bridge(link.callback(Msg::RouteChanged));
        let mut matrix_agent = MatrixAgent::bridge(link.callback(Msg::NewMessage));
        matrix_agent.send(matrix::Request::SetSession(session.unwrap()));
        matrix_agent.send(matrix::Request::GetLoggedIn);
        App {
            _matrix_agent: matrix_agent,
            route_agent,
            route: None,
            storage,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RouteChanged(route) => {
                self.route = Some(route);
            }
            Msg::ChangeRoute(route) => {
                let route: Route = route.into();
                self.route = Some(route.clone());
                info!("{:?}", self.route);
                self.route_agent.send(ChangeRoute(route));
            }
            Msg::NewMessage(response) => {
                //info!("NewMessage: {:#?}", response);
                match response {
                    Response::SaveSession(session) => {
                        let mut storage = self.storage.lock().unwrap();
                        storage.store(AUTH_KEY, Json(&session));
                    }
                    Response::Error(_) => {}
                    Response::LoggedIn(logged_in) => {
                        let route: Route = if logged_in {
                            //self.state.logged_in = true;

                            // replace with sync routeagent message once its possible
                            // https://github.com/yewstack/yew/issues/1127
                            //RouteService::new().get_route();
                            AppRoute::MainView.into()
                        } else {
                            AppRoute::Login.into()
                        };

                        self.route = Some(route.clone());
                        self.route_agent.send(ChangeRoute(route));
                    }
                    _ => {}
                }
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            {
                match &self.route {
                    None => html! {<Login />},
                    Some(route) => match AppRoute::switch(route.clone()) {
                        Some(AppRoute::MainView) => {
                            html! {
                                <MainView />
                            }
                        },
                        Some(AppRoute::Login) => {
                            html! {
                                <Login />
                            }
                        },
                        None => VNode::from("404")
                    }
                }
            }
        }
    }
}
