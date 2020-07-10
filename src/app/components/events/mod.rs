use crate::app::matrix::types::get_media_download_url;
use matrix_sdk::events::{MessageEventStub, MessageEventContent};
use matrix_sdk::Room;
use url::Url;

pub mod image;
pub mod notice;
pub mod text;
pub mod video;

pub fn is_new_user<C: MessageEventContent>(prev_event: Option<&MessageEventStub<C>>, event: &MessageEventStub<C>) -> bool {
    if let Some(prev_event) = prev_event {
        prev_event.sender != event.sender
    } else {
        true
    }
}

pub fn get_sender_displayname<'a, C: MessageEventContent>(room: &'a Room, event: &'a MessageEventStub<C>) -> &'a str {
    room.joined_members
        .get(&event.sender)
        .or_else(|| room.invited_members.get(&event.sender))
        .and_then(|member| member.display_name.as_deref())
        .unwrap_or_else(|| event.sender.as_deref())
}

pub fn get_sender_avatar<'a, C: MessageEventContent>(homeserver_url: &'a Url, room: &'a Room, event: &'a MessageEventStub<C>) -> Option<Url> {
    let member = room
        .joined_members
        .get(&event.sender)
        .or_else(|| room.invited_members.get(&event.sender))?;

    Some(get_media_download_url(homeserver_url, member.avatar_url.as_deref()?))
}
