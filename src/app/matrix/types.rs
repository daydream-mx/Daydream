use js_int::UInt;
use matrix_sdk::identifiers::RoomId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SmallRoom {
    pub(crate) name: String,
    pub(crate) unread_notifications: Option<UInt>,
    pub(crate) unread_highlight: Option<UInt>,
    pub(crate) id: RoomId,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct MessageWrapper {
    pub(crate) sender_displayname: String,
    pub(crate) room_id: RoomId,
    // TODO use ruma structs
    pub(crate) content: String,
}
