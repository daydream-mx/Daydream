use matrix_sdk::{
    events::room::message::{MessageEvent, NoticeMessageEventContent},
    Room,
};
use yew::prelude::*;

use crate::app::components::events::{get_sender_displayname, is_new_user};

pub(crate) struct Notice {
    props: Props,
}

#[derive(Clone, Properties, Debug)]
pub struct Props {
    #[prop_or_default]
    pub prev_event: Option<MessageEvent>,
    #[prop_or_default]
    pub event: Option<MessageEvent>,
    #[prop_or_default]
    pub notice_event: Option<NoticeMessageEventContent>,
    #[prop_or_default]
    pub room: Option<Room>,
}

impl Component for Notice {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Notice { props }
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

        if new_user {
            html! {
                <p style="opacity: .6;"><displayname>{sender_displayname}{": "}</displayname>
                    {
                        self.props
                            .notice_event
                            .clone()
                            .unwrap().body.clone()
                    }
                </p>
            }
        } else {
            html! {
                <p style="opacity: .6;">
                    {
                        self.props
                            .notice_event
                            .clone()
                            .unwrap().body.clone()
                    }
                </p>
            }
        }
    }
}
