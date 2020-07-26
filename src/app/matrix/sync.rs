use std::mem;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use log::*;
use matrix_sdk::{
    api::r0::filter::{FilterDefinition, LazyLoadOptions, RoomEventFilter, RoomFilter},
    api::r0::sync::sync_events::Filter,
    api::r0::sync::sync_events::Response as SyncResponse,
    events::{
        room::message::MessageEventContent, AnySyncMessageEvent, AnySyncRoomEvent,
        AnySyncStateEvent, EventJson,
    },
    identifiers::RoomId,
    locks::RwLock,
    Client, Room, SyncSettings,
};
use wasm_bindgen_futures::spawn_local;
use yew::Callback;

use lazy_static::lazy_static;
use matrix_sdk::js_int::UInt;

use crate::app::components::events::RoomExt;
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
        let settings = SyncSettings::default()
            .timeout(Duration::from_secs(30))
            .filter(Filter::FilterDefinition(FilterDefinition {
                room: Some(RoomFilter {
                    timeline: Some(RoomEventFilter {
                        limit: Some(UInt::new(20).unwrap()),
                        lazy_load_options: LazyLoadOptions::Enabled {
                            include_redundant_members: true,
                        },
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }));
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

    async fn on_state_event(&self, room_id: &RoomId, event: AnySyncStateEvent) {
        if let AnySyncStateEvent::RoomCreate(_event) = event {
            info!("Sent JoinedRoomSync State");
            let resp = Response::JoinedRoomSync(room_id.clone());
            self.callback.emit(resp);
        }
    }

    async fn on_room_message(&self, room_id: &RoomId, event: AnySyncRoomEvent) {
        // TODO handle all messages...

        if let AnySyncRoomEvent::State(AnySyncStateEvent::RoomCreate(_create_event)) = event.clone()
        {
            info!("Sent JoinedRoomSync Timeline");
            let resp = Response::JoinedRoomSync(room_id.clone());
            self.callback.emit(resp);
        }

        if let AnySyncRoomEvent::Message(AnySyncMessageEvent::RoomMessage(mut event)) = event {
            if let MessageEventContent::Text(text_event) = event.content.clone() {
                let homeserver_url = self.matrix_client.clone().homeserver().clone();

                let cloned_event = event.clone();
                let client = self.matrix_client.clone();
                let local_room_id = room_id.clone();
                let sync_number = SYNC_NUMBER.lock().unwrap();
                if *sync_number == 1 {
                    spawn_local(async move {
                        let room: Arc<RwLock<Room>> = client
                            .clone()
                            .get_joined_room(&local_room_id)
                            .await
                            .unwrap();
                        if cloned_event.sender.clone() != client.user_id().await.unwrap() {
                            let (avatar_url, room_name, displayname) = {
                                let room = room.read().await;
                                (
                                    room.get_sender_avatar(
                                        &homeserver_url,
                                        &AnySyncMessageEvent::RoomMessage(cloned_event.clone()),
                                    ),
                                    room.display_name(),
                                    room.get_sender_displayname(&AnySyncMessageEvent::RoomMessage(
                                        cloned_event,
                                    ))
                                    .to_string(),
                                )
                            };

                            let title = if displayname == room_name {
                                displayname
                            } else {
                                format!("{} ({})", displayname, room_name)
                            };

                            let notification =
                                Notifications::new(avatar_url, title, text_event.body.clone());
                            notification.show();
                        }
                    });
                }
            }
            if let MessageEventContent::Image(image_event) = &mut event.content {
                if let Some(image_url) = &mut image_event.url {
                    let old_image_url = mem::take(image_url);
                    *image_url = get_media_download_url(
                        self.matrix_client.clone().homeserver(),
                        &old_image_url,
                    )
                    .to_string();
                }

                if let Some(info) = &mut image_event.info {
                    if let Some(thumbnail_url) = &mut info.thumbnail_url {
                        let old_thumbnail_url = mem::take(thumbnail_url);
                        *thumbnail_url = get_media_download_url(
                            self.matrix_client.clone().homeserver(),
                            &old_thumbnail_url,
                        )
                        .to_string();
                    }
                }
            }
            if let MessageEventContent::Video(video_event) = &mut event.content {
                if let Some(video_url) = &mut video_event.url {
                    let old_video_url = mem::take(video_url);
                    *video_url = get_video_media_download_url(
                        self.matrix_client.clone().homeserver(),
                        old_video_url,
                    )
                    .to_string();
                }

                if let Some(info) = &mut video_event.info {
                    if let Some(thumbnail_url) = &mut info.thumbnail_url {
                        let old_thumbnail_url = mem::take(thumbnail_url);
                        *thumbnail_url = get_media_download_url(
                            self.matrix_client.clone().homeserver(),
                            &old_thumbnail_url,
                        )
                        .to_string();
                    }
                }
            }

            let serialized_event = EventJson::from(AnySyncMessageEvent::RoomMessage(event));
            let resp = Response::Sync((room_id.clone(), serialized_event));
            self.callback.emit(resp);
        }
    }
}
