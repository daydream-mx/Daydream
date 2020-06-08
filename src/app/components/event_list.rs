use std::collections::HashMap;

use log::*;
use matrix_sdk::{
    events::room::message::{MessageEvent, MessageEventContent},
    identifiers::RoomId,
    Room,
};
use yew::prelude::*;

use crate::app::components::events::{image::Image, notice::Notice, text::Text, video::Video};
use crate::app::matrix::{MatrixAgent, Request, Response};

pub struct EventList {
    link: ComponentLink<Self>,
    state: State,
    matrix_agent: Box<dyn Bridge<MatrixAgent>>,
    props: Props,
}

#[derive(Default)]
pub struct State {
    // TODO handle all events
    pub events: HashMap<RoomId, Vec<MessageEvent>>,
    pub message: Option<String>,
}

#[allow(clippy::large_enum_variant)]
pub enum Msg {
    NewMessage(Response),
    SetMessage(String),
    SendMessage,
    Nope,
}

#[derive(Clone, PartialEq, Properties, Debug)]
pub struct Props {
    #[prop_or_default]
    pub current_room: Option<Room>,
}

impl Component for EventList {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let matrix_callback = link.callback(Msg::NewMessage);
        let mut matrix_agent = MatrixAgent::bridge(matrix_callback);

        let state = State {
            events: Default::default(),
            message: None,
        };

        if props.current_room.is_some() {
            let room_id = props.current_room.clone().unwrap().room_id;
            if !state.events.contains_key(&room_id) {
                matrix_agent.send(Request::GetOldMessages((room_id, None)));
            }
        }

        EventList {
            props,
            link,
            matrix_agent,
            state,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::NewMessage(response) => {
                match response {
                    Response::Sync((room_id, msg)) => {
                        // TODO handle all events
                        if self.state.events.contains_key(&room_id) {
                            if !(self.state.events[&room_id]
                                .iter()
                                .map(|x| x.event_id.clone())
                                .any(|x| x == msg.event_id))
                            {
                                self.state.events.get_mut(&room_id).unwrap().push(msg);
                                room_id == self.props.current_room.clone().unwrap().room_id
                            } else {
                                false
                            }
                        } else {
                            let mut msgs = Vec::new();
                            msgs.push(msg);
                            self.state.events.insert(room_id.clone(), msgs);
                            room_id == self.props.current_room.clone().unwrap().room_id
                        }
                    }
                    Response::OldMessages((room_id, mut messages)) => {
                        // This is a clippy false positive
                        #[allow(clippy::map_entry)]
                        if self.state.events.contains_key(&room_id) {
                            self.state
                                .events
                                .get_mut(&room_id)
                                .unwrap()
                                .append(messages.as_mut());
                            true
                        } else {
                            self.state.events.insert(room_id, messages);
                            true
                        }
                    }

                    _ => false,
                }
            }
            Msg::SetMessage(message) => {
                self.state.message = Some(message);
                true
            }
            Msg::SendMessage => {
                info!("Sending Message");
                if self.state.message.is_some() {
                    self.matrix_agent.send(Request::SendMessage((
                        self.props.current_room.clone().unwrap().room_id,
                        self.state.message.clone().unwrap(),
                    )));
                    self.state.message = None;
                }

                true
            }
            Msg::Nope => false,
        }
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        if self.props != props {
            if props.clone().current_room.is_some() {
                let room_id = props.clone().current_room.unwrap().room_id;
                if !self.state.events.contains_key(&room_id) {
                    self.matrix_agent
                        .send(Request::GetOldMessages((room_id, None)));
                }
            }
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        return html! {
            <div class="container uk-flex uk-flex-column uk-width-5-6 uk-padding uk-padding-remove-bottom" style="height: 100%">
                <h1>{ self.props.current_room.as_ref().unwrap().display_name() }</h1>
                <div class="scrollable" style="height: 100%">
                    {
                        if self.state.events.contains_key(&self.props.current_room.as_ref().unwrap().room_id) {
                            let events = self.state.events[&self.props.current_room.as_ref().unwrap().room_id].clone();
                            let mut elements: Vec<Html> = Vec::new();
                            for (pos, event) in self.state.events[&self.props.current_room.as_ref().unwrap().room_id].iter().enumerate() {
                                if pos == 0 {
                                    elements.push(self.get_event(None, event));
                                } else {
                                    elements.push(self.get_event(Some(events[pos - 1].clone()), event));
                                }
                            }
                            elements.into_iter().collect::<Html>()
                        } else {
                            html! {}
                        }
                    }
                    <div id="anchor"></div>
                </div>
                <form class="uk-margin"
                    onsubmit=self.link.callback(|e: FocusEvent| {e.prevent_default();  Msg::Nope})
                    onkeypress=self.link.callback(|e: KeyboardEvent| {
                        if e.key() == "Enter" { Msg::SendMessage } else { Msg::Nope }
                    })>
                    <div>
                        <div class="uk-inline" style="display: block !important;">
                            <span class="uk-form-icon" uk-icon="icon: pencil"></span>
                            <input class="uk-input" type="text"
                                value=&self.state.message.as_ref().unwrap_or(&"".to_string())
                                oninput=self.link.callback(|e: InputData| Msg::SetMessage(e.value))
                            />
                        </div>
                    </div>
                </form>
            </div>
        };
    }
}

impl EventList {
    // Typeinspection of IDEA breaks with this :D
    //noinspection RsTypeCheck
    fn get_event(&self, prev_event: Option<MessageEvent>, event: &MessageEvent) -> Html {
        // TODO make encryption supported

        let new_user = if prev_event.is_some() {
            prev_event.clone().unwrap().sender != event.sender
        } else {
            true
        };

        let sender_displayname = {
            let room = self.props.current_room.as_ref().unwrap();
            match room.members.get(&event.sender) {
                None => event.sender.to_string(),
                Some(member) => member
                    .display_name
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| event.sender.to_string()),
            }
        };
        match &event.content {
            MessageEventContent::Text(text_event) => {
                html! {
                    <Text
                        prev_event=prev_event.clone()
                        event=Some(event.clone())
                        room=Some(self.props.current_room.clone().unwrap())
                        text_event=Some(text_event.clone())
                    />
                }
            }
            MessageEventContent::Notice(notice_event) => {
                html! {
                    <Notice
                        prev_event=prev_event.clone()
                        event=Some(event.clone())
                        room=Some(self.props.current_room.clone().unwrap())
                        notice_event=Some(notice_event.clone())
                    />
                }
            }
            MessageEventContent::Image(image_event) => {
                html! {
                    <Image
                        prev_event=prev_event.clone()
                        event=Some(event.clone())
                        room=Some(self.props.current_room.clone().unwrap())
                        image_event=Some(image_event.clone())
                    />
                }
            }
            MessageEventContent::Video(video_event) => {
                html! {
                    <Video
                        prev_event=prev_event.clone()
                        event=Some(event.clone())
                        room=Some(self.props.current_room.clone().unwrap())
                        video_event=Some(video_event.clone())
                    />
                }
            }
            _ => {
                html! {}
            }
        }
    }
}
