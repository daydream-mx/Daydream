use crate::app::components::events::{get_sender_displayname, is_new_user};
use matrix_sdk::{
    events::room::message::{MessageEvent, TextMessageEventContent},
    Room,
};
use web_sys::Node;
use yew::prelude::*;
use yew::virtual_dom::VNode;

pub struct Text {
    props: Props,
}

#[derive(Clone, Properties, Debug)]
pub struct Props {
    #[prop_or_default]
    pub prev_event: Option<MessageEvent>,
    #[prop_or_default]
    pub event: Option<MessageEvent>,
    #[prop_or_default]
    pub text_event: Option<TextMessageEventContent>,
    #[prop_or_default]
    pub room: Option<Room>,
}

impl Component for Text {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Text { props }
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
        if self.props.text_event.clone().unwrap().formatted.is_some() {
            let message = if new_user {
                format!(
                    "<displayname>{}:</displayname> {}",
                    sender_displayname,
                    self.props
                        .text_event
                        .clone()
                        .unwrap()
                        .formatted
                        .unwrap()
                        .body
                )
            } else {
                self.props
                    .text_event
                    .clone()
                    .unwrap()
                    .formatted
                    .unwrap()
                    .body
            };
            let js_text_event = {
                let div = web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .create_element("p")
                    .unwrap();
                div.set_inner_html(message.as_str());
                div
            };
            let node = Node::from(js_text_event);
            VNode::VRef(node)
        } else if new_user {
            html! {
               <p><displayname>{sender_displayname}{": "}</displayname>{self.props.text_event.clone().unwrap().body.clone()}</p>
            }
        } else {
            html! {
               <p>{self.props.text_event.clone().unwrap().body.clone()}</p>
            }
        }
    }
}
