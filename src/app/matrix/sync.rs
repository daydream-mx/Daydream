use std::time::Duration;

use log::*;
use matrix_sdk::{
    api::r0::sync::sync_events::Response as SyncResponse, events::collections::all::RoomEvent,
    identifiers::RoomId, Client, SyncSettings,
};
use yew::Callback;

use crate::app::matrix::Response;

pub struct Sync {
    pub(crate) matrix_client: Client,
    pub(crate) callback: Callback<Response>,
}

impl Sync {
    pub async fn start_sync(&self) {
        let client = self.matrix_client.clone();
        let settings = SyncSettings::default().timeout(Duration::from_secs(30));

        client
            .sync_forever(settings, |response| self.on_sync_response(response))
            .await;
    }

    async fn on_sync_response(&self, response: SyncResponse) {
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

        match event {
            RoomEvent::RoomMessage(event) => {
                let resp = Response::Sync((room_id.clone(), event));
                self.callback.emit(resp);
            }
            _ => {
                return;
            }
        }
    }
}
