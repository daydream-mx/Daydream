use std::rc::Rc;

use crate::app::components::events::{get_sender_displayname, is_new_user};
use matrix_sdk::{
    events::room::message::{ImageMessageEventContent, MessageEvent},
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
    pub prev_event: Option<MessageEvent>,
    #[prop_or_default]
    pub event: Option<MessageEvent>,
    #[prop_or_default]
    pub image_event: Option<ImageMessageEventContent>,
    #[prop_or_default]
    pub room: Option<Rc<Room>>,
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
        let new_user = is_new_user(
            self.props.prev_event.as_ref(),
            self.props.event.as_ref().unwrap(),
        );
        let sender_displayname = if new_user {
            get_sender_displayname(
                self.props.room.as_ref().unwrap(),
                self.props.event.as_ref().unwrap(),
            )
        } else {
            "".to_string()
        };

        if let Some(image_url) = &self.props.image_event.as_ref().unwrap().url {
            let thumbnail = self
                .props
                .image_event
                .as_ref()
                .unwrap()
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
                        <a href={lightbox_href_full.clone()}><img src=thumbnail/></a>
                        <div class="lightbox short-animate" id={lightbox_id_full.clone()}>
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
                        <a href={lightbox_href_full.clone()}><img src=thumbnail/></a>
                        <div class="lightbox short-animate" id={lightbox_id_full.clone()}>
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
