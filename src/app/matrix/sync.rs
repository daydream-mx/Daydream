use std::time::Duration;

use matrix_sdk::{
    api::r0::sync::sync_events::Response as SyncResponse,
    events::{collections::all::RoomEvent, room::message::MessageEventContent},
    identifiers::RoomId,
    Client, SyncSettings,
};
use yew::Callback;
use log::*;
use crate::app::matrix::types::{get_media_download_url, get_video_media_download_url};
use crate::app::matrix::Response;

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
        for (room_id, room) in response.rooms.join {
            // FIXME: Is there a smarter way?
            let resp = Response::SyncPing;
            self.callback.emit(resp);
            for event in room.timeline.events {
                if let Ok(event) = event.deserialize() {
                    self.on_room_message(&room_id, event).await
                }
            }
        }
    }

    async fn on_room_message(&self, room_id: &RoomId, event: RoomEvent) {
        // TODO handle all messages...

        if let RoomEvent::RoomMessage(mut event) = event {
            if let MessageEventContent::Image(mut image_event) = event.clone().content {
                if image_event.url.is_some() {
                    let new_url = Some(get_media_download_url(
                        self.matrix_client.clone(),
                        image_event.url.unwrap(),
                    ));
                    image_event.url = new_url;
                }
                if image_event.info.is_some() {
                    let mut info = image_event.info.unwrap();
                    if info.thumbnail_url.is_some() {
                        let new_url = Some(get_media_download_url(
                            self.matrix_client.clone(),
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
                        self.matrix_client.clone(),
                        video_event.url.unwrap(),
                    ));
                    video_event.url = new_url;
                }
                if video_event.info.is_some() {
                    let mut info = video_event.info.unwrap();
                    if info.thumbnail_url.is_some() {
                        let new_url = Some(get_media_download_url(
                            self.matrix_client.clone(),
                            info.thumbnail_url.unwrap(),
                        ));
                        info.thumbnail_url = new_url;
                    }
                    video_event.info = Some(info);
                }
                event.content = MessageEventContent::Video(video_event);
            }
            let resp = Response::Sync((room_id.clone(), event.clone()));
            self.callback.emit(resp);
        }
    }
}
