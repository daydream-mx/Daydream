use matrix_sdk::identifiers::RoomId;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SmallRoom {
    pub(crate) name: String,
    pub(crate) id: RoomId
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct MessageWrapper {
    pub(crate) room_id: RoomId,
    // TODO use ruma structs
    pub(crate) content: String,
}
