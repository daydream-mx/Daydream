use log::*;
use yew::agent::{Dispatched, Dispatcher};
use yew::prelude::*;

use crate::app::matrix::{MatrixAgent, Request};

pub struct Login {
    link: ComponentLink<Self>,
    homeserver: String,
    username: String,
    password: String,
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
        let matrix_agent = MatrixAgent::dispatcher();
        Login {
            link,
            //TODO use state
            homeserver: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
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
            <div class="container h-100">
                <div class="row align-items-center h-100">
                    <h1>{"Login"}</h1>
                    <form class="col-6 mx-auto" onsubmit=self.link.callback(|e: Event| {e.prevent_default();  Msg::Login})>
                        <div class="form-group">
                            <label for="homeserver">{"Homeserver URL"}</label>
                            <input
                                class="form-control"
                                type="url"
                                id="homeserver"
                                placeholder="Homeserver URL"
                                value=&self.homeserver
                                oninput=self.link.callback(|e: InputData| Msg::SetHomeserver(e.value))
                            />
                        </div>
                        <div class="form-group">
                            <label for="username">{"MXID/Username"}</label>
                            <input
                                class="form-control"
                                id="username"
                                placeholder="MXID/Username"
                                value=&self.username
                                oninput=self.link.callback(|e: InputData| Msg::SetUsername(e.value))
                            />
                        </div>
                        <div class="form-group">
                            <label for="password">{"Password"}</label>
                            <input
                                class="form-control"
                                type="password"
                                id="password"
                                placeholder="Password"
                                value=&self.password
                                oninput=self.link.callback(|e: InputData| Msg::SetPassword(e.value))
                            />
                        </div>

                        <button type="submit" class="btn btn-primary">
                            { "Login" }
                        </button>
                    </form>
                </div>
            </div>
        }
    }
}
