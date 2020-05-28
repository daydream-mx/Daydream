use std::convert::TryFrom;
use std::sync::Arc;

use futures_locks::RwLock;
use matrix_sdk::{
    events::room::message::{MessageEvent, MessageEventContent, TextMessageEventContent},
    identifiers::{RoomId, UserId},
    js_int::UInt,
    Client, Room,
};
use serde::{Deserialize, Serialize};

// TODO: Add Into trait
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SmallRoom {
    pub(crate) name: String,
    pub(crate) unread_notifications: Option<UInt>,
    pub(crate) unread_highlight: Option<UInt>,
    pub(crate) id: RoomId,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct MessageWrapper {
    pub(crate) sender_displayname: Option<String>,
    pub(crate) room_id: Option<RoomId>,
    pub(crate) sender: UserId,
    // TODO use ruma structs
    pub(crate) content: String,
}

impl MessageWrapper {
    pub async fn get_displayname(&self, client: Client) -> String {
        let room: Arc<RwLock<Room>> = client
            .get_joined_room(&self.room_id.clone().unwrap())
            .await
            .unwrap();
        let room = room.read().await;
        let member = room.members.get(&self.sender.clone()).unwrap();
        member
            .display_name
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or(self.sender.clone().to_string())
    }
}

impl TryFrom<MessageEvent> for MessageWrapper {
    type Error = ();

    fn try_from(event: MessageEvent) -> Result<Self, Self::Error> {
        return if let MessageEvent {
            content: MessageEventContent::Text(TextMessageEventContent { body: msg_body, .. }),
            sender,
            ..
        } = event
        {
            Ok(MessageWrapper {
                sender_displayname: None, // We cant get it without a Client and therefor cant calculate it here
                room_id: event.room_id,
                sender,
                content: msg_body,
            })
        } else {
            Err(())
        };
    }
}
