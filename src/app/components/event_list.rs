use std::collections::HashMap;

use log::*;
use matrix_sdk::{
    events::room::message::{MessageEvent, MessageEventContent},
    identifiers::{EventId, RoomId},
    Room,
};
use yew::prelude::*;

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
                matrix_agent.send(Request::GetOldMessages((room_id.clone(), None)));
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
                                .collect::<Vec<EventId>>()
                                .contains(&msg.event_id))
                            {
                                self.state.events.get_mut(&room_id).unwrap().push(msg);
                                return true;
                            } else {
                                return false;
                            }
                        } else {
                            let mut msgs = Vec::new();
                            msgs.push(msg);
                            self.state.events.insert(room_id, msgs);
                            return true;
                        }
                        return false;
                    }
                    Response::OldMessages((room_id, mut messages)) => {
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
                        .send(Request::GetOldMessages((room_id.clone(), None)));
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
                            self.state.events[&self.props.current_room.as_ref().unwrap().room_id].iter().map(|event| self.get_event(event)).collect::<Html>()
                        } else {
                            html!{}
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
    fn get_event(&self, event: &MessageEvent) -> Html {
        let sender_displayname = {
            let room = self.props.current_room.as_ref().unwrap();
            let member = room.members.get(&event.sender).unwrap();
            member
                .display_name
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or(event.sender.to_string())
        };
        match &event.content {
            MessageEventContent::Text(text_event) => {
                html! {
                   <p>{sender_displayname}{": "}{text_event.body.clone()}</p>
                }
            }
            MessageEventContent::Notice(notice_event) => {
                html! {
                   <p style="opacity: .6;">{sender_displayname}{": "}{notice_event.body.clone()}</p>
                }
            }
            MessageEventContent::Image(image_event) => {
                let caption = format!("{}: {}", sender_displayname, image_event.body);

                let image_url = image_event.url.clone().unwrap();
                let thumbnail = match image_event.info.clone().unwrap().thumbnail_url {
                    None => image_url.clone(),
                    Some(v) => v,
                };
                html! {
                   <div uk-lightbox="">
                        <a class="uk-inline" href=image_url data-caption=caption >
                            <img src=thumbnail alt=caption />
                        </a>
                   </div>
                }
            }
            _ => {
                html! {}
            }
        }
    }
}
