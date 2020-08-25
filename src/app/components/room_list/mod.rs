use std::{collections::HashMap, rc::Rc};

use log::*;
use matrix_sdk::{identifiers::RoomId, Room};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use yew::prelude::*;
use yew::utils::document;
use yew::{Bridge, Bridged, Component, ComponentLink, Html};
use yewtil::NeqAssign;

use tr::tr;

use crate::app::components::raw_html::RawHTML;
use crate::app::components::room_list::item::RoomItem;
use crate::matrix::{MatrixAgent, Request, Response};

mod item;

pub struct RoomList {
    link: ComponentLink<Self>,
    state: State,
    matrix_agent: Box<dyn Bridge<MatrixAgent>>,
    props: Props,
}

#[allow(clippy::large_enum_variant)]
pub enum Msg {
    NewMessage(Response),
    ChangeRoom(RoomId),
    SetFilter(String),
    ToggleTheme,
}

#[derive(Serialize, Deserialize, Default)]
pub struct State {
    rooms: HashMap<RoomId, Rc<Room>>,
    current_room: Option<RoomId>,
    loading: bool,
    search_query: Option<String>,
    dark_theme: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub change_room_callback: Callback<Rc<Room>>,
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
                // Handle new rooms from sync
                Response::JoinedRoomSync(room_id) => {
                    info!("Got JoinedRoomSync");
                    if !(self.state.rooms.contains_key(&room_id)) {
                        self.matrix_agent.send(Request::GetJoinedRoom(room_id));
                    }
                    false
                }
                Response::JoinedRoom((room_id, room)) => {
                    info!("Got JoinedRoom");
                    self.state.rooms.insert(room_id, Rc::new(room));
                    if self.state.loading {
                        self.state.loading = false;
                    }
                    true
                }
                _ => false,
            },
            Msg::ChangeRoom(room_id) => {
                if self.state.current_room.is_some()
                    && self.state.current_room.as_ref().unwrap() == &room_id
                {
                    return false;
                }

                let room = self.state.rooms[&room_id].clone();
                self.props.change_room_callback.emit(room);
                self.state.current_room = Some(room_id);
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
                        <RawHTML inner_html=include_str!("../../svgs/loading_animation.svg")/>
                    </div>
                </div>
            }
        } else {
            let rooms: Html = match self.state.search_query.as_deref() {
                None | Some("") => self
                    .state
                    .rooms
                    .iter()
                    .map(|(_, room)| self.get_room(room))
                    .collect(),
                _ => self
                    .state
                    .rooms
                    .iter()
                    .filter(|(_, room)| {
                        room.display_name()
                            .to_lowercase()
                            .contains(&self.state.search_query.as_ref().unwrap().to_lowercase())
                    })
                    .map(|(_, room)| self.get_room(room))
                    .collect(),
            };

            html! {
                <div class="roomlist" style="height: 100%">
                    <div class="top-bar">
                        <div class="userdata">
                        </div>
                        <div class="search">
                            <div>
                                <span class="material-icons">{"search"}</span>
                                <input
                                    class="search-input"
                                    type="search"
                                    placeholder={
                                        tr!(
                                            // Placeholder text for the roomlist filtering
                                            "Filter Rooms..."
                                        )
                                    }
                                    value=self.state.search_query.as_deref().unwrap_or("")
                                    oninput=self.link.callback(|e: InputData| Msg::SetFilter(e.value)) />
                            </div>
                        </div>
                    </div>
                    <div class="scrollable list">{rooms}</div>
                    <div class="bottom-bar">
                        <div class="toggleWrapper">
                            <div>
                                <input type="checkbox" class="dn" id="dn" checked=self.state.dark_theme value=self.state.dark_theme onclick=self.link.callback(|_e: MouseEvent| {Msg::ToggleTheme})/>
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
                    </div>
                </div>
            }
        }
    }
}

impl RoomList {
    fn get_room(&self, matrix_room: &Rc<Room>) -> Html {
        let room = matrix_room.clone();
        html! {
            <RoomItem change_room_callback=self.link.callback(Msg::ChangeRoom) room=room />
        }
    }
}
