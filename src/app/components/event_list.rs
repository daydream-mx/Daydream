use linked_hash_set::LinkedHashSet;
use log::*;
use matrix_sdk::identifiers::{EventId, RoomId};
use serde::{Deserialize, Serialize};
use yew::prelude::*;

use crate::app::matrix::types::MessageWrapper;
use crate::app::matrix::{MatrixAgent, Request, Response};

pub struct EventList {
    link: ComponentLink<Self>,
    state: State,
    matrix_agent: Box<dyn Bridge<MatrixAgent>>,
    props: Props,
}

#[derive(Serialize, Deserialize, Default)]
pub struct State {
    // TODO handle all events
    // TODO use roomId based hashmap
    pub events: LinkedHashSet<MessageWrapper>,
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
    pub current_room: Option<RoomId>,
    #[prop_or_default]
    pub displayname: String,
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

        if props.clone().current_room.is_some() {
            let room_id = props.clone().current_room.clone().unwrap();
            if state
                .events
                .iter()
                .filter(|x| x.room_id.clone().unwrap() == room_id)
                .collect::<LinkedHashSet<&MessageWrapper>>()
                .is_empty()
            {
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
                    Response::Sync(msg) => {
                        // TODO handle all events
                        if !(self
                            .state
                            .events
                            .iter()
                            .map(|x| x.event_id.clone())
                            .collect::<Vec<EventId>>()
                            .contains(&msg.event_id.clone()))
                        {
                            self.state.events.insert(msg);
                        }
                        true
                    }
                    Response::OldMessages(messages) => {
                        // TODO this doesn't seem smart
                        // let mut new_events_map = LinkedHashSet::new();
                        info!("{}", self.state.events.len());
                        self.state.events = self
                            .state
                            .events
                            .clone()
                            .into_iter()
                            .chain(messages)
                            .collect();
                        info!("{}", self.state.events.len());
                        /*for event in self.state.events.clone().into_iter() {
                            new_events_map.insert(event);
                        }
                        for event in messages.clone().into_iter() {
                            new_events_map.insert(event);
                        }
                        self.state.events = new_events_map.clone();*/
                        true
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
                        self.props.current_room.clone().unwrap(),
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
            info!("{:#?}", props);
            if props.clone().current_room.is_some() {
                let room_id = props.clone().current_room.clone().unwrap();
                if self
                    .state
                    .events
                    .iter()
                    .filter(|x| x.room_id.clone().unwrap() == room_id)
                    .collect::<LinkedHashSet<&MessageWrapper>>()
                    .is_empty()
                {
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
                <h1>{ self.props.displayname.clone() }</h1>
                <div class="scrollable" style="height: 100%">
                    { self.state.events.iter().filter(|x| x.room_id.clone().unwrap() == self.props.current_room.clone().unwrap()).map(|event| self.get_event(event.clone())).collect::<Html>() }
                    <div id="anchor"></div>
                </div>
                <form  class="uk-margin"
                onsubmit=self.link.callback(|e: FocusEvent| {e.prevent_default();  Msg::Nope})
                onkeypress=self.link.callback(|e: KeyboardEvent| {
                    if e.key() == "Enter" { Msg::SendMessage } else { Msg::Nope }
                })>
                    <div>
                        <div class="uk-inline" style="display: block !important;">
                            <span class="uk-form-icon" uk-icon="icon: pencil"></span>
                            <input class="uk-input" type="text"
                                value=&self.state.message.clone().unwrap_or("".to_string())
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
    fn get_event(&self, event: MessageWrapper) -> Html {
        if event.event_type == "m.text" {
            html! {
               <p>{event.sender_displayname.unwrap_or(event.sender.to_string()).clone()}{": "}{event.content.clone()}</p>
            }
        } else if event.event_type == "m.notice" {
            html! {
               <p style="opacity: .6;">{event.sender_displayname.unwrap_or(event.sender.to_string()).clone()}{": "}{event.content.clone()}</p>
            }
        } else if event.event_type == "m.image" {
            let caption = format!(
                "{}: {}",
                event
                    .sender_displayname
                    .unwrap_or(event.sender.to_string())
                    .clone(),
                event.content.clone()
            );
            let thumbnail = match event.info.clone().unwrap().thumbnail_url {
                None => event.info.clone().unwrap().url.clone().unwrap(),
                Some(v) => v,
            };
            html! {
               <div uk-lightbox="">
                    <a class="uk-inline" href=event.info.clone().unwrap().url.clone().unwrap() data-caption=caption.clone() >
                        <img src=thumbnail.clone() alt=caption.clone() />
                    </a>
               </div>
            }
        } else {
            html! {}
        }
    }
}
