use matrix_sdk::events::room::message::MessageEvent;
use matrix_sdk::Room;

pub mod image;
pub mod notice;
pub mod text;
pub mod video;

pub fn is_new_user(prev_event: Option<MessageEvent>, event: MessageEvent) -> bool {
    if let Some(prev_event) = prev_event {
        prev_event.sender != event.sender
    } else {
        true
    }
}

pub fn get_sender_displayname(room: Room, event: MessageEvent) -> String {
    match room.members.get(&event.sender) {
        None => event.sender.to_string(),
        Some(member) => member
            .display_name
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| event.sender.to_string()),
    }
}
