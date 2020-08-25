use std::rc::Rc;

use crate::utils::extensions::{EventExt, RoomExt};
use matrix_sdk::{
    events::{room::message::ImageMessageEventContent, AnySyncMessageEvent},
    Room,
};
use rand::random;
use yew::prelude::*;

pub(crate) struct Image {
    props: Props,
}

#[derive(Clone, Properties, Debug)]
pub struct Props {
    #[prop_or_default]
    pub prev_event: Option<AnySyncMessageEvent>,
    pub event: AnySyncMessageEvent,
    pub image_event: ImageMessageEventContent,
    pub room: Rc<Room>,
}

impl Component for Image {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Image { props }
    }

    fn update(&mut self, _msg: Self::Message) -> bool {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        // TODO fix the PartialEq hack
        /*if format!("{:?}", self.props) != format!("{:?}", props) {
            self.props = props;
            true
        } else {
            false
        }*/
        true
    }

    //noinspection RsTypeCheck
    fn view(&self) -> Html {
        let new_user = self.props.event.is_new_user(self.props.prev_event.as_ref());
        let sender_displayname = if new_user {
            self.props.room.get_sender_displayname(&self.props.event)
        } else {
            ""
        };

        if let Some(image_url) = &self.props.image_event.url {
            let thumbnail = self
                .props
                .image_event
                .info
                .as_ref()
                .unwrap()
                .thumbnail_url
                .as_ref()
                .unwrap_or(image_url);

            let lightbox_id: u8 = random();
            let lightbox_id_full = format!("image_{}", lightbox_id);
            let lightbox_href_full = format!("#image_{}", lightbox_id);
            if new_user {
                html! {
                    <div>
                        <p><displayname>{sender_displayname}{": "}</displayname></p>
                        <a href=lightbox_href_full><div class="thumbnail-container"><img src=thumbnail/></div></a>
                        <div class="lightbox short-animate" id=lightbox_id_full>
                            <img class="long-animate" src=image_url/>
                        </div>
                        <div id="lightbox-controls" class="short-animate">
                            <a id="close-lightbox" class="long-animate" href="#!">{"Close Lightbox"}</a>
                        </div>
                    </div>
                }
            } else {
                html! {
                    <div>
                        <a href=lightbox_href_full><img src=thumbnail/></a>
                        <div class="lightbox short-animate" id=lightbox_id_full>
                            <img class="long-animate" src=image_url/>
                        </div>
                        <div id="lightbox-controls" class="short-animate">
                            <a id="close-lightbox" class="long-animate" href="#!">{"Close Lightbox"}</a>
                        </div>
                    </div>
                }
            }
        } else {
            html! {}
        }
    }
}
