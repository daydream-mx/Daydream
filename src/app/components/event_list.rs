use std::collections::HashMap;
use std::convert::TryFrom;

use log::*;
use matrix_sdk::{
    events::room::message::{MessageEvent, MessageEventContent},
    identifiers::{EventId, RoomId},
    Room,
};
use web_sys::Node;
use yew::prelude::*;
use yew::virtual_dom::VNode;

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
                        return if self.state.events.contains_key(&room_id) {
                            if !(self.state.events[&room_id]
                                .iter()
                                .map(|x| x.event_id.clone())
                                .collect::<Vec<EventId>>()
                                .contains(&msg.event_id))
                            {
                                self.state.events.get_mut(&room_id).unwrap().push(msg);
                                return if room_id
                                    == self.props.current_room.clone().unwrap().room_id
                                {
                                    true
                                } else {
                                    false
                                };
                            } else {
                                false
                            }
                        } else {
                            let mut msgs = Vec::new();
                            msgs.push(msg);
                            self.state.events.insert(room_id.clone(), msgs);
                            return if room_id == self.props.current_room.clone().unwrap().room_id {
                                true
                            } else {
                                false
                            };
                        };
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
                            let events = self.state.events[&self.props.current_room.as_ref().unwrap().room_id].clone();
                            let mut elements: Vec<Html> = Vec::new();
                            for (pos, event) in self.state.events[&self.props.current_room.as_ref().unwrap().room_id].iter().enumerate() {
                                if pos == 0 {
                                    elements.push(self.get_event(None, event));
                                } else {
                                    elements.push(self.get_event(Some(&events[pos - 1]), event));
                                };
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
    fn get_event(&self, prev_event: Option<&MessageEvent>, event: &MessageEvent) -> Html {
        // TODO make encryption supported

        let new_user = if prev_event.is_some() {
            if prev_event.unwrap().sender.to_string() == event.sender.to_string() {
                false
            } else {
                true
            }
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
                    .unwrap_or(event.sender.to_string()),
            }
        };
        match &event.content {
            MessageEventContent::Text(text_event) => {
                if text_event.formatted_body.is_some() {
                    let message = if new_user {
                        format!(
                            "<displayname>{}:</displayname> {}",
                            sender_displayname,
                            text_event.formatted_body.as_ref().unwrap()
                        )
                    } else {
                        format!(
                            "{}",
                            text_event.formatted_body.as_ref().unwrap()
                        )
                    };
                    let js_text_event = {
                        let div = web_sys::window()
                            .unwrap()
                            .document()
                            .unwrap()
                            .create_element("p")
                            .unwrap();
                        div.set_inner_html(
                            message
                                .as_str(),
                        );
                        div
                    };
                    let node = Node::from(js_text_event);
                    let vnode = VNode::VRef(node);
                    vnode
                } else {
                    if new_user {
                        html! {
                           <p><displayname>{sender_displayname}{": "}</displayname>{text_event.body.clone()}</p>
                        }
                    } else {
                        html! {
                           <p>{text_event.body.clone()}</p>
                        }
                    }
                }
            }
            MessageEventContent::Notice(notice_event) => {
                if new_user {
                    html! {
                        <p style="opacity: .6;"><displayname>{sender_displayname}{": "}</displayname>{notice_event.body.clone()}</p>
                    }
                } else {
                    html! {
                       <p style="opacity: .6;">{notice_event.body.clone()}</p>
                    }
                }
            }
            MessageEventContent::Image(image_event) => {
                let caption = format!("{}: {}", sender_displayname, image_event.body);
                if image_event.url.clone().is_some() {
                    let image_url = image_event.url.clone().unwrap();
                    let thumbnail = match image_event.info.clone().unwrap().thumbnail_url {
                        None => image_url.clone(),
                        Some(v) => v,
                    };
                    if new_user {
                        html! {
                            <>
                                <p><displayname>{sender_displayname}{": "}</displayname></p>
                                <div uk-lightbox="">
                                    <a class="uk-inline" href=image_url data-caption=caption >
                                        <img src=thumbnail alt=caption />
                                    </a>
                               </div>
                            </>
                        }
                    } else {
                        html! {
                           <div uk-lightbox="">
                                <a class="uk-inline" href=image_url data-caption=caption >
                                    <img src=thumbnail alt=caption />
                                </a>
                           </div>
                        }
                    }
                } else {
                    html! {}
                }
            }
            _ => {
                html! {}
            }
        }
    }
}
