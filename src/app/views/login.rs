use log::*;
use serde::{Deserialize, Serialize};
use yew::prelude::*;

use tr::tr;

use crate::app::matrix::{MatrixAgent, Request, Response};
use crate::errors::{Field, MatrixError};
use wasm_bindgen::__rt::core::time::Duration;
use wasm_bindgen::__rt::std::thread::sleep;

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
        let mut homeserver_classes = "uk-input";
        let mut mxid_classes = "uk-input";
        let mut password_classes = "uk-input";
        match self.state.error_field.as_ref() {
            Some(v) => match v {
                Field::Homeserver => homeserver_classes = "uk-input uk-form-danger",
                Field::MXID => mxid_classes = "uk-input uk-form-danger",
                Field::Password => password_classes = "uk-input uk-form-danger",
            },
            _ => {}
        }

        if self.state.loading {
            html! {
                <div class="container">
                    <div class="uk-position-center uk-padding">
                        <span uk-spinner="ratio: 4.5"></span>
                    </div>
                </div>
            }
        } else {
            html! {
                <div class="container">
                    <div class="uk-position-center uk-padding">
                        <h1 class="title">
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
                                        <h3 class="error">
                                            {
                                                tr!(
                                                    // {0} is the Error that happened on login
                                                    // The error message of the Login page
                                                    "Error: {0}",
                                                    v
                                                )
                                            }
                                        </h3>
                                    }
                                }
                                None => {
                                    html!{}
                                }
                            }
                        }

                        <form class="uk-form-stacked uk-margin" onsubmit=self.link.callback(|e: FocusEvent| {e.prevent_default();  Msg::Login})>
                            <div class="uk-margin">
                                <label class="uk-form-label">
                                    {
                                        tr!(
                                            // The URL Field of the Login page
                                            "Homeserver URL"
                                        )
                                    }
                                </label>
                                <div class="uk-form-controls">
                                    <input
                                        class=homeserver_classes
                                        type="url"
                                        id="homeserver"
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
                            </div>
                            <div class="uk-margin">
                                <label class="uk-form-label">
                                    {
                                        tr!(
                                            // The Matrix ID Field of the Login page
                                            "MXID"
                                        )
                                    }
                                </label>
                                <div class="uk-form-controls">
                                    <input
                                        class=mxid_classes
                                        id="username"
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
                            </div>
                            <div class="uk-margin">
                                <label class="uk-form-label">
                                    {
                                        tr!(
                                            // The Password Field of the Login page
                                            "Password"
                                        )
                                    }
                                </label>
                                <div class="uk-form-controls">
                                    <input
                                        class=password_classes
                                        type="password"
                                        id="password"
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
                            </div>

                            <div class="uk-margin">
                                <div class="uk-form-controls">
                                    <button class="uk-button uk-button-primary">
                                    {
                                        tr!(
                                            // The Login Button of the Login page
                                            "Login"
                                        )
                                    }
                                    </button>
                                </div>
                            </div>
                        </form>
                    </div>
                </div>
            }
        }
    }
}
