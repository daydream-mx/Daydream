use log::*;
use matrix_sdk::{
    api::r0::{filter::RoomEventFilter, message::get_message_events::Direction},
    events::{
        collections::all::RoomEvent,
        room::message::{MessageEvent, MessageEventContent, TextMessageEventContent, FormattedBody, MessageFormat},
        EventJson,
    },
    identifiers::RoomId,
    js_int::UInt,
    locks::RwLock,
    Client, ClientConfig, MessagesRequestBuilder, Room, Session,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::sync::{Arc, Mutex};
use url::Url;
use wasm_bindgen_futures::spawn_local;
use yew::format::Json;
use yew::services::{storage::Area, StorageService};
use yew::worker::*;

use crate::app::matrix::types::{get_media_download_url, get_video_media_download_url};
use crate::constants::AUTH_KEY;
use crate::errors::MatrixError;
use pulldown_cmark::{html, Options, Parser};

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

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum Response {
    Error(MatrixError),
    LoggedIn(bool),
    // TODO properly handle sync events
    Sync((RoomId, MessageEvent)),
    SyncPing,
    JoinedRoomList(HashMap<RoomId, Room>),
    Userdata(),
    OldMessages((RoomId, Vec<MessageEvent>)),
    JoinedRoom((RoomId, Room)),
}

#[derive(Debug, Clone)]
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
                self.matrix_state.homeserver = Some(homeserver);
            }
            Request::SetUsername(username) => {
                self.matrix_state.username = Some(username);
            }
            Request::SetPassword(password) => {
                self.matrix_state.password = Some(password);
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
                let client = login_client.unwrap();
                let username = self.matrix_state.username.clone().unwrap();
                let password = self.matrix_state.password.clone().unwrap();
                let subscribers = self.subscribers.clone();
                let agent = self.clone();
                spawn_local(async move {
                    // TODO handle login error
                    if agent.session.is_some() {
                        let stored_session = agent.session.unwrap();
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
                        // FIXME gracefully handle login errors
                        let login_response: matrix_sdk::api::r0::session::login::Response = client
                            .login(username, password, None, Some("Daydream".to_string()))
                            .await
                            .unwrap();
                        let session_store = SessionStore {
                            access_token: login_response.access_token,
                            user_id: login_response.user_id.to_string(),
                            device_id: login_response.device_id,
                            homeserver_url: client.homeserver().to_string(),
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
                        for sub in agent.subscribers.iter() {
                            let resp = Response::LoggedIn(logged_in);
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
                spawn_local(async move {
                    let rooms: Arc<RwLock<HashMap<RoomId, Arc<RwLock<Room>>>>> =
                        agent.matrix_client.unwrap().joined_rooms();

                    let readable_rooms = rooms.read().await;
                    let mut rooms_unarced: HashMap<RoomId, Room> = HashMap::new();
                    for (id, room) in readable_rooms.iter() {
                        let unarced_room = (*room.write().await).clone();
                        rooms_unarced.insert(id.clone(), unarced_room);
                    }
                    for sub in agent.subscribers.iter() {
                        let resp = Response::JoinedRoomList(rooms_unarced.clone());
                        agent.link.respond(*sub, resp);
                    }
                });
            }
            Request::GetUserdata => {
                // Noop
            }
            Request::GetOldMessages((room_id, from)) => {
                let agent = self.clone();
                spawn_local(async move {
                    let mut builder = &mut MessagesRequestBuilder::new();
                    builder = builder.room_id(room_id.clone());
                    if let Some(from) = from {
                        builder = builder.from(from);
                    } else {
                        builder = builder.from(
                            agent
                                .matrix_client
                                .clone()
                                .unwrap()
                                .sync_token()
                                .await
                                .unwrap(),
                        );
                    }
                    let filter = RoomEventFilter {
                        types: Some(vec!["m.room.message".to_string()]),
                        ..Default::default()
                    };
                    builder = builder
                        .filter(filter)
                        .direction(Direction::Backward)
                        .limit(UInt::new(30).unwrap());

                    // TODO handle error gracefully
                    let messsages = agent
                        .matrix_client
                        .clone()
                        .unwrap()
                        .room_messages(builder.clone())
                        .await
                        .unwrap();
                    // TODO save end point for future loading

                    let mut wrapped_messages: Vec<MessageEvent> = Vec::new();
                    let chunk_iter: Vec<EventJson<RoomEvent>> = messsages.chunk;
                    let (oks, _): (Vec<_>, Vec<_>) = chunk_iter
                        .iter()
                        .map(|event| event.deserialize())
                        .partition(Result::is_ok);

                    let deserialized_events: Vec<RoomEvent> =
                        oks.into_iter().map(Result::unwrap).collect();

                    for event in deserialized_events.into_iter().rev() {
                        if let RoomEvent::RoomMessage(mut event) = event {
                            if let MessageEventContent::Image(mut image_event) = event.clone().content {
                                if image_event.url.is_some() {
                                    let new_url = Some(get_media_download_url(
                                        agent.matrix_client.clone().unwrap(),
                                        image_event.url.unwrap(),
                                    ));
                                    image_event.url = new_url;
                                }
                                if image_event.info.is_some() {
                                    let mut info = image_event.info.unwrap();
                                    if info.thumbnail_url.is_some() {
                                        let new_url = Some(get_media_download_url(
                                            agent.matrix_client.clone().unwrap(),
                                            info.thumbnail_url.unwrap(),
                                        ));
                                        info.thumbnail_url = new_url;
                                    }
                                    image_event.info = Some(info);
                                }
                                event.content = MessageEventContent::Image(image_event);
                            }
                            if let MessageEventContent::Video(mut video_event) = event.content {
                                if video_event.url.is_some() {
                                    let new_url = Some(get_video_media_download_url(
                                        agent.matrix_client.clone().unwrap(),
                                        video_event.url.unwrap(),
                                    ));
                                    video_event.url = new_url;
                                }
                                if video_event.info.is_some() {
                                    let mut info = video_event.info.unwrap();
                                    if info.thumbnail_url.is_some() {
                                        let new_url = Some(get_media_download_url(
                                            agent.matrix_client.clone().unwrap(),
                                            info.thumbnail_url.unwrap(),
                                        ));
                                        info.thumbnail_url = new_url;
                                    }
                                    video_event.info = Some(info);
                                }
                                event.content = MessageEventContent::Video(video_event);
                            }
                            wrapped_messages.push(event.clone());
                        }
                    }

                    for sub in agent.subscribers.iter() {
                        let resp =
                            Response::OldMessages((room_id.clone(), wrapped_messages.clone()));
                        agent.link.respond(*sub, resp);
                    }
                });
            }
            Request::GetJoinedRoom(room_id) => {
                let agent = self.clone();
                spawn_local(async move {
                    let room: Arc<RwLock<Room>> = agent
                        .matrix_client
                        .unwrap()
                        .get_joined_room(&room_id)
                        .await
                        .unwrap();
                    let read_clone = room.read().await;
                    let clean_room = (*read_clone).clone();
                    for sub in agent.subscribers.iter() {
                        let resp = Response::JoinedRoom((room_id.clone(), clean_room.clone()));
                        agent.link.respond(*sub, resp);
                    }
                });
            }
            Request::SendMessage((room_id, message)) => {
                let client = self.matrix_client.clone().unwrap();
                spawn_local(async move {
                    let mut options = Options::empty();
                    options.insert(Options::ENABLE_STRIKETHROUGH);
                    let parser = Parser::new_ext(message.as_str(), options);

                    let mut formatted_message: String =
                        String::with_capacity(message.len() * 3 / 2);
                    html::push_html(&mut formatted_message, parser);
                    formatted_message = formatted_message.replace("<p>", "").replace("</p>", "");
                    formatted_message.pop();

                    let content = if formatted_message == message {
                        MessageEventContent::Text(TextMessageEventContent::new_plain(message))
                    } else {
                        MessageEventContent::Text(TextMessageEventContent {
                            body: message,
                            relates_to: None,
                            formatted: Some(FormattedBody{
                                body: formatted_message,
                                format: MessageFormat::Html,
                            })
                        })
                    };
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
        self.matrix_client.as_ref().unwrap().logged_in().await
    }

    fn login(&mut self) -> Option<Client> {
        if (self.matrix_state.homeserver.is_none()
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
            let homeserver_url = Url::parse(&homeserver).unwrap();
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

            Some(client)
        } else {
            let homeserver = self.matrix_state.homeserver.clone().unwrap();

            let client_config = ClientConfig::new();
            let homeserver_url = Url::parse(&homeserver).unwrap();
            let client = Client::new_with_config(homeserver_url, client_config).unwrap();
            self.matrix_client = Some(client.clone());

            Some(client)
        }
    }
}
