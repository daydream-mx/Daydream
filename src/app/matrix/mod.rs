use std::collections::HashSet;
use std::sync::Arc;

use log::*;
use matrix_sdk::{
    api::r0::{
        filter::RoomEventFilter,
        message::{
            get_message_events::Direction, get_message_events::Request as GetMessagesRequest,
        },
    },
    events::{
        room::message::{FormattedBody, MessageEventContent, TextMessageEventContent},
        AnyMessageEvent, AnyRoomEvent, AnySyncMessageEvent,
    },
    identifiers::RoomId,
    js_int::uint,
    locks::RwLock,
    Client, Raw, Room,
};
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew::worker::*;

use crate::app::matrix::types::{get_media_download_url, get_video_media_download_url};
use crate::errors::MatrixError;
use login::{login, SessionStore};

pub mod login;
mod sync;
pub mod types;

#[derive(Default, Clone, Debug)]
pub struct MatrixClient {
    pub(crate) homeserver: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
}

#[derive(Clone, Debug)]
pub struct MatrixAgent {
    link: AgentLink<MatrixAgent>,
    matrix_state: MatrixClient,
    matrix_client: Option<Client>,
    // TODO make arc mutex :(
    subscribers: HashSet<HandlerId>,
    session: Option<SessionStore>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    SetHomeserver(String),
    SetUsername(String),
    SetPassword(String),
    SetSession(SessionStore),
    Login,
    GetLoggedIn,
    GetOldMessages((RoomId, Option<String>)),
    StartSync,
    GetJoinedRoom(RoomId),
    SendMessage((RoomId, String)),
}

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Response {
    Error(MatrixError),
    LoggedIn(bool),
    // TODO properly handle sync events
    Sync((RoomId, Raw<AnySyncMessageEvent>)),
    JoinedRoomSync(RoomId),
    SyncPing,
    OldMessages((RoomId, Vec<Raw<AnyMessageEvent>>)),
    JoinedRoom((RoomId, Room)),
    SaveSession(SessionStore),
}

#[derive(Debug, Clone)]
pub enum Msg {
    OnSyncResponse(Response),
}

