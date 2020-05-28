use std::convert::TryInto;

use log::*;
use matrix_sdk::{
    api::r0::sync::sync_events::Response as SyncResponse, events::collections::all::RoomEvent,
    events::room::message::MessageEventContent, identifiers::RoomId, Client, SyncSettings,
};

use crate::app::matrix::types::MessageWrapper;
use crate::app::matrix::Response;
use yew::Callback;

pub struct Sync {
    pub(crate) matrix_client: Client,
    pub(crate) callback: Callback<Response>,
}

impl Sync {
    pub async fn start_sync(&self) {
        let client = self.matrix_client.clone();
        let resp = client.clone().sync(SyncSettings::default()).await;
        match resp {
            Ok(_) => {
                let resp = Response::FinishedFirstSync;
                self.callback.emit(resp);
            }
            _ => {}
        }

        let settings = SyncSettings::default().token(client.clone().sync_token().await.unwrap());
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
            RoomEvent::RoomMessage(event) => match event.content {
                MessageEventContent::Text(_) => {
                    let mut wrapped_event: MessageWrapper =
                        event.try_into().expect("m.room.message");

                    if wrapped_event.room_id.is_none() {
                        wrapped_event.room_id = Some(room_id.clone());
                    }

                    wrapped_event.sender_displayname = Some(
                        wrapped_event
                            .get_displayname(self.matrix_client.clone())
                            .await,
                    );
                    let resp = Response::Sync(wrapped_event);
                    self.callback.emit(resp);
                }
                _ => {
                    return;
                }
            },
            _ => {
                return;
            }
        }
    }
}
