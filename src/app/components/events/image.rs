use crate::app::components::events::{get_sender_displayname, is_new_user};
use matrix_sdk::{
    events::room::message::{ImageMessageEventContent, MessageEvent},
    Room,
};
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
    pub room: Option<Room>,
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
        let new_user = is_new_user(
            self.props.prev_event.clone(),
            self.props.event.clone().unwrap(),
        );
        let sender_displayname = if new_user {
            get_sender_displayname(
                self.props.room.clone().unwrap(),
                self.props.event.clone().unwrap(),
            )
        } else {
            "".to_string()
        };

        let caption = format!(
            "{}: {}",
            sender_displayname,
            self.props.image_event.as_ref().unwrap().body
        );
        if self
            .props
            .image_event
            .as_ref()
            .unwrap()
            .url
            .clone()
            .is_some()
        {
            let image_url = self
                .props
                .image_event
                .as_ref()
                .unwrap()
                .url
                .clone()
                .unwrap();
            let thumbnail = match self
                .props
                .image_event
                .as_ref()
                .unwrap()
                .info
                .clone()
                .unwrap()
                .thumbnail_url
            {
                None => image_url.clone(),
                Some(v) => v,
            };
            if new_user {
                html! {
                    <>
                        <p><displayname>{sender_displayname}{": "}</displayname></p>
                        <div uk-lightbox="">
                            <a class="uk-inline" href=image_url data-caption=caption >
                                <img src=thumbnail alt=caption />
                            </a>
                       </div>
                    </>
                }
            } else {
                html! {
                   <div uk-lightbox="">
                        <a class="uk-inline" href=image_url data-caption=caption >
                            <img src=thumbnail alt=caption />
                        </a>
                   </div>
                }
            }
        } else {
            html! {}
        }
    }
}
