use matrix_sdk::{events::room::message::MessageEventContent, js_int::UInt, Room};
use rand::random;
use yew::prelude::*;

use crate::app::components::events::{get_sender_avatar, get_sender_displayname, is_new_user};
use crate::app::matrix::types::get_media_download_url;
use url::Url;

pub(crate) struct RoomItem {
    props: Props,
    link: ComponentLink<Self>,
}

pub enum Msg {
    ChangeRoom(Room),
}

#[derive(Clone, Properties, Debug)]
pub struct Props {
    #[prop_or_default]
    pub room: Option<Room>,

    #[prop_or_default]
    pub change_room_callback: Callback<Room>,
}

impl Component for RoomItem {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        RoomItem { props, link }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::ChangeRoom(room) => {
                self.props.change_room_callback.emit(room);
            }
        }
        false
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        // TODO fix the PartialEq hack
        if format!("{:#?}", self.props) != format!("{:#?}", props) {
            self.props = props;
            true
        } else {
            false
        }
    }

    //noinspection RsTypeCheck
    fn view(&self) -> Html {
        let room = self.props.room.clone().unwrap();

        // TODO placeholder for encrypted rooms
        let last_message = match self
            .props
            .room
            .as_ref()
            .unwrap()
            .messages
            .clone()
            .into_iter()
            .last()
        {
            None => "".to_string(),
            Some(m) => {
                let content = m.content.clone();
                if let MessageEventContent::Text(text_event) = content {
                    text_event.body
                } else {
                    "".to_string()
                }
            }
        };
        html! {
            <div class="room-list-item">
                <a onclick=self.link.callback(move |e: MouseEvent| Msg::ChangeRoom(room.clone()))>
                    <div class="content">
                        // TODO remove placeholder
                        <img class="avatar" src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACgAAAAoCAYAAACM/rhtAAAARUlEQVRYhe3OMQ0AIADAMBKUowmBoIKMo0f/jrnX+dmoA4KCdUBQsA4ICtYBQcE6IChYBwQF64CgYB0QFKwDgoJ1QPC1C8gY0kSgNLTWAAAAAElFTkSuQmCC"/>
                        <div>
                            <h5 class="name">{self.props.room.as_ref().unwrap().display_name()}</h5>
                            <p class="latest-msg">{last_message}</p>
                        </div>
                    </div>
                </a>
            </div>
        }
    }
}
