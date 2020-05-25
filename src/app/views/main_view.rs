use log::*;
use serde_derive::{Deserialize, Serialize};
use yew::agent::{Dispatched, Dispatcher};
use yew::prelude::*;
use yew::services::storage::Area;
use yew::services::StorageService;
use yew::ComponentLink;

use crate::app::matrix::{MatrixAgent, Response};

pub struct MainView {
    link: ComponentLink<Self>,
    storage: StorageService,
    matrix_agent: Dispatcher<MatrixAgent>,
    state: State,
    _producer: Box<dyn Bridge<MatrixAgent>>,
}

pub enum Msg {
    NewMessage(Response),
}

#[derive(Serialize, Deserialize)]
pub struct State {}

impl Component for MainView {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local).unwrap();
        let matrix_agent = MatrixAgent::dispatcher();
        let matrix_callback = link.callback(Msg::NewMessage);
        let _producer = MatrixAgent::bridge(matrix_callback);
        let state = State {};

        MainView {
            link,
            storage,
            matrix_agent,
            state,
            _producer,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::NewMessage(response) => {
                info!("NewMessage: {:#?}", response);
                if response.message == "client_logged_in" {
                    info!("client_logged_in: {}", response.content);
                }
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        info!("rendered MainView!");
        html! {
        <p>{"Test"}</p>
        }
    }
}
