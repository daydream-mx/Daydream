use std::collections::{HashMap, HashSet};
use std::convert::{TryFrom, TryInto};
use std::sync::{Arc, Mutex};

use futures_locks::RwLock;
use linked_hash_set::LinkedHashSet;
use log::*;
use matrix_sdk::{
    api::r0::{filter::RoomEventFilter, message::get_message_events::Direction},
    events::collections::all::RoomEvent,
    events::room::message::{MessageEventContent, TextMessageEventContent},
    identifiers::RoomId,
    js_int::UInt,
    Client, ClientConfig, MessagesRequestBuilder, Room, Session,
};
use serde::{Deserialize, Serialize};
use url::Url;
use wasm_bindgen_futures::spawn_local;
use yew::format::Json;
use yew::services::{storage::Area, StorageService};
use yew::worker::*;

use crate::app::matrix::types::{MessageWrapper, SmallRoom};
use crate::constants::AUTH_KEY;
use crate::errors::MatrixError;

mod sync;
pub mod types;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct MatrixClient {
    pub(crate) homeserver: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct SessionStore {
    pub(crate) access_token: String,
    pub(crate) user_id: String,
    pub(crate) device_id: String,
    pub(crate) homeserver_url: String,
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
    GetUserdata,
    GetOldMessages((RoomId, Option<String>)),
    StartSync,
    GetJoinedRooms,
    GetJoinedRoom(RoomId),
    SendMessage((RoomId, String)),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Response {
    Error(MatrixError),
    LoggedIn(bool),
    // TODO properly handle sync events
    Sync(MessageWrapper),
    FinishedFirstSync,
    JoinedRoomList(HashMap<RoomId, SmallRoom>),
    Userdata(),
    OldMessages(LinkedHashSet<MessageWrapper>),
    JoinedRoom((RoomId, SmallRoom)),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Msg {
    OnSyncResponse(Response),
}

impl Agent for MatrixAgent {
    type Reach = Context<MatrixAgent>;
    type Message = Msg;
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
        MatrixAgent {
            link,
            matrix_state: Default::default(),
            matrix_client: None,
            subscribers: HashSet::new(),
            storage,
            session,
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::OnSyncResponse(resp) => {
                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, resp.clone());
                }
            }
        }
    }

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
                    for sub in self.subscribers.iter() {
                        let resp = Response::Error(MatrixError::MissingClient);
                        self.link.respond(*sub, resp);
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

                    for sub in subscribers.iter() {
                        let resp = Response::LoggedIn(true);
                        agent.link.respond(*sub, resp);
                    }
                });
            }
            Request::GetLoggedIn => {
                let subscribers = self.subscribers.clone();
                let login_client = self.login();
                if login_client.is_none() {
                    for sub in self.subscribers.iter() {
                        let resp = Response::Error(MatrixError::MissingClient);
                        self.link.respond(*sub, resp);
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
                        for sub in subscribers.iter() {
                            let resp = Response::LoggedIn(logged_in.clone());
                            agent.link.respond(*sub, resp);
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
            Request::GetJoinedRooms => {
                let agent = self.clone();
                let client = agent.matrix_client.clone().unwrap();
                spawn_local(async move {
                    for sub in agent.subscribers.iter() {
                        let rooms: Arc<RwLock<HashMap<RoomId, Arc<RwLock<Room>>>>> =
                            client.clone().joined_rooms();
                        let mut rooms_list_hack = HashMap::new();
                        for (id, room) in rooms.read().await.iter() {
                            let small_room = SmallRoom {
                                name: room.read().await.display_name(),
                                unread_notifications: room
                                    .read()
                                    .await
                                    .unread_notifications
                                    .clone(),
                                unread_highlight: room.read().await.unread_highlight.clone(),
                                id: id.clone(),
                            };
                            rooms_list_hack.insert(id.clone(), small_room);
                        }

                        let resp = Response::JoinedRoomList(rooms_list_hack);
                        agent.link.respond(*sub, resp);
                    }
                });
            }
            Request::GetUserdata => {
                // Noop
            }
            Request::GetOldMessages((room_id, from)) => {
                let agent = self.clone();
                let client = self.matrix_client.clone().unwrap();
                spawn_local(async move {
                    let mut builder = &mut MessagesRequestBuilder::new();
                    builder = builder.room_id(room_id.clone());
                    if from.is_some() {
                        builder = builder.from(from.unwrap());
                    } else {
                        builder = builder.from(client.clone().sync_token().await.unwrap());
                    }
                    let filter = RoomEventFilter {
                        types: Some(vec!["m.room.message".to_string()]),
                        ..Default::default()
                    };
                    builder = builder
                        .filter(filter)
                        .direction(Direction::Backward)
                        .limit(UInt::new(30).unwrap());

                    let messsages = client.room_messages(builder.clone()).await.unwrap();
                    // TODO save end point for future loading

                    let mut wrapped_messages: LinkedHashSet<MessageWrapper> = LinkedHashSet::new();
                    for event in messsages.chunk.iter().rev() {
                        if let Ok(event) = event.deserialize() {
                            match event {
                                RoomEvent::RoomMessage(event) => match event.content {
                                    MessageEventContent::Text(_) => {
                                        let mut wrapped_event: MessageWrapper =
                                            event.try_into().expect("m.room.message");

                                        if wrapped_event.room_id.is_none() {
                                            wrapped_event.room_id = Some(room_id.clone());
                                        }

                                        wrapped_event.sender_displayname = Some(
                                            wrapped_event.get_displayname(client.clone()).await,
                                        );
                                        wrapped_messages.insert(wrapped_event);
                                    }
                                    _ => {
                                        return;
                                    }
                                },
                                _ => {
                                    continue;
                                }
                            }
                        }
                    }

                    for sub in agent.subscribers.iter() {
                        let resp = Response::OldMessages(wrapped_messages.clone());
                        agent.link.respond(*sub, resp);
                    }
                });
            }
            Request::GetJoinedRoom(room_id) => {
                let agent = self.clone();
                let client = self.matrix_client.clone().unwrap();
                spawn_local(async move {
                    let room: Arc<RwLock<Room>> =
                        client.get_joined_room(&room_id.clone()).await.unwrap();
                    let small_room = SmallRoom {
                        name: room.read().await.display_name(),
                        unread_notifications: room.read().await.unread_notifications.clone(),
                        unread_highlight: room.read().await.unread_highlight.clone(),
                        id: room_id.clone(),
                    };
                    for sub in agent.subscribers.iter() {
                        let resp = Response::JoinedRoom((room_id.clone(), small_room.clone()));
                        agent.link.respond(*sub, resp);
                    }
                });
            }
            Request::SendMessage((room_id, message)) => {
                let client = self.matrix_client.clone().unwrap();
                let content =
                    MessageEventContent::Text(TextMessageEventContent::new_plain(message.clone()));
                spawn_local(async move {
                    client.room_send(&room_id, content, None).await;
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
            callback: self.link.callback(Msg::OnSyncResponse),
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
            for sub in self.subscribers.iter() {
                let resp = Response::Error(MatrixError::MissingFields);
                self.link.respond(*sub, resp);
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