impl Agent for MatrixAgent {
    type Reach = Public<Self>;
    type Message = Msg;
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        MatrixAgent {
            link,
            matrix_state: Default::default(),
            matrix_client: None,
            subscribers: HashSet::new(),
            session: Default::default(),
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
            Request::SetSession(session) => {
                self.session = Some(session);
            }

            Request::SetHomeserver(homeserver) => {
                self.matrix_state.homeserver = Some(homeserver);
            }
            Request::SetUsername(username) => {
                self.matrix_state.username = Some(username);
            }
            Request::SetPassword(password) => {
                self.matrix_state.password = Some(password);
            }
            Request::Login => {
                info!("Starting Login");
                let homeserver = self.matrix_state.homeserver.as_ref();
                let session = self.session.as_ref();
                let client = login(session, homeserver);
                match client {
                    Ok(client) => {
                        if let Some(_session) = session {
                            for sub in self.subscribers.iter() {
                                let resp = Response::LoggedIn(true);
                                self.link.respond(*sub, resp);
                            }
                        }
                        self.matrix_client = Some(client.clone());
                        let username = self.matrix_state.username.clone().unwrap();
                        let password = self.matrix_state.password.clone().unwrap();
                        let agent = self.clone();
                        spawn_local(async move {
                            // FIXME gracefully handle login errors
                            // TODO make the String to &str conversion smarter if possible
                            let login_response = agent
                                .matrix_client
                                .as_ref()
                                .unwrap()
                                .login(&username, &password, None, Some("Daydream"))
                                .await;
                            match login_response {
                                Ok(login_response) => {
                                    let session_store = SessionStore {
                                        access_token: login_response.access_token,
                                        user_id: login_response.user_id.to_string(),
                                        device_id: login_response.device_id.into(),
                                        homeserver_url: client.homeserver().to_string(),
                                    };
                                    for sub in agent.subscribers.iter() {
                                        let resp = Response::SaveSession(session_store.clone());
                                        agent.link.respond(*sub, resp);
                                        let resp = Response::LoggedIn(true);
                                        agent.link.respond(*sub, resp);
                                    }
                                }
                                Err(e) => {
                                    if let matrix_sdk::Error::Reqwest(e) = e {
                                        match e.status() {
                                            None => {
                                                for sub in agent.subscribers.iter() {
                                                    let resp = Response::Error(
                                                        MatrixError::SDKError(e.to_string()),
                                                    );
                                                    agent.link.respond(*sub, resp);
                                                }
                                            }
                                            Some(v) => {
                                                if v.is_server_error() {
                                                    for sub in agent.subscribers.iter() {
                                                        let resp = Response::Error(
                                                            MatrixError::LoginTimeout,
                                                        );
                                                        agent.link.respond(*sub, resp);
                                                    }
                                                } else {
                                                    for sub in agent.subscribers.iter() {
                                                        let resp = Response::Error(
                                                            MatrixError::SDKError(e.to_string()),
                                                        );
                                                        agent.link.respond(*sub, resp);
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        for sub in agent.subscribers.iter() {
                                            let resp = Response::Error(MatrixError::SDKError(
                                                e.to_string(),
                                            ));
                                            agent.link.respond(*sub, resp);
                                        }
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => {
                        for sub in self.subscribers.iter() {
                            let resp = Response::Error(e.clone());
                            self.link.respond(*sub, resp);
                        }
                    }
                }
            }
            Request::GetLoggedIn => {
                let homeserver = self.matrix_state.homeserver.as_ref();
                let session = self.session.clone();
                let client = login(session.as_ref(), homeserver);
                match client {
                    Ok(client) => {
                        info!("Got client");
                        self.matrix_client = Some(client);
                        info!("Client set");
                        let agent = self.clone();
                        spawn_local(async move {
                            let logged_in = agent.get_logged_in().await;

                            if !logged_in && session.is_some() {
                                error!("Not logged in but got session");
                            } else {
                                for sub in agent.subscribers.iter() {
                                    let resp = Response::LoggedIn(logged_in);
                                    agent.link.respond(*sub, resp);
                                }
                            }
                        });
                    }
                    Err(e) => {
                        error!("Got no client: {:?}", e);
                        for sub in self.subscribers.iter() {
                            let resp = Response::Error(e.clone());
                            self.link.respond(*sub, resp);
                        }
                    }
                }
            }
            Request::StartSync => {
                // Always clone agent after having tried to login!
                let agent = self.clone();
                spawn_local(async move {
                    agent.start_sync().await;
                });
            }
            Request::GetOldMessages((room_id, from)) => {
                let agent = self.clone();
                spawn_local(async move {
                    let sync_token = match from {
                        Some(from) => from,
                        None => agent
                            .matrix_client
                            .as_ref()
                            .unwrap()
                            .sync_token()
                            .await
                            .unwrap(),
                    };
                    let mut req =
                        GetMessagesRequest::new(&room_id, &sync_token, Direction::Backward);
                    let filter = RoomEventFilter {
                        types: Some(vec!["m.room.message".to_string()]),
                        ..Default::default()
                    };
                    // TODO find better way than cloning
                    req.filter = Some(filter);
                    req.limit = uint!(30);

                    // TODO handle error gracefully
                    let messsages = agent
                        .matrix_client
                        .clone()
                        .unwrap()
                        .room_messages(req)
                        .await
                        .unwrap();
                    // TODO save end point for future loading

                    let mut wrapped_messages: Vec<Raw<AnyMessageEvent>> = Vec::new();
                    let chunk_iter: Vec<Raw<AnyRoomEvent>> = messsages.chunk;
                    let (oks, _): (Vec<_>, Vec<_>) = chunk_iter
                        .iter()
                        .map(|event| event.deserialize())
                        .partition(Result::is_ok);

                    let deserialized_events: Vec<AnyRoomEvent> =
                        oks.into_iter().map(Result::unwrap).collect();

                    for event in deserialized_events.into_iter().rev() {
                        // TODO deduplicate betweeen this and sync
                        if let AnyRoomEvent::Message(AnyMessageEvent::RoomMessage(mut event)) =
                            event
                        {
                            if let MessageEventContent::Image(mut image_event) =
                                event.clone().content
                            {
                                if let Some(image_event_url) = image_event.url {
                                    let new_url = get_media_download_url(
                                        agent.matrix_client.as_ref().unwrap().homeserver(),
                                        &image_event_url,
                                    );
                                    image_event.url = Some(new_url.to_string());
                                }
                                if let Some(mut info) = image_event.info {
                                    if let Some(thumbnail_url) = info.thumbnail_url.as_ref() {
                                        let new_url = get_media_download_url(
                                            agent.matrix_client.as_ref().unwrap().homeserver(),
                                            thumbnail_url,
                                        );
                                        info.thumbnail_url = Some(new_url.to_string());
                                    }
                                    image_event.info = Some(info);
                                }
                                event.content = MessageEventContent::Image(image_event);
                            }
                            if let MessageEventContent::Video(mut video_event) = event.content {
                                if let Some(video_event_url) = video_event.url {
                                    let new_url = get_video_media_download_url(
                                        agent.matrix_client.as_ref().unwrap().homeserver(),
                                        video_event_url,
                                    );
                                    video_event.url = Some(new_url.to_string());
                                }
                                if let Some(mut info) = video_event.info {
                                    if let Some(thumbnail_url) = info.thumbnail_url {
                                        let new_url = Some(get_media_download_url(
                                            agent.matrix_client.as_ref().unwrap().homeserver(),
                                            &thumbnail_url,
                                        ))
                                        .unwrap();
                                        info.thumbnail_url = Some(new_url.to_string());
                                    }
                                    video_event.info = Some(info);
                                }
                                event.content = MessageEventContent::Video(video_event);
                            }

                            let serialized_event =
                                Raw::from(AnyMessageEvent::RoomMessage(event.clone()));
                            wrapped_messages.push(serialized_event);
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
            Request::SendMessage((room_id, raw_message)) => {
                let client = self.matrix_client.clone().unwrap();
                spawn_local(async move {
                    let replacer = gh_emoji::Replacer::new();
                    let message = replacer.replace_all(raw_message.as_str());

                    let mut options = Options::empty();
                    options.insert(Options::ENABLE_STRIKETHROUGH);
                    let parser = Parser::new_ext(message.as_ref(), options);

                    let mut formatted_message: String =
                        String::with_capacity(message.len() * 3 / 2);
                    html::push_html(&mut formatted_message, parser);
                    formatted_message = formatted_message.replace("<p>", "").replace("</p>", "");
                    formatted_message.pop();

                    let content = if formatted_message == message {
                        MessageEventContent::Text(TextMessageEventContent::plain(message))
                    } else {
                        MessageEventContent::Text(TextMessageEventContent {
                            body: message.to_string(),
                            relates_to: None,
                            formatted: Some(FormattedBody::html(formatted_message)),
                        })
                    };
                    if let Err(e) = client.room_send(&room_id, content, None).await {
                        // TODO show error in UI or try again if possible
                        error!("Error sending message: {}", e);
                    }
                });
            }
        }
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }

    fn name_of_resource() -> &'static str {
        "worker.js"
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
}
