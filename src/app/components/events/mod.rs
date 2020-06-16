use crate::app::matrix::types::get_media_download_url;
use matrix_sdk::events::room::message::MessageEvent;
use matrix_sdk::Room;
use url::Url;

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

pub fn get_sender_avatar(homeserver_url: Url, room: Room, event: MessageEvent) -> Option<String> {
    match room.members.get(&event.sender) {
        None => None,
        Some(member) => {
            let avatar_url_mxc = member.avatar_url.as_ref().map(ToString::to_string);
            match avatar_url_mxc {
                None => None,
                Some(v) => Some(get_media_download_url(&homeserver_url, v)),
            }
        }
    }
}
