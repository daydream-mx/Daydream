use log::*;
use serde_derive::{Deserialize, Serialize};
use yew::prelude::*;
use yew::ComponentLink;

use crate::app::matrix::{MatrixAgent, Request, Response};
use wasm_bindgen::__rt::std::collections::{HashMap, HashSet};

pub struct MainView {
    link: ComponentLink<Self>,
    state: State,
    matrix_agent: Box<dyn Bridge<MatrixAgent>>,
}

pub enum Msg {
    NewMessage(Response),
}

#[derive(Serialize, Deserialize, Default)]
pub struct State {
    // TODO handle all events
    pub events: HashSet<String>,
}

impl Component for MainView {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let matrix_callback = link.callback(Msg::NewMessage);
        let mut matrix_agent = MatrixAgent::bridge(matrix_callback);
        matrix_agent.send(Request::StartSync);
        let state = State {
            events: Default::default(),
        };

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
                    }
                    Response::Sync(msg) => {
                        // TODO handle all events
                        self.state.events.insert(msg);
                    }
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
            <div class="container h-100">
            { self.state.events.iter().map(|event| self.get_event(event)).collect::<Html>() }
            </div>
        }
    }
}

impl MainView {
    fn get_event(&self, event: &String) -> Html {
        html! {
            <><p>{event}</p><br/></>
        }
    }
}
