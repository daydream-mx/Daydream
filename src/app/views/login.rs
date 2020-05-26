use log::*;
use yew::agent::{Dispatched, Dispatcher};
use yew::prelude::*;
use yew::services::storage::{Area, StorageService};

use crate::app::matrix::{MatrixAgent, Request};

pub struct Login {
    link: ComponentLink<Self>,
    homeserver: String,
    username: String,
    password: String,
    storage: StorageService,
    matrix_agent: Dispatcher<MatrixAgent>,
}

pub enum Msg {
    SetHomeserver(String),
    SetUsername(String),
    SetPassword(String),
    Login,
    Nope,
}


impl Component for Login {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local).unwrap();
        let matrix_agent = MatrixAgent::dispatcher();
        Login {
            link,
            //TODO use state
            homeserver: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            storage,
            matrix_agent,
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
                false
            }
            Msg::Nope => false,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        info!("rendered Login!");
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
                        />
                <button onclick=self.link.callback(|_: MouseEvent| Msg::Login)
                       onkeypress=self.link.callback(|e: KeyboardEvent| {
                           if e.key() == "Enter" { Msg::Login } else { Msg::Nope }
                       })>
                    { "Login" }
                </button>
            </div>
        }
    }
}
