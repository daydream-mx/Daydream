use std::collections::HashMap;

use log::*;
use matrix_sdk::{identifiers::RoomId, js_int::UInt};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use yew::prelude::*;
use yew::{Bridge, Bridged, Component, ComponentLink, Html};
use yewtil::NeqAssign;

use crate::app::matrix::types::SmallRoom;
use crate::app::matrix::{MatrixAgent, Request, Response};
use yew::utils::document;

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
    ToggleTheme,
}

#[derive(Serialize, Deserialize, Default)]
pub struct State {
    rooms: HashMap<RoomId, SmallRoom>,
    current_room: Option<String>,
    loading: bool,
    search_query: Option<String>,
    dark_theme: bool,
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
            search_query: None,
            dark_theme: false,
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
                // Handle new rooms from sync
                Response::Sync(msg) => {
                    // TODO properly do this...
                    if self.state.rooms.is_empty() {
                        self.matrix_agent.send(Request::GetJoinedRooms);
                    }
                    if !(self
                        .state
                        .rooms
                        .keys()
                        .map(|x| x.clone())
                        .collect::<Vec<RoomId>>()
                        .contains(&msg.room_id.clone().unwrap()))
                    {
                        self.matrix_agent
                            .send(Request::GetJoinedRoom(msg.room_id.clone().unwrap()));
                    }
                    true
                }
                Response::JoinedRoom((room_id, room)) => {
                    self.state.rooms.insert(room_id, room);
                    true
                }
                _ => false,
            },
            Msg::ChangeRoom(room) => {
                let displayname = self
                    .state
                    .rooms
                    .iter()
                    .filter(|(id, _)| **id == room)
                    .map(|(_, room)| room.name.clone())
                    .collect::<String>();
                self.props
                    .change_room_callback
                    .emit((displayname, room.clone()));
                self.state.current_room = Some(room);
                true
            }
            Msg::SetFilter(query) => {
                self.state.search_query = Some(query);
                true
            }
            Msg::ToggleTheme => {
                self.state.dark_theme = !self.state.dark_theme;
                let theme = if self.state.dark_theme {
                    "dark"
                } else {
                    "light"
                };
                document()
                    .document_element()
                    .unwrap()
                    .dyn_into::<HtmlElement>()
                    .unwrap()
                    .dataset()
                    .set("theme", theme)
                    .unwrap();
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
                    <div class="uk-padding uk-padding-remove-bottom" style="height: 50px">
                        <form class="uk-search uk-search-default">
                            <span uk-search-icon=""></span>
                            <input
                                class="uk-search-input"
                                type="search"
                                placeholder="Filter Rooms..."
                                value=&self.state.search_query.clone().unwrap_or("".to_string())
                                oninput=self.link.callback(|e: InputData| Msg::SetFilter(e.value)) />
                        </form>
                    </div>
                    <ul class="scrollable uk-flex uk-flex-column uk-padding uk-nav-default uk-nav-parent-icon uk-padding-remove-bottom" uk-nav="">
                        <li class="uk-nav-header">{"Rooms"}</li>
                        {
                            if self.state.search_query.is_none() || (self.state.search_query.clone().unwrap_or("".to_string()) == "".to_string()) {
                                self.state.rooms.iter().map(|(_, room)| self.get_room(room.clone())).collect::<Html>()
                            } else {
                                self.state.rooms.iter().filter(|(_, room)| room.name.to_lowercase().contains(&self.state.search_query.clone().unwrap().to_lowercase())).map(|(_, room)| self.get_room(room.clone())).collect::<Html>()
                            }
                        }
                    </ul>
                    <div class="toggleWrapper uk-margin uk-flex uk-flex-column uk-flex-center" style="height: 60px;">
                        <input type="checkbox" class="dn" id="dn" checked=self.state.dark_theme value=self.state.dark_theme onclick=self.link.callback(|e: MouseEvent| {Msg::ToggleTheme})/>
                        <label for="dn" class="toggle">
                            <span class="toggle__handler">
                                <span class="crater crater--1"></span>
                                <span class="crater crater--2"></span>
                                <span class="crater crater--3"></span>
                            </span>
                            <span class="star star--1"></span>
                            <span class="star star--2"></span>
                            <span class="star star--3"></span>
                            <span class="star star--4"></span>
                            <span class="star star--5"></span>
                            //<span class="star star--6"></span>
                        </label>
                    </div>
                </div>
            };
        }
    }
}

impl RoomList {
    fn get_room(&self, room: SmallRoom) -> Html {
        // TODO better linking than onlclick (yew limitation?)

        let classes = if self.state.current_room.clone().is_some() {
            if self.state.current_room.clone().unwrap() == room.id.to_string() {
                "uk-active"
            } else {
                ""
            }
        } else {
            ""
        };

        let room_id = room.clone().id.to_string();
        html! {
            <li class=classes>
                <a onclick=self.link.callback(move |e: MouseEvent| Msg::ChangeRoom(room_id.clone()))>
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
