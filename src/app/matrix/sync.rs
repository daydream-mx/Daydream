use std::convert::TryInto;

use log::*;
use matrix_sdk::{
    api::r0::sync::sync_events::Response as SyncResponse, events::collections::all::RoomEvent,
    identifiers::RoomId, Client, SyncSettings,
};

use crate::app::matrix::types::{MessageWrapper, ImageInfoWrapper};
use crate::app::matrix::Response;
use crate::errors::MatrixError;
use yew::Callback;
use std::time::Duration;

pub struct Sync {
    pub(crate) matrix_client: Client,
    pub(crate) callback: Callback<Response>,
}

impl Sync {
    pub async fn start_sync(&self) {
        let client = self.matrix_client.clone();
        let resp = client.clone().sync(SyncSettings::default()).await;
        match resp {
            _ => {
                let resp = Response::FinishedFirstSync;
                self.callback.emit(resp);
            }
        }
        let settings = match client.clone().sync_token().await {
            None => {
                SyncSettings::default().token(client.clone().sync_token().await.unwrap()).timeout(Duration::from_secs(30)).full_state(true)
            },
            Some(token) => {
                SyncSettings::default().token(token).timeout(Duration::from_secs(30)).full_state(true)
            },
        };

        client
            .clone()
            .sync_forever(settings, |response| self.on_sync_response(response))
            .await;
    }

    async fn on_sync_response(&self, response: SyncResponse) {
        for (room_id, room) in response.rooms.join {
            for event in room.timeline.events {
                if let Ok(event) = event.deserialize() {
                    self.on_room_message(&room_id, event).await
                }
            }
        }
    }

    async fn on_room_message(&self, room_id: &RoomId, event: RoomEvent) {
        // TODO handle all messages...

        match event {
            RoomEvent::RoomMessage(event) => {
                let wrapped_event_result: Result<MessageWrapper, MatrixError> = event.try_into();
                match wrapped_event_result {
                    Ok(mut wrapped_event) => {
                        if wrapped_event.room_id.is_none() {
                            wrapped_event.room_id = Some(room_id.clone());
                        }

                        wrapped_event.sender_displayname = Some(
                            wrapped_event
                                .get_displayname(self.matrix_client.clone())
                                .await,
                        );

                        // Convert mxc URLs
                        if wrapped_event.info.is_some() {
                            let mxc_url = wrapped_event
                                .info
                                .clone()
                                .unwrap()
                                .url
                                .clone()
                                .unwrap();
                            let download_url = wrapped_event
                                .get_media_download_url(
                                    self.matrix_client.clone(),
                                    mxc_url,
                                );
                            let mxc_thumbnail_url = wrapped_event
                                .info
                                .clone()
                                .unwrap()
                                .thumbnail_url
                                .clone()
                                .unwrap();
                            let thumbnail_download_url = wrapped_event
                                .get_media_download_url(
                                    self.matrix_client.clone(),
                                    mxc_thumbnail_url,
                                );
                            wrapped_event.info = Some(ImageInfoWrapper {
                                url: Some(download_url),
                                thumbnail_url: Some(thumbnail_download_url),
                            });
                        }
                        let resp = Response::Sync(wrapped_event);
                        self.callback.emit(resp);
                    }
                    Err(_) => {
                        // Ignore events we cant parse
                        return;
                    }
                }
            }
            _ => {
                return;
            }
        }
    }
}
