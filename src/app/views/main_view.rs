use log::*;
use serde_derive::{Deserialize, Serialize};
use yew::prelude::*;
use yew::ComponentLink;

use crate::app::matrix::{MatrixAgent, Response};

pub struct MainView {
    link: ComponentLink<Self>,
    state: State,
    matrix_agent: Box<dyn Bridge<MatrixAgent>>,
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
        let matrix_callback = link.callback(Msg::NewMessage);
        let matrix_agent = MatrixAgent::bridge(matrix_callback);
        let state = State {};

        MainView {
            link,
            matrix_agent,
            state,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::NewMessage(response) => {
                info!("NewMessage: {:#?}", response);
                match response {
                    Response::LoggedIn(v) => {
                        info!("client_logged_in: {}", v);
                    },
                    _ => {}
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
