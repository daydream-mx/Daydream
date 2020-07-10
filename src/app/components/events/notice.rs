use std::rc::Rc;

use linkify::LinkFinder;
use matrix_sdk::{
    events::{room::message::NoticeMessageEventContent, AnyMessageEventStub},
    Room,
};
use web_sys::Node;
use yew::prelude::*;
use yew::virtual_dom::VNode;

use crate::app::components::events::{RoomExt, EventExt};

pub(crate) struct Notice {
    props: Props,
}

#[derive(Clone, Properties, Debug)]
pub struct Props {
    #[prop_or_default]
    pub prev_event: Option<AnyMessageEventStub>,
    pub event: AnyMessageEventStub,
    pub notice_event: NoticeMessageEventContent,
    pub room: Rc<Room>,
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

        let mut pure_content = self.props.notice_event.body.clone();
        let finder = LinkFinder::new();
        let pure_content_clone = pure_content.clone();
        let links: Vec<_> = finder.links(&pure_content_clone).collect();

        if !links.is_empty() {
            for link in links {
                let html_link = format!("<a href={}>{}</a>", link.as_str(), link.as_str());
                pure_content.replace_range(link.start()..link.end(), &html_link);
            }
        }

        if new_user {
            let full_html = format!(
                "<p style=\"opacity: .6;\"><displayname>{}: </displayname>{}</p>",
                sender_displayname, pure_content
            );
            let js_text_event = {
                let div = web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .create_element("p")
                    .unwrap();
                div.set_inner_html(full_html.as_str());
                div
            };
            let node = Node::from(js_text_event);
            VNode::VRef(node)
        } else {
            let full_html = format!("<p style=\"opacity: .6;\">{}</p>", pure_content);
            let js_text_event = {
                let div = web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .create_element("p")
                    .unwrap();
                div.set_inner_html(full_html.as_str());
                div
            };
            let node = Node::from(js_text_event);
            VNode::VRef(node)
        }
    }
}
