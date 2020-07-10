use std::rc::Rc;

use crate::app::components::events::{RoomExt, EventExt};
use matrix_sdk::{
    events::{room::message::VideoMessageEventContent, AnyMessageEventStub},
    Room,
};
use rand::random;
use yew::prelude::*;

pub(crate) struct Video {
    props: Props,
}

#[derive(Clone, Properties, Debug)]
pub struct Props {
    #[prop_or_default]
    pub prev_event: Option<AnyMessageEventStub>,
    pub event: AnyMessageEventStub,
    pub video_event: VideoMessageEventContent,
    pub room: Rc<Room>,
}

impl Component for Video {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Video { props }
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

        let _caption = format!("{}: {}", sender_displayname, self.props.video_event.body);

        if let Some(video_url) = self.props.video_event.url.as_ref() {
            let thumbnail = self
                .props
                .video_event
                .info
                .as_ref()
                .unwrap()
                .thumbnail_url
                .as_ref()
                .unwrap_or(video_url);

            let lightbox_id: u8 = random();
            let lightbox_id_full = format!("video_{}", lightbox_id);
            let lightbox_href_full = format!("#video_{}", lightbox_id);
            if new_user {
                html! {
                    <div>
                        <p><displayname>{sender_displayname}{": "}</displayname></p>
                        <a href={lightbox_href_full}><img src=thumbnail/></a>
                        <div class="lightbox short-animate" id={lightbox_id_full}>
                            <video class="long-animate" controls=true>
                              <source src=video_url type="video/mp4"/>
                            {"Your browser does not support the video tag."}
                            </video>
                        </div>
                        <div id="lightbox-controls" class="short-animate">
                            <a id="close-lightbox" class="long-animate" href="#!">{"Close Lightbox"}</a>
                        </div>
                    </div>
                }
            } else {
                html! {
                    <div>
                        <a href={lightbox_href_full}><img src=thumbnail/></a>
                        <div class="lightbox short-animate" id={lightbox_id_full}>
                            <video class="long-animate" controls=true>
                              <source src=video_url type="video/mp4"/>
                            {"Your browser does not support the video tag."}
                            </video>
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
