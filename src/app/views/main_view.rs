use log::*;
use serde::{Deserialize, Serialize};
use yew::prelude::*;
use yew::ComponentLink;

use crate::app::matrix::{MatrixAgent, Request, Response};
use std::collections::{HashSet, HashMap};
use matrix_sdk::{Client, ClientConfig, Session};
use url::Url;
use std::convert::TryFrom;
use wasm_bindgen_futures::spawn_local;
use crate::app::matrix::types::{SmallRoom, MessageWrapper};
use matrix_sdk::identifiers::RoomId;

pub struct MainView {
    link: ComponentLink<Self>,
    state: State,
    matrix_agent: Box<dyn Bridge<MatrixAgent>>,
}

pub enum Msg {
    NewMessage(Response),
    ChangeRoom(String)
}

#[derive(Serialize, Deserialize, Default)]
pub struct State {
    // TODO handle all events
    pub events: HashSet<MessageWrapper>,
    pub rooms: HashMap<RoomId, SmallRoom>,
    pub current_room: Option<RoomId>
}

impl Component for MainView {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let matrix_callback = link.callback(Msg::NewMessage);
        let mut matrix_agent = MatrixAgent::bridge(matrix_callback);
        matrix_agent.send(Request::StartSync);
        let state = State {
            events: Default::default(),
            rooms: Default::default(),
            current_room: None
        };

        MainView {
            link,
            matrix_agent,
            state,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::NewMessage(response) => {
                match response {
                    Response::FinishedFirstSync => {
                        self.matrix_agent.send(Request::GetJoinedRooms);
                    }
                    Response::Sync(msg) => {
                        // TODO handle all events
                        self.state.events.insert(msg);
                    }
                    Response::JoinedRoomList(rooms) => {
                        self.state.rooms = rooms
                    }
                    _ => {}
                }
            }
            Msg::ChangeRoom(room) => {
                self.state.current_room = Some(RoomId::try_from(room).unwrap());

            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        if !self.state.rooms.is_empty() {
            if self.state.current_room.is_none() {
                return html! {
                    <div class="uk-flex h-100 non-scrollable-container">
                        <div class="container h-100 uk-width-1-6">
                            <ul class="scrollable h-100 uk-padding uk-nav-default uk-nav-parent-icon" uk-nav="">
                                <li class="uk-nav-header">{"Rooms"}</li>
                                { self.state.rooms.iter().map(|(_, room)| self.get_room(room.clone())).collect::<Html>() }
                            </ul>
                        </div>

                        <div class="container h-100 uk-width-5-6 uk-padding">
                            <div class="scrollable h-100">
                                // TODO add some content to the empty page
                            </div>
                        </div>
                    </div>
                }
            } else {
                return html! {
                    <div class="uk-flex h-100 non-scrollable-container">
                        <div class="container h-100 uk-width-1-6">
                            <ul class="scrollable h-100 uk-padding uk-nav-default uk-nav-parent-icon" uk-nav="">
                                <li class="uk-nav-header">{"Rooms"}</li>
                                { self.state.rooms.iter().map(|(_, room)| self.get_room(room.clone())).collect::<Html>() }
                            </ul>
                        </div>

                        <div class="container h-100 uk-width-5-6 uk-padding">
                            <h1>{ self.state.rooms.iter().filter(|(id, _)| **id == self.state.current_room.clone().unwrap()).map(|(_, room)| room.name.clone()).collect::<String>() }</h1>
                            <div class="scrollable h-100">
                                { self.state.events.iter().filter(|x| x.room_id == self.state.current_room.clone().unwrap()).map(|event| self.get_event(event.content.clone())).collect::<Html>() }
                            </div>
                        </div>
                    </div>
                }
            }

        } else {
            return html! {
                <div class="container">
                    <div class="uk-position-center uk-padding">
                        <span uk-spinner="ratio: 4.5"></span>
                    </div>
                </div>
            }
        }
    }
}

impl MainView {
    fn get_event(&self, event: String) -> Html {
        html! {
            <p>{event}</p>
        }
    }

    fn get_room(&self, room: SmallRoom) -> Html {
        // TODO better linking than onlclick (yew limitation?)

        let room_id = room.clone().id.to_string();
        html! {
            <li><a href="#" onclick=self.link.callback(move |e: MouseEvent| Msg::ChangeRoom(room_id.clone()))>{room.name.clone()}</a></li>
        }
    }
}
