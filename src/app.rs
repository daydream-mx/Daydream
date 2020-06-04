use yew::{prelude::*, virtual_dom::VNode};
use yew_router::agent::RouteRequest::ChangeRoute;
use yew_router::{prelude::*, Switch};

use crate::app::matrix::{MatrixAgent, Response};
use crate::app::views::{login::Login, main_view::MainView};

pub mod components;
mod matrix;
mod views;

#[derive(Switch, Clone)]
pub enum AppRoute {
    #[to = "/login"]
    Login,
    #[to = "/"]
    MainView,
}
pub enum Msg {
    RouteChanged(Route<()>),
    ChangeRoute(AppRoute),
    NewMessage(Response),
}
pub struct App {
    matrix_agent: Box<dyn Bridge<MatrixAgent>>,
    link: ComponentLink<Self>,
    route: Option<Route<()>>,
    route_agent: Box<dyn Bridge<RouteAgent<()>>>,
}

impl App {
    fn change_route(&self, app_route: AppRoute) -> Callback<MouseEvent> {
        self.link.callback(move |_| {
            let route = app_route.clone();
            Msg::ChangeRoute(route)
        })
    }
}
impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let route_agent = RouteAgent::bridge(link.callback(Msg::RouteChanged));
        let mut matrix_agent = MatrixAgent::bridge(link.callback(Msg::NewMessage));
        matrix_agent.send(matrix::Request::GetLoggedIn);
        App {
            matrix_agent,
            route_agent,
            route: None,
            link,
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
                self.route_agent.send(ChangeRoute(route));
            }
            Msg::NewMessage(response) => {
                //info!("NewMessage: {:#?}", response);
                match response {
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
