use log::*;
use matrix_sdk::{Client, ClientConfig};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashSet;
use url::Url;
use wasm_bindgen_futures::spawn_local;
use yew::worker::*;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct MatrixClient {
    pub(crate) homeserver: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
}

#[derive(Clone)]
pub struct MatrixAgent {
    link: AgentLink<MatrixAgent>,
    matrix_state: MatrixClient,
    matrix_client: Option<Client>,
    subscribers: HashSet<HandlerId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    SetHomeserver(String),
    SetUsername(String),
    SetPassword(String),
    Login(),
    GetLoggedIn,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response {
    pub message: String,
    pub content: String,
}

impl Agent for MatrixAgent {
    type Reach = Context;
    type Message = ();
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        MatrixAgent {
            link,
            matrix_state: Default::default(),
            matrix_client: None,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, _: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, _: HandlerId) {
        match msg {
            Request::SetHomeserver(homeserver) => {
                self.matrix_state.homeserver = Some(homeserver.clone());
            }
            Request::SetUsername(username) => {
                self.matrix_state.username = Some(username.clone());
            }
            Request::SetPassword(password) => {
                self.matrix_state.password = Some(password.clone());
            }
            Request::Login() => {
                let login_client = self.login();
                if login_client.is_none() {
                    let resp = Response {
                        message: "login_missing_client".to_string(),
                        content: "".to_string(),
                    };
                    for sub in self.subscribers.iter() {
                        self.link.respond(*sub, resp.clone());
                    }
                    return;
                }
                let client = login_client.clone().unwrap();
                let username = self.matrix_state.username.clone().unwrap();
                let password = self.matrix_state.password.clone().unwrap();
                let username = username.clone();
                let password = password.clone();
                let client = client.clone();
                let subscribers = self.subscribers.clone();
                let agent = self.clone();
                spawn_local(async move {
                    client
                        .login(
                            username.clone(),
                            password.clone(),
                            None,
                            Some("Daydream".to_string()),
                        )
                        .await;
                    info!("did login");
                    let resp = Response {
                        message: "login_logged_in".to_string(),
                        content: "".to_string(),
                    };
                    info!("prepared login response");
                    for sub in subscribers.iter() {
                        agent.link.respond(*sub, resp.clone());
                    }
                    info!("sent login response");
                });
            }
            Request::GetLoggedIn => {
                let subscribers = self.subscribers.clone();
                let agent = self.clone();
                spawn_local(async move {
                    let logged_in = agent.get_logged_in().await;
                    let resp = Response {
                        message: "client_logged_in".to_string(),
                        content: logged_in,
                    };
                    for sub in subscribers.iter() {
                        agent.link.respond(*sub, resp.clone());
                    }
                });
            }
        }
    }
    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}

impl MatrixAgent {
    async fn get_logged_in(&self) -> String {
        self.matrix_client
            .clone()
            .unwrap()
            .logged_in()
            .await
            .to_string()
    }

    fn login(&mut self) -> Option<Client> {
        return if self.matrix_state.homeserver.is_none()
            || self.matrix_state.username.is_none()
            || self.matrix_state.password.is_none()
            || self.matrix_client.is_some()
        {
            let resp = Response {
                message: "login_fields_missing".to_string(),
                content: "".to_string(),
            };
            for sub in self.subscribers.iter() {
                self.link.respond(*sub, resp.clone());
            }
            None
        } else {
            let homeserver = self.matrix_state.homeserver.clone().unwrap();

            let client_config = ClientConfig::new();
            let homeserver_url = Url::parse(&homeserver.clone()).unwrap();
            let client = Client::new_with_config(homeserver_url, client_config).unwrap();
            self.matrix_client = Some(client.clone());

            Some(client.clone())
        };
    }
}
