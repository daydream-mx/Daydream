use log::*;
use matrix_sdk::Room;
use serde::{Deserialize, Serialize};
use yew::ComponentLink;
use yew::prelude::*;

use crate::app::components::{event_list::EventList, room_list::RoomList};
use crate::app::matrix::{MatrixAgent, Request};

pub struct MainView {
    link: ComponentLink<Self>,
    state: State,
}

pub enum Msg {
    ChangeRoom(Room),
}

#[derive(Serialize, Deserialize, Default)]
pub struct State {
    pub current_room: Option<Room>,
    pub current_room_displayname: String,
}

impl Component for MainView {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut matrix_agent = MatrixAgent::dispatcher();
        matrix_agent.send(Request::StartSync);
        let state = State {
            current_room: None,
            current_room_displayname: Default::default(),
        };

        MainView { link, state }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::ChangeRoom(room) => {
                info!("Changing room to: {}", room.room_id);
                self.state.current_room = Some(room);
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    //noinspection RsTypeCheck
    fn view(&self) -> Html {
        if self.state.current_room.is_none() {
            html! {
                <div class="uk-flex auto-scrollable-container uk-background-default" style="height: 100%">
                    <RoomList change_room_callback=self.link.callback(Msg::ChangeRoom)/>

                    <div class="container uk-flex uk-width-5-6 uk-padding">
                        <div class="scrollable">
                            // TODO add some content to the empty page
                        </div>
                    </div>
                </div>
            }
        } else {
            if self.state.current_room.as_ref().unwrap().is_encrypted() {
                html! {
                    <div class="uk-flex auto-scrollable-container" style="height: 100%">
                        <RoomList change_room_callback=self.link.callback(Msg::ChangeRoom)/>
                        <div class="container uk-flex uk-flex-column uk-width-5-6 uk-padding uk-padding-remove-bottom" style="height: 100%">
                            <h1>{ self.state.current_room.as_ref().unwrap().display_name() }</h1>
                            <h4>{ "Daydream currently does not support encryption." }</h4>
                        </div>
                    </div>
                }
            } else {
                html! {
                    <div class="uk-flex auto-scrollable-container" style="height: 100%">
                        <RoomList change_room_callback=self.link.callback(Msg::ChangeRoom)/>
                        <EventList current_room=self.state.current_room.as_ref().unwrap() />
                    </div>
                }
            }
        }
    }
}
