use std::rc::Rc;

use matrix_sdk::{events::room::message::MessageEventContent, identifiers::RoomId, Room};
use yew::prelude::*;
use yewtil::NeqAssign;

pub(crate) struct RoomItem {
    props: Props,
    link: ComponentLink<Self>,
}

pub enum Msg {
    ChangeRoom(Rc<Room>),
}

#[derive(Clone, Properties, Debug, PartialEq)]
pub struct Props {
    pub room: Rc<Room>,

    #[prop_or_default]
    pub change_room_callback: Callback<RoomId>,
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
                self.props.change_room_callback.emit(room.room_id.clone());
            }
        }
        false
    }

    fn change(&mut self, props: Self::Properties) -> bool {
        self.props.neq_assign(props)
    }

    //noinspection RsTypeCheck
    fn view(&self) -> Html {
        let room = self.props.room.clone();

        // TODO placeholder for encrypted rooms
        let last_message = match room.messages.iter().last() {
            None => "".to_string(),
            Some(m) => {
                if let MessageEventContent::Text(text_event) = m.content.clone() {
                    text_event.body.clone()
                } else {
                    "".to_string()
                }
            }
        };

        let display_name = room.display_name();

        html! {
            <div class="room-list-item">
                <a onclick=self.link.callback(move |e: MouseEvent| Msg::ChangeRoom(room.clone()))>
                    <div class="content">
                        // TODO remove placeholder
                        <img class="avatar" src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACgAAAAoCAYAAACM/rhtAAAARUlEQVRYhe3OMQ0AIADAMBKUowmBoIKMo0f/jrnX+dmoA4KCdUBQsA4ICtYBQcE6IChYBwQF64CgYB0QFKwDgoJ1QPC1C8gY0kSgNLTWAAAAAElFTkSuQmCC"/>
                        <div>
                            <h5 class="name">{display_name}</h5>
                            <p class="latest-msg">{last_message}</p>
                        </div>
                    </div>
                </a>
            </div>
        }
    }
}
