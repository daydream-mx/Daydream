use log::*;
use serde_derive::{Deserialize, Serialize};
use yew::agent::{Dispatched, Dispatcher};
use yew::prelude::*;
use yew::services::storage::{Area, StorageService};
use yew_router::{route::Route};

use crate::app::matrix::{MatrixAgent, Request, Response};
use yew_router::agent::RouteRequest;
use yew_router::prelude::*;

pub struct Login {
    link: ComponentLink<Self>,
    router: Box<dyn Bridge<RouteAgent>>,
    homeserver: String,
    username: String,
    password: String,
    storage: StorageService,
    matrix_agent: Dispatcher<MatrixAgent>,
    _producer: Box<dyn Bridge<MatrixAgent>>,
}

pub enum Msg {
    SetHomeserver(String),
    SetUsername(String),
    SetPassword(String),
    Login,
    Navigate(String),
    NewMessage(Response),
    Nope,
}

impl Component for Login {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|_| Msg::Nope); // TODO use a dispatcher instead.
        let router = RouteAgent::bridge(callback);

        let storage = StorageService::new(Area::Local).unwrap();
        let matrix_agent = MatrixAgent::dispatcher();
        let matrix_callback = link.callback(Msg::NewMessage);
        let _producer = MatrixAgent::bridge(matrix_callback);
        Login {
            link,
            router,
            homeserver: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            storage,
            matrix_agent,
            _producer,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetHomeserver(homeserver) => {
                self.homeserver = homeserver.clone();
                self.matrix_agent
                    .send(Request::SetHomeserver(homeserver.clone()));
                true
            }
            Msg::SetUsername(username) => {
                self.username = username.clone();
                self.matrix_agent
                    .send(Request::SetUsername(username.clone()));
                true
            }
            Msg::SetPassword(password) => {
                self.password = password.clone();
                self.matrix_agent
                    .send(Request::SetPassword(password.clone()));
                true
            }
            Msg::Login => {
                info!("Start Login");
                self.matrix_agent.send(Request::Login());
                true
            }
            Msg::Navigate(route_string) => {
                let route = Route::from(route_string);

                self.router.send(RouteRequest::ChangeRoute(route));
                true
            }
            Msg::Nope => false,
            Msg::NewMessage(response) => {
                info!("NewMessage: {:#?}", response);
                if response.message == "login_logged_in" {
                    info!("Finished Login");
                    self.link
                        .callback(|_: InputData| Msg::Navigate("/app".to_owned()));
                }
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        info!("rendered App!");
        html! {
            <div>
                <input class="server"
                   placeholder="Homeserver URL"
                   value=&self.homeserver
                   oninput=self.link.callback(|e: InputData| Msg::SetHomeserver(e.value))
                   //onkeypress=self.link.callback(|e: KeyboardEvent| {
                   //    if e.key() == "Enter" { Msg::Add } else { Msg::Nope }
                   //})
                    />
                <input class="username"
                       placeholder="MXID/Username"
                       value=&self.username
                       oninput=self.link.callback(|e: InputData| Msg::SetUsername(e.value))
                       //onkeypress=self.link.callback(|e: KeyboardEvent| {
                       //    if e.key() == "Enter" { Msg::Add } else { Msg::Nope }
                       //})
                        />
                <input class="password"
                       placeholder="Password"
                       type="password"
                       value=&self.password
                       oninput=self.link.callback(|e: InputData| Msg::SetPassword(e.value))
                       //onkeypress=self.link.callback(|e: KeyboardEvent| {
                       //    if e.key() == "Enter" { Msg::Add } else { Msg::Nope }
                       //})
                        />
                <button onclick=self.link.callback(|_: MouseEvent| Msg::Login)>
                    { "Login" }
                </button>
            </div>
        }
    }
}
