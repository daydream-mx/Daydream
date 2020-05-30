use std::convert::TryFrom;

use linked_hash_set::LinkedHashSet;
use log::*;
use matrix_sdk::identifiers::RoomId;
use serde::{Deserialize, Serialize};
use yew::prelude::*;
use yew::ComponentLink;

use crate::app::components::{event_list::EventList, room_list::RoomList};
use crate::app::matrix::types::MessageWrapper;
use crate::app::matrix::{MatrixAgent, Request, Response};

pub struct MainView {
    link: ComponentLink<Self>,
    state: State,
    matrix_agent: Box<dyn Bridge<MatrixAgent>>,
}

pub enum Msg {
    NewMessage(Response),
    ChangeRoom((String, String)),
}

#[derive(Serialize, Deserialize, Default)]
pub struct State {
    // TODO handle all events
    pub events: LinkedHashSet<MessageWrapper>,
    pub current_room: Option<RoomId>,
    pub current_room_displayname: String,
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
            current_room: None,
            current_room_displayname: Default::default(),
        };

        MainView {
            link,
            matrix_agent,
            state,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::NewMessage(response) => match response {
                Response::FinishedFirstSync => {
                    //self.matrix_agent.send(Request::GetJoinedRooms);
                }
                _ => {}
            },
            Msg::ChangeRoom((displayname, room)) => {
                let room_id = RoomId::try_from(room).unwrap();

                self.state.current_room = Some(room_id.clone());
                self.state.current_room_displayname = displayname;
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        if self.state.current_room.is_none() {
            return html! {
                <div class="uk-flex auto-scrollable-container uk-background-default" style="height: 100%">
                    <RoomList change_room_callback=self.link.callback(Msg::ChangeRoom)/>

                    <div class="container uk-flex uk-width-5-6 uk-padding">
                        <div class="scrollable">
                            // TODO add some content to the empty page
                        </div>
                    </div>
                </div>
            };
        } else {
            return html! {
                <div class="uk-flex auto-scrollable-container" style="height: 100%">
                    <RoomList change_room_callback=self.link.callback(Msg::ChangeRoom)/>
                    <EventList current_room=self.state.current_room.clone() displayname=self.state.current_room_displayname.clone() />
                </div>
            };
        }
    }
}
