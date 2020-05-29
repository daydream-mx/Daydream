use std::convert::TryFrom;
use std::sync::Arc;

use futures_locks::RwLock;
use matrix_sdk::{
    events::room::message::{MessageEvent, MessageEventContent},
    identifiers::{EventId, RoomId, UserId},
    js_int::UInt,
    Client, Room,
};
use serde::{Deserialize, Serialize};

use crate::errors::MatrixError;

// TODO: Add Into trait
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SmallRoom {
    pub(crate) name: String,
    pub(crate) unread_notifications: Option<UInt>,
    pub(crate) unread_highlight: Option<UInt>,
    pub(crate) id: RoomId,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct ImageInfoWrapper {
    pub(crate) url: Option<String>,
    pub(crate) thumbnail_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct MessageWrapper {
    pub(crate) sender_displayname: Option<String>,
    pub(crate) room_id: Option<RoomId>,
    pub(crate) event_id: EventId,
    pub(crate) event_type: String,
    pub(crate) sender: UserId,
    // TODO use ruma structs
    pub(crate) content: String,
    pub(crate) info: Option<ImageInfoWrapper>,
}

impl MessageWrapper {
    pub async fn get_displayname(&self, client: Client) -> String {
        let room: Arc<RwLock<Room>> = client
            .get_joined_room(&self.room_id.clone().unwrap())
            .await
            .unwrap();
        let room = room.read().await;
        let member = match room.members.get(&self.sender.clone()) {
            None => {
                return self.sender.clone().to_string();
            }
            Some(v) => v,
        };
        member
            .display_name
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or(self.sender.clone().to_string())
    }

    pub fn get_media_download_url(&self, client: Client, mxc_url: String) -> String {
        let url_parts_raw = mxc_url.clone().replace("mxc://", "");
        let url_parts: Vec<&str> = url_parts_raw.split("/").collect();
        let server_name = url_parts.first().unwrap().to_string();
        let media_id = url_parts.last().unwrap().to_string();
        let new_path = format!(
            "_matrix/media/r0/download/{}/{}/fix.jpg",
            server_name.clone(),
            media_id.clone()
        );
        let mut new_url = client.clone().homeserver().clone();
        new_url.set_path(new_path.as_str());
        new_url.to_string()
    }
}

impl TryFrom<MessageEvent> for MessageWrapper {
    type Error = MatrixError;

    fn try_from(event: MessageEvent) -> Result<Self, Self::Error> {
        return match event.content {
            MessageEventContent::Text(text_event) => {
                Ok(MessageWrapper {
                    sender_displayname: None, // We cant get it without a Client and therefor cant calculate it here
                    room_id: event.room_id,
                    event_id: event.event_id,
                    event_type: "m.text".to_string(),
                    sender: event.sender,
                    content: text_event.body,
                    info: None,
                })
            }
            MessageEventContent::Image(image_event) => {
                let thumbnail_url = match image_event.info {
                    None => None,
                    Some(i) => i.thumbnail_url,
                };
                Ok(MessageWrapper {
                    sender_displayname: None, // We cant get it without a Client and therefor cant calculate it here
                    room_id: event.room_id,
                    event_id: event.event_id,
                    event_type: "m.image".to_string(),
                    sender: event.sender,
                    content: image_event.body,
                    info: Some(ImageInfoWrapper {
                        url: image_event.url,
                        thumbnail_url,
                    }),
                })
            }
            MessageEventContent::Notice(notice_event) => {
                Ok(MessageWrapper {
                    sender_displayname: None, // We cant get it without a Client and therefor cant calculate it here
                    room_id: event.room_id,
                    event_id: event.event_id,
                    event_type: "m.notice".to_string(),
                    sender: event.sender,
                    content: notice_event.body,
                    info: None,
                })
            }
            _ => Err(MatrixError::UnsupportedEvent),
        };
    }
}
