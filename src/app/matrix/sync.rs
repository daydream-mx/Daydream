use crate::app::matrix::Response;
use log::*;
use matrix_sdk::{api::r0::sync::sync_events::Response as SyncResponse, events::collections::all::RoomEvent, events::room::message::{MessageEvent, MessageEventContent, TextMessageEventContent}, identifiers::RoomId, Client, SyncSettings, Error};
use crate::app::matrix::types::MessageWrapper;

pub struct Sync<F>
where
    F: Fn(Response) + std::marker::Sync,
{
    pub(crate) matrix_client: Client,
    pub(crate) callback: F,
}

impl<F> Sync<F>
where
    F: Fn(Response) + std::marker::Sync,
{
    pub async fn start_sync(&self) {
        let client = self.matrix_client.clone();
        let resp = client.clone().sync(SyncSettings::default()).await;
        match resp {
            Ok(_) => {
                let resp = Response::FinishedFirstSync;
                (self.callback)(resp);
            },
            _ => {},
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
        let msg_body = if let RoomEvent::RoomMessage(MessageEvent {
            content: MessageEventContent::Text(TextMessageEventContent { body: msg_body, .. }),
            ..
        }) = event
        {
            msg_body.clone()
        } else {
            return;
        };

        let wrapper = MessageWrapper {
            room_id: room_id.clone(),
            content: msg_body
        };

        let resp = Response::Sync(wrapper);
        (self.callback)(resp);
    }
}
