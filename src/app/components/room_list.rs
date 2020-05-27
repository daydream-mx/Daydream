use std::collections::HashMap;

use js_int::UInt;
use log::*;
use matrix_sdk::identifiers::RoomId;
use serde::{Deserialize, Serialize};
use yew::prelude::*;
use yew::{Bridge, Bridged, Component, ComponentLink, Html};
use yewtil::NeqAssign;

use crate::app::matrix::types::SmallRoom;
use crate::app::matrix::{MatrixAgent, Response};

pub struct RoomList {
    link: ComponentLink<Self>,
    state: State,
    matrix_agent: Box<dyn Bridge<MatrixAgent>>,
    props: Props,
}

pub enum Msg {
    NewMessage(Response),
    ChangeRoom(String),
    SetFilter(String),
}

#[derive(Serialize, Deserialize, Default)]
pub struct State {
    rooms: HashMap<RoomId, SmallRoom>,
    current_room: Option<RoomId>,
    loading: bool,
    search_query: Option<String>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub change_room_callback: Callback<(String, String)>,
}

impl Component for RoomList {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let matrix_callback = link.callback(Msg::NewMessage);
        let matrix_agent = MatrixAgent::bridge(matrix_callback);
        let state = State {
            rooms: Default::default(),
            current_room: None,
            loading: true,
            search_query: None
        };

        RoomList {
            props,
            link,
            matrix_agent,
            state,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::NewMessage(response) => match response {
                Response::JoinedRoomList(rooms) => {
                    self.state.rooms = rooms;
                    self.state.loading = false;
                    true
                }
                _ => false,
            },
            Msg::ChangeRoom(room) => {
                let displayname = self.state.rooms.iter().filter(|(id, _)| **id == room).map(|(_, room)| room.name.clone()).collect::<String>();
                self.props.change_room_callback.emit((displayname, room));
                false
            }
            Msg::SetFilter(query) => {
                self.state.search_query = Some(query);
                true
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        if self.state.loading {
            return html! {
                <div class="container">
                    <div class="uk-position-center uk-padding">
                        <span uk-spinner="ratio: 4.5"></span>
                    </div>
                </div>
            };
        } else {
            return html! {
                <div class="container uk-flex uk-flex-column uk-width-1-6" style="height: 100%">
                    <div class="uk-padding uk-padding-remove-bottom">
                        <form class="uk-search uk-search-default">
                            <span uk-search-icon=""></span>
                            <input
                                class="uk-search-input"
                                type="search"
                                placeholder="Filter Rooms..."
                                oninput=self.link.callback(|e: InputData| Msg::SetFilter(e.value)) />
                        </form>
                    </div>
                    <ul class="scrollable uk-flex uk-flex-column uk-padding uk-nav-default uk-nav-parent-icon" uk-nav="" style="height: 100%">
                        <li class="uk-nav-header">{"Rooms"}</li>
                        {
                            if self.state.search_query.is_none() || (self.state.search_query.clone().unwrap_or("".to_string()) == "".to_string()) {
                                self.state.rooms.iter().map(|(_, room)| self.get_room(room.clone())).collect::<Html>()
                            } else {
                                self.state.rooms.iter().filter(|(_, room)| room.name.contains(&self.state.search_query.clone().unwrap())).map(|(_, room)| self.get_room(room.clone())).collect::<Html>()
                            }
                        }
                    </ul>
                </div>
            };
        }
    }
}

impl RoomList {
    fn get_room(&self, room: SmallRoom) -> Html {
        // TODO better linking than onlclick (yew limitation?)

        let room_id = room.clone().id.to_string();
        html! {
            <li>
                <a href="#" onclick=self.link.callback(move |e: MouseEvent| Msg::ChangeRoom(room_id.clone()))>
                    {room.name.clone()}
                    {
                        if room.unread_notifications.is_some() && room.unread_notifications.unwrap() != UInt::from(0u32) {
                            html! { <span class="uk-badge uk-margin-small-left">{room.unread_notifications.unwrap()}</span> }
                        } else {
                            html! {}
                        }
                    }
                    {
                        if room.unread_highlight.is_some() && room.unread_highlight.unwrap() != UInt::from(0u32) {
                            html! { <span class="uk-badge red uk-margin-small-left">{room.unread_highlight.unwrap()}</span> }
                        } else {
                            html! {}
                        }
                    }
                </a>
            </li>
        }
    }
}
