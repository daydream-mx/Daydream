use matrix_sdk::events::room::message::MessageEvent;
use yew::prelude::*;

struct Video {
    props: Props,
}

#[derive(Clone, Properties, Debug)]
pub struct Props {
    #[prop_or_default]
    pub prev_event: Option<MessageEvent>,
    #[prop_or_default]
    pub event: Option<MessageEvent>,
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

    fn change(&mut self, props: Self::Properties) -> bool {
        // TODO fix the PartialEq hack
        if format!("{:#?}", self.props) != format!("{:#?}", props) {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        html! {}
    }
}
