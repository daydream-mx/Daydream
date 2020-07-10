use matrix_sdk::events::{
    room::redaction::RedactionEventStub, AnyMessageEvent, AnyMessageEventStub, MessageEvent,
    MessageEventContent, MessageEventStub,
};

pub trait AnyMessageEventExt {
    fn without_room_id(self) -> AnyMessageEventStub;
}

impl AnyMessageEventExt for AnyMessageEvent {
    fn without_room_id(self) -> AnyMessageEventStub {
        fn without_room_id<C>(ev: MessageEvent<C>) -> MessageEventStub<C>
        where
            C: MessageEventContent,
        {
            MessageEventStub {
                content: ev.content,
                event_id: ev.event_id,
                sender: ev.sender,
                origin_server_ts: ev.origin_server_ts,
                unsigned: ev.unsigned,
            }
        }

        use AnyMessageEventStub::*;

        match self {
            Self::CallAnswer(ev) => CallAnswer(without_room_id(ev)),
            Self::CallInvite(ev) => CallInvite(without_room_id(ev)),
            Self::CallHangup(ev) => CallHangup(without_room_id(ev)),
            Self::CallCandidates(ev) => CallCandidates(without_room_id(ev)),
            Self::RoomEncrypted(ev) => RoomEncrypted(without_room_id(ev)),
            Self::RoomMessage(ev) => RoomMessage(without_room_id(ev)),
            Self::RoomMessageFeedback(ev) => RoomMessageFeedback(without_room_id(ev)),
            Self::Sticker(ev) => Sticker(without_room_id(ev)),
            Self::Custom(ev) => Custom(without_room_id(ev)),
            Self::RoomRedaction(ev) => RoomRedaction(RedactionEventStub {
                content: ev.content,
                event_id: ev.event_id,
                sender: ev.sender,
                origin_server_ts: ev.origin_server_ts,
                unsigned: ev.unsigned,
                redacts: ev.redacts,
            }),
        }
    }
}
