use crate::app::matrix::types::MessageWrapper;
use crate::app::matrix::Response;
use futures_locks::RwLock;
use log::*;
use matrix_sdk::{
    api::r0::sync::sync_events::Response as SyncResponse,
    events::collections::all::RoomEvent,
    events::room::message::{MessageEvent, MessageEventContent, TextMessageEventContent},
    identifiers::RoomId,
    Client, Room, SyncSettings,
};
use std::sync::Arc;

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
        if let RoomEvent::RoomMessage(MessageEvent {
            content: MessageEventContent::Text(TextMessageEventContent { body: msg_body, .. }),
            sender,
            ..
        }) = event
        {
            let name = {
                let room: Arc<RwLock<Room>> =
                    self.matrix_client.get_joined_room(room_id).await.unwrap();
                let room = room.read().await;
                let member = room.members.get(&sender).unwrap();
                member
                    .display_name
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or(sender.to_string())
            };

            let wrapper = MessageWrapper {
                sender_displayname: name.clone(),
                room_id: room_id.clone(),
                content: msg_body.clone(),
            };

            let resp = Response::Sync(wrapper);
            (self.callback)(resp);
        } else {
            return;
        };
    }
}
