use std::include_str;
use std::thread::sleep;
use std::time::Duration;

use log::*;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

use tr::tr;

use crate::app::components::raw_html::RawHTML;
use crate::app::matrix::{MatrixAgent, Request, Response};
use crate::errors::{Field, MatrixError};

pub struct Login {
    link: ComponentLink<Self>,
    state: State,
    matrix_agent: Box<dyn Bridge<MatrixAgent>>,
}

pub enum Msg {
    NewMessage(Response),
    SetHomeserver(String),
    SetUsername(String),
    SetPassword(String),
    Login,
}

#[derive(Serialize, Deserialize, Default)]
pub struct State {
    loading: bool,
    homeserver: String,
    username: String,
    password: String,
    error: Option<String>,
    error_field: Option<Field>,
    retries: u64,
}

impl Component for Login {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let matrix_callback = link.callback(Msg::NewMessage);
        let matrix_agent = MatrixAgent::bridge(matrix_callback);
        let state = State {
            loading: false,
            homeserver: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            error: None,
            error_field: None,
            retries: 0,
        };
        Login {
            link,
            state,
            matrix_agent,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetHomeserver(homeserver) => {
                self.state.homeserver = homeserver.clone();
                self.matrix_agent.send(Request::SetHomeserver(homeserver));
                true
            }
            Msg::SetUsername(username) => {
                self.state.username = username.clone();
                self.matrix_agent.send(Request::SetUsername(username));
                true
            }
            Msg::SetPassword(password) => {
                self.state.password = password.clone();
                self.matrix_agent.send(Request::SetPassword(password));
                true
            }
            Msg::Login => {
                // Reset Errors
                self.state.error = None;
                self.state.error_field = None;

                // Start loading
                self.matrix_agent.send(Request::Login);
                self.state.loading = true;
                true
            }
            Msg::NewMessage(response) => {
                match response {
                    Response::Error(error) => {
                        match error.clone() {
                            MatrixError::MissingFields(field) => {
                                self.state.loading = false;
                                self.state.error = Some(error.to_string());
                                self.state.error_field = Some(field);
                                true
                            }
                            MatrixError::LoginTimeout => {
                                // If we had less than 10 tries try again
                                if self.state.retries < 10 {
                                    self.state.retries += 1;
                                    info!("Trying login again in 5 seconds");
                                    sleep(Duration::from_secs(5));
                                    self.link.send_message(Msg::Login);
                                    false
                                } else {
                                    self.state.loading = false;
                                    self.state.error =
                                        Some("Login failed after 10 tries.".to_string());
                                    true
                                }
                            }
                            MatrixError::SDKError(e) => {
                                // TODO handle login error != timeout better
                                error!("SDK Error: {}", e);
                                false
                            }
                            _ => false,
                        }
                    }
                    _ => false,
                }
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    //noinspection RsTypeCheck
    fn view(&self) -> Html {
        let mut homeserver_classes = "login-input";
        let mut mxid_classes = "login-input";
        let mut password_classes = "login-input";
        if let Some(v) = self.state.error_field.as_ref() {
            match v {
                Field::Homeserver => homeserver_classes = "login-input uk-form-danger",
                Field::MXID => mxid_classes = "login-input uk-form-danger",
                Field::Password => password_classes = "login-input uk-form-danger",
            }
        }

        if self.state.loading {
            html! {
                <div class="container">
                    <div class="uk-position-center uk-padding sun-animation">
                        <RawHTML inner_html=include_str!("../svgs/loading_animation.svg")/>
                    </div>
                </div>
            }
        } else {
            html! {
                <>
                    <div class="login-page-bg"></div>
                    <div class="scrollable" style="width:100vw; padding:2px;">
                        <div>
                            <div class="login-bg">
                                <div class="daydream-title"><RawHTML inner_html=include_str!("../svgs/DaydreamLogo_v0_light.svg")/></div>
                                <div class="login-content">
                                    <h1 class="login-title">
                                        {
                                            tr!(
                                                // The Login Button of the Login page
                                                "Login"
                                            )
                                        }
                                    </h1>
                                    {
                                        match &self.state.error {
                                            Some(v) => {
                                                html! {
                                                    <h4 class="error">
                                                        {
                                                            tr!(
                                                                // {0} is the Error that happened on login
                                                                // The error message of the Login page
                                                                "Error: {0}",
                                                                v
                                                            )
                                                        }
                                                    </h4>
                                                }
                                            }
                                            None => {
                                                html!{}
                                            }
                                        }
                                    }

                                    <form id="login_form" onsubmit=self.link.callback(|e: FocusEvent| {e.prevent_default();  Msg::Login})>
                                        <div class="login-inline login-input-first">
                                           <span class="material-icons login-icons" id="ma-icon" style="font-size: 28px !important;">{"http"}</span>
                                            <input
                                                pattern="^https:\\/\\/.*$"
                                                required=true
                                                class=homeserver_classes
                                                type="url"
                                                id="homeserver"
                                                name="homeserver"
                                                placeholder=
                                                {
                                                    tr!(
                                                        // The URL Field of the Login page
                                                        "Homeserver URL"
                                                    )
                                                }
                                                value=&self.state.homeserver
                                                oninput=self.link.callback(|e: InputData| Msg::SetHomeserver(e.value))
                                            />
                                        </div>
                                        <div class="login-inline">
                                           <span class="material-icons login-icons" id="ma-icon">{"person"}</span>
                                            <input
                                                required=true
                                                pattern="^(@[\x21-\x39\x3B-\x7E]+:.*|[\x21-\x39\x3B-\x7E]+)$"
                                                class=mxid_classes
                                                id="username"
                                                name="username"
                                                placeholder=
                                                {
                                                    tr!(
                                                        // The Matrix ID Field of the Login page
                                                        "MXID"
                                                    )
                                                }
                                                value=&self.state.username
                                                oninput=self.link.callback(|e: InputData| Msg::SetUsername(e.value))
                                            />
                                        </div>
                                        <div class="login-inline">
                                           <span class="material-icons login-icons" id="ma-icon">{"vpn_key"}</span>
                                            <input
                                                required=true
                                                class=password_classes
                                                type="password"
                                                id="password"
                                                name="password"
                                                placeholder=
                                                {
                                                    tr!(
                                                        // The Password Field of the Login page
                                                        "Password"
                                                    )
                                                }
                                                value=&self.state.password
                                                oninput=self.link.callback(|e: InputData| Msg::SetPassword(e.value))
                                            />
                                        </div>

                                        <button class="login-button">
                                            {
                                                tr!(
                                                    // The Login Button of the Login page
                                                    "Login"
                                                )
                                            }
                                        </button>
                                    </form>
                                </div>
                            </div>
                        </div>
                    </div>
                </>
            }
        }
    }
}
