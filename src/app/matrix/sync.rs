use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use log::*;
use matrix_sdk::{
    api::r0::sync::sync_events::Response as SyncResponse,
    events::{
        collections::all::{RoomEvent, StateEvent},
        room::message::MessageEventContent,
        EventJson,
    },
    identifiers::RoomId,
    locks::RwLock,
    Client, Room, SyncSettings,
};
use wasm_bindgen_futures::spawn_local;
use yew::Callback;

use lazy_static::lazy_static;

use crate::app::components::events::{get_sender_avatar, get_sender_displayname};
use crate::app::matrix::types::{get_media_download_url, get_video_media_download_url};
use crate::app::matrix::Response;
use crate::utils::notifications::Notifications;

lazy_static! {
    static ref SYNC_NUMBER: Mutex<i32> = Mutex::new(0);
}

pub struct Sync {
    pub(crate) matrix_client: Client,
    pub(crate) callback: Callback<Response>,
}

impl Sync {
    pub async fn start_sync(&self) {
        debug!("start sync!");
        let client = self.matrix_client.clone();
        let settings = SyncSettings::default().timeout(Duration::from_secs(30));
        //.full_state(true);

        debug!("start sync_forever!");
        client
            .sync_forever(settings, |response| self.on_sync_response(response))
            .await;
    }

    async fn on_sync_response(&self, response: SyncResponse) {
        debug!("got sync!");

        // FIXME: Is there a smarter way?
        let resp = Response::SyncPing;
        self.callback.emit(resp);
        for (room_id, room) in response.rooms.join {
            for event in room.state.events {
                if let Ok(event) = event.deserialize() {
                    self.on_state_event(&room_id, event).await
                }
            }
            for event in room.timeline.events {
                if let Ok(event) = event.deserialize() {
                    self.on_room_message(&room_id, event).await
                }
            }
        }
        let mut sync_number = SYNC_NUMBER.lock().unwrap();
        if *sync_number == 0 {
            *sync_number = 1;
        }
    }

    async fn on_state_event(&self, room_id: &RoomId, event: StateEvent) {
        if let StateEvent::RoomCreate(_event) = event {
            info!("Sent JoinedRoomSync State");
            let resp = Response::JoinedRoomSync(room_id.clone());
            self.callback.emit(resp);
        }
    }

    async fn on_room_message(&self, room_id: &RoomId, event: RoomEvent) {
        // TODO handle all messages...

        if let RoomEvent::RoomCreate(_create_event) = event.clone() {
            info!("Sent JoinedRoomSync Timeline");
            let resp = Response::JoinedRoomSync(room_id.clone());
            self.callback.emit(resp);
        }

        if let RoomEvent::RoomMessage(mut event) = event {
            if let MessageEventContent::Text(text_event) = event.clone().content {
                let homeserver_url = self.matrix_client.clone().homeserver().clone();

                let cloned_event = event.clone();
                let client = self.matrix_client.clone();
                let local_room_id = room_id.clone();
                let sync_number = SYNC_NUMBER.lock().unwrap();
                if *sync_number == 1 {
                    spawn_local(async move {
                        let room: Arc<RwLock<Room>> = client
                            .clone()
                            .get_joined_room(&local_room_id.clone())
                            .await
                            .unwrap();
                        let read_clone = room.read().await;
                        let clean_room = (*read_clone).clone();
                        let avatar_url = get_sender_avatar(
                            homeserver_url,
                            clean_room.clone(),
                            cloned_event.clone(),
                        );
                        let room_name = clean_room.display_name();
                        let displayname = get_sender_displayname(clean_room, cloned_event.clone());

                        let title = if displayname == room_name {
                            displayname
                        } else {
                            format!("{} ({})", displayname, room_name)
                        };

                        let notification =
                            Notifications::new(avatar_url, title, text_event.body.clone());
                        notification.show();
                    });
                }
            }
            if let MessageEventContent::Image(mut image_event) = event.clone().content {
                if image_event.url.is_some() {
                    let new_url = Some(get_media_download_url(
                        self.matrix_client.clone().homeserver(),
                        image_event.url.unwrap(),
                    ));
                    image_event.url = new_url;
                }
                if image_event.info.is_some() {
                    let mut info = image_event.info.unwrap();
                    if info.thumbnail_url.is_some() {
                        let new_url = Some(get_media_download_url(
                            self.matrix_client.clone().homeserver(),
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
                        self.matrix_client.clone().homeserver(),
                        video_event.url.unwrap(),
                    ));
                    video_event.url = new_url;
                }
                if video_event.info.is_some() {
                    let mut info = video_event.info.unwrap();
                    if info.thumbnail_url.is_some() {
                        let new_url = Some(get_media_download_url(
                            self.matrix_client.clone().homeserver(),
                            info.thumbnail_url.unwrap(),
                        ));
                        info.thumbnail_url = new_url;
                    }
                    video_event.info = Some(info);
                }
                event.content = MessageEventContent::Video(video_event);
            }

            let serialized_event = EventJson::from(event.clone());
            let resp = Response::Sync((room_id.clone(), serialized_event));
            //self.callback.emit(resp);
        }
    }
}
