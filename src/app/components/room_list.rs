use std::collections::HashMap;
use std::include_str;

use matrix_sdk::{identifiers::RoomId, js_int::UInt, Room};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use yew::prelude::*;
use yew::utils::document;
use yew::{Bridge, Bridged, Component, ComponentLink, Html};
use yewtil::NeqAssign;

use tr::tr;

use crate::app::components::raw_html::RawHTML;
use crate::app::matrix::{MatrixAgent, Request, Response};

pub struct RoomList {
    link: ComponentLink<Self>,
    state: State,
    matrix_agent: Box<dyn Bridge<MatrixAgent>>,
    props: Props,
}

#[allow(clippy::large_enum_variant)]
pub enum Msg {
    NewMessage(Response),
    ChangeRoom(Room),
    SetFilter(String),
    ToggleTheme,
}

#[derive(Serialize, Deserialize, Default)]
pub struct State {
    rooms: HashMap<RoomId, Room>,
    current_room: Option<Room>,
    loading: bool,
    search_query: Option<String>,
    dark_theme: bool,
    did_fetch: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub change_room_callback: Callback<Room>,
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
            did_fetch: false,
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
                    self.state.did_fetch = false;
                    true
                }
                // Better Initial Sync Detection
                Response::SyncPing => {
                    if self.state.rooms.is_empty() && !self.state.did_fetch {
                        self.matrix_agent.send(Request::GetJoinedRooms);
                        self.state.did_fetch = true;
                    }
                    false
                }
                // Handle new rooms from sync
                Response::Sync((room_id, _msg)) => {
                    if !(self.state.rooms.contains_key(&room_id)) && !self.state.did_fetch {
                        self.matrix_agent.send(Request::GetJoinedRoom(room_id));
                    }
                    false
                }
                Response::JoinedRoom((room_id, room)) => {
                    self.state.rooms.insert(room_id, room);
                    true
                }
                _ => false,
            },
            Msg::ChangeRoom(room) => {
                if self.state.current_room.is_some() {
                    if self.state.current_room.as_ref().unwrap() == &room {
                        return false;
                    }
                }
                self.props.change_room_callback.emit(room.clone());
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

    //noinspection RsTypeCheck
    fn view(&self) -> Html {
        if self.state.loading {
            html! {
                <div class="container">
                    <div class="uk-position-center uk-padding">
                        <RawHTML inner_html=include_str!("../svgs/loading_animation.svg")/>
                    </div>
                </div>
            }
        } else {
            html! {
                <div class="container roomlist uk-flex uk-flex-column uk-width-1-6" style="height: 100%">
                    <div class="uk-padding uk-padding-remove-bottom" style="height: 50px">
                        <form class="uk-search uk-search-default">
                            <span class="material-icons" id ="ma-icon">{"search"}</span>
                            <input
                                class="uk-search-input"
                                type="search"
                                placeholder={
                                    tr!(
                                        // Placeholder text for the roomlist filtering
                                        "Filter Rooms..."
                                    )
                                }
                                value=&self.state.search_query.as_ref().unwrap_or(&"".to_string())
                                oninput=self.link.callback(|e: InputData| Msg::SetFilter(e.value)) />
                        </form>
                    </div>
                    <ul class="scrollable uk-flex uk-flex-column uk-padding uk-nav-default uk-nav-parent-icon uk-padding-remove-bottom" uk-nav="">
                        <li class="uk-nav-header">
                            {
                                tr!(
                                    // Header of the Roomlist
                                    "Rooms"
                                )
                            }
                        </li>
                        {
                            if self.state.search_query.is_none() || (self.state.search_query.as_ref().unwrap_or(&"".to_string()) == &"".to_string()) {
                                self.state.rooms.iter().map(|(_, room)| self.get_room(room)).collect::<Html>()
                            } else {
                                self.state.rooms.iter().filter(|(_, room)| room.display_name().to_lowercase().contains(&self.state.search_query.as_ref().unwrap().to_lowercase())).map(|(_, room)| self.get_room(room)).collect::<Html>()
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
            }
        }
    }
}

impl RoomList {
    fn get_room(&self, matrix_room: &Room) -> Html {
        let classes = if self.state.current_room.is_some() {
            if self.state.current_room.as_ref().unwrap().room_id == matrix_room.room_id {
                "uk-active"
            } else {
                ""
            }
        } else {
            ""
        };

        let room = matrix_room.clone();
        html! {
            <li class=classes>
                <a onclick=self.link.callback(move |e: MouseEvent| Msg::ChangeRoom(room.clone()))>
                    {matrix_room.display_name()}
                    {
                        if matrix_room.unread_notifications.is_some() && matrix_room.unread_notifications.unwrap() != UInt::from(0u32) {
                            html! { <span class="uk-badge uk-margin-small-left">{matrix_room.unread_notifications.unwrap()}</span> }
                        } else {
                            html! {}
                        }
                    }
                    {
                        if matrix_room.unread_highlight.is_some() && matrix_room.unread_highlight.unwrap() != UInt::from(0u32) {
                            html! { <span class="uk-badge red uk-margin-small-left">{matrix_room.unread_highlight.unwrap()}</span> }
                        } else {
                            html! {}
                        }
                    }
                </a>
            </li>
        }
    }
}
