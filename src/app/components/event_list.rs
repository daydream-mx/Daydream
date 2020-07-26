use std::{collections::HashMap, rc::Rc};

use crate::utils::ruma::AnyMessageEventExt;
use log::*;
use matrix_sdk::{
    events::{room::message::MessageEventContent, AnyMessageEventContent, AnySyncMessageEvent},
    identifiers::RoomId,
    Room,
};
use yew::{prelude::*, virtual_dom::VList};

use crate::app::components::{
    events::{image::Image, notice::Notice, text::Text, video::Video},
    input::Input,
};
use crate::app::matrix::{MatrixAgent, Request, Response};

pub struct EventList {
    on_submit: Callback<String>,
    state: State,
    matrix_agent: Box<dyn Bridge<MatrixAgent>>,
    props: Props,
}

#[derive(Default)]
pub struct State {
    // TODO handle all events
    pub events: HashMap<RoomId, Vec<AnySyncMessageEvent>>,
}

#[allow(clippy::large_enum_variant)]
pub enum Msg {
    NewMessage(Response),
    SendMessage(String),
    Nope,
}

#[derive(Clone, PartialEq, Properties, Debug)]
pub struct Props {
    pub current_room: Rc<Room>,
}

impl Component for EventList {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let matrix_callback = link.callback(Msg::NewMessage);
        let mut matrix_agent = MatrixAgent::bridge(matrix_callback);

        let state = State {
            events: Default::default(),
        };

        let room_id = props.current_room.room_id.clone();
        if !state.events.contains_key(&room_id) {
            matrix_agent.send(Request::GetOldMessages((room_id, None)));
        }

        EventList {
            on_submit: link.callback(Msg::SendMessage),
            props,
            matrix_agent,
            state,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::NewMessage(Response::Sync((room_id, raw_msg))) => {
                // TODO handle all events
                if let Ok(msg) = raw_msg.deserialize() {
                    if self.state.events.contains_key(&room_id) {
                        if !(self.state.events[&room_id]
                            .iter()
                            .any(|x| x.event_id() == msg.event_id()))
                        {
                            self.state.events.get_mut(&room_id).unwrap().push(msg);
                            room_id == self.props.current_room.room_id
                        } else {
                            false
                        }
                    } else {
                        let msgs = vec![msg];
                        self.state.events.insert(room_id.clone(), msgs);
                        room_id == self.props.current_room.room_id
                    }
                } else {
                    false
                }
            }
            Msg::NewMessage(Response::OldMessages((room_id, messages))) => {
                let mut deserialized_messages: Vec<AnySyncMessageEvent> = messages
                    .iter()
                    .map(|x| x.deserialize())
                    .filter_map(Result::ok)
                    .map(|x| x.without_room_id())
                    .collect();
                // This is a clippy false positive
                #[allow(clippy::map_entry)]
                if self.state.events.contains_key(&room_id) {
                    self.state
                        .events
                        .get_mut(&room_id)
                        .unwrap()
                        .append(deserialized_messages.as_mut());
                    true
                } else {
                    self.state.events.insert(room_id, deserialized_messages);
                    true
                }
            }
            Msg::NewMessage(_) => false,
            Msg::SendMessage(message) => {
                info!("Sending Message");
                self.matrix_agent.send(Request::SendMessage((
                    self.props.current_room.room_id.clone(),
                    message,
                )));
                false
            }
            Msg::Nope => false,
        }
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        if self.props != props {
            let room_id = props.current_room.room_id.clone();
            if !self.state.events.contains_key(&room_id) {
                self.matrix_agent
                    .send(Request::GetOldMessages((room_id, None)));
            }

            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let events = if self
            .state
            .events
            .contains_key(&self.props.current_room.room_id)
        {
            let events = &self.state.events[&self.props.current_room.room_id];

            let mut html_nodes = VList::new();
            if let Some(event) = events.first() {
                html_nodes.add_child(self.get_event(None, event));
            }
            html_nodes.add_children(
                events
                    .windows(2)
                    .map(|e| self.get_event(Some(&e[0]), &e[1])),
            );

            html_nodes.into()
        } else {
            html! {}
        };

        html! {
            <div class="event-list">
                <div class="room-title"><div><h1>{ self.props.current_room.display_name() }</h1></div></div>
                <div class="scrollable message-scrollarea">
                    <div class="message-container">
                        { events }
                        <div id="anchor"></div>
                    </div>
                </div>
                <Input on_submit=&self.on_submit/>
            </div>
        }
    }
}

impl EventList {
    // Typeinspection of IDEA breaks with this :D
    //noinspection RsTypeCheck
    fn get_event(
        &self,
        prev_event: Option<&AnySyncMessageEvent>,
        event: &AnySyncMessageEvent,
    ) -> Html {
        // TODO make encryption supported

        match &event.content() {
            AnyMessageEventContent::RoomMessage(room_message) => match room_message {
                MessageEventContent::Text(text_event) => {
                    html! {
                        <Text
                            prev_event=prev_event.cloned()
                            event=event.clone()
                            room=self.props.current_room.clone()
                            text_event=text_event.clone()
                        />
                    }
                }
                MessageEventContent::Notice(notice_event) => {
                    html! {
                        <Notice
                            prev_event=prev_event.cloned()
                            event=event.clone()
                            room=self.props.current_room.clone()
                            notice_event=notice_event.clone()
                        />
                    }
                }
                MessageEventContent::Image(image_event) => {
                    html! {
                        <Image
                            prev_event=prev_event.cloned()
                            event=event.clone()
                            room=self.props.current_room.clone()
                            image_event=image_event.clone()
                        />
                    }
                }
                MessageEventContent::Video(video_event) => {
                    html! {
                        <Video
                            prev_event=prev_event.cloned()
                            event=event.clone()
                            room=self.props.current_room.clone()
                            video_event=video_event.clone()
                        />
                    }
                }
                _ => html! {},
            },
            _ => html! {},
        }
    }
}
