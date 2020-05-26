use std::collections::HashSet;
use std::convert::TryFrom;
use std::sync::{Arc, Mutex};

use log::*;
use matrix_sdk::{Client, ClientConfig, Session, SyncSettings};
use serde_derive::{Deserialize, Serialize};
use url::Url;
use wasm_bindgen_futures::spawn_local;
use yew::format::Json;
use yew::services::{storage::Area, StorageService};
use yew::worker::*;

use crate::constants::AUTH_KEY;
use crate::errors::MatrixError;

mod sync;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct MatrixClient {
    pub(crate) homeserver: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct SessionStore {
    access_token: String,
    user_id: String,
    device_id: String,
    homeserver_url: String,
}

#[derive(Clone, Debug)]
pub struct MatrixAgent {
    link: AgentLink<MatrixAgent>,
    matrix_state: MatrixClient,
    matrix_client: Option<Client>,
    // TODO make arc mutex :(
    subscribers: HashSet<HandlerId>,
    storage: Arc<Mutex<StorageService>>,
    session: Option<SessionStore>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    SetHomeserver(String),
    SetUsername(String),
    SetPassword(String),
    Login(),
    GetLoggedIn,
    StartSync,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Response {
    Error(MatrixError),
    LoggedIn(bool),
    // TODO properly handle sync events
    Sync(String),
}

impl Agent for MatrixAgent {
    type Reach = Context;
    type Message = ();
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
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
        info!("session: {:#?}", session);
        MatrixAgent {
            link,
            matrix_state: Default::default(),
            matrix_client: None,
            subscribers: HashSet::new(),
            storage,
            session,
        }
    }

    fn update(&mut self, _: Self::Message) {}

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }
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
                    let resp = Response::Error(MatrixError::MissingClient);
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
                    // TODO handle login error
                    if agent.session.is_some() {
                        let stored_session = agent.session.clone().unwrap();
                        let session = Session {
                            access_token: stored_session.access_token,
                            user_id: matrix_sdk::identifiers::UserId::try_from(
                                stored_session.user_id.as_str(),
                            )
                            .unwrap(),
                            device_id: stored_session.device_id,
                        };
                        client.restore_login(session).await;
                    } else {
                        let login_response: matrix_sdk::api::r0::session::login::Response = client
                            .login(
                                username.clone(),
                                password.clone(),
                                None,
                                Some("Daydream".to_string()),
                            )
                            .await
                            .unwrap();
                        let session_store = SessionStore {
                            access_token: login_response.access_token,
                            user_id: login_response.user_id.to_string(),
                            device_id: login_response.device_id,
                            homeserver_url: client.homeserver().clone().into_string(),
                        };
                        let mut storage = agent.storage.lock().unwrap();
                        storage.store(AUTH_KEY, Json(&session_store));
                    }

                    info!("did login");
                    let resp = Response::LoggedIn(true);
                    info!("prepared login response");
                    for sub in subscribers.iter() {
                        agent.link.respond(*sub, resp.clone());
                    }
                    info!("sent login response");
                });
            }
            Request::GetLoggedIn => {
                let subscribers = self.subscribers.clone();
                let login_client = self.login();
                if login_client.is_none() {
                    let resp = Response::Error(MatrixError::MissingClient);
                    for sub in self.subscribers.iter() {
                        self.link.respond(*sub, resp.clone());
                    }
                    return;
                }

                // Always clone agent after having tried to login!
                let agent = self.clone();

                spawn_local(async move {
                    let logged_in = agent.get_logged_in().await;

                    if !logged_in && agent.session.is_some() {
                        error!("Not logged in but got session");
                    } else {
                        let resp = Response::LoggedIn(logged_in.clone());
                        for sub in subscribers.iter() {
                            agent.link.respond(*sub, resp.clone());
                        }
                    }
                });
            }
            Request::StartSync => {
                // Always clone agent after having tried to login!
                let agent = self.clone();
                spawn_local(async move {
                    agent.start_sync().await;
                });
            }
        }
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}

unsafe impl Send for MatrixAgent {}
unsafe impl std::marker::Sync for MatrixAgent {}

impl MatrixAgent {
    async fn start_sync(&self) {
        let sync = sync::Sync {
            matrix_client: self.matrix_client.clone().unwrap(),
            callback: |x: Response| {
                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, x.clone());
                }
            },
        };
        sync.start_sync().await;
    }

    async fn get_logged_in(&self) -> bool {
        if self.matrix_client.is_none() {
            return false;
        }
        self.matrix_client.clone().unwrap().logged_in().await
    }

    fn login(&mut self) -> Option<Client> {
        return if (self.matrix_state.homeserver.is_none()
            || self.matrix_state.username.is_none()
            || self.matrix_state.password.is_none()
            || self.matrix_client.is_some())
            && self.session.is_none()
        {
            let resp = Response::Error(MatrixError::MissingFields);
            for sub in self.subscribers.iter() {
                self.link.respond(*sub, resp.clone());
            }
            None
        } else if self.session.is_some() {
            let homeserver = self.session.clone().unwrap().homeserver_url;

            let client_config = ClientConfig::new();
            let homeserver_url = Url::parse(&homeserver.clone()).unwrap();
            let client = Client::new_with_config(homeserver_url, client_config).unwrap();
            self.matrix_client = Some(client.clone());

            // Also directly restore Login data
            let stored_session = self.session.clone().unwrap();
            let session = Session {
                access_token: stored_session.access_token,
                user_id: matrix_sdk::identifiers::UserId::try_from(stored_session.user_id.as_str())
                    .unwrap(),
                device_id: stored_session.device_id,
            };
            let client_clone = client.clone();
            spawn_local(async move {
                client_clone.restore_login(session).await;
            });

            Some(client.clone())
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
