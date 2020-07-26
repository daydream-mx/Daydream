use crate::errors::{Field, MatrixError};
use log::*;
use matrix_sdk::{Client, ClientConfig, Session};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use url::Url;
use wasm_bindgen_futures::spawn_local;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct SessionStore {
    pub(crate) access_token: String,
    pub(crate) user_id: String,
    pub(crate) device_id: String,
    pub(crate) homeserver_url: String,
}

pub fn login(
    session: Option<&SessionStore>,
    homeserver: Option<&String>,
) -> Result<Client, MatrixError> {
    info!("preparing client");
    match session {
        Some(session) => restore_client(session),
        None => match homeserver {
            Some(homeserver) => {
                let homeserver = Url::parse(&homeserver);
                match homeserver {
                    Ok(homeserver) => {
                        let client_config = ClientConfig::new();
                        let client = Client::new_with_config(homeserver, client_config).unwrap();
                        Ok(client)
                    }
                    Err(e) => Err(MatrixError::UrlParseError(e.to_string())),
                }
            }
            None => Err(MatrixError::MissingFields(Field::Homeserver)),
        },
    }
}

fn restore_client(session: &SessionStore) -> Result<Client, MatrixError> {
    let homeserver = Url::parse(&session.homeserver_url);
    match homeserver {
        Ok(homeserver) => {
            let client_config = ClientConfig::new();
            let client = Client::new_with_config(homeserver, client_config);
            match client {
                Ok(client) => {
                    info!("got client");
                    // Also directly restore Login data
                    let session = Session {
                        access_token: session.access_token.clone(),
                        user_id: matrix_sdk::identifiers::UserId::try_from(
                            session.user_id.as_str(),
                        )
                        .unwrap(),
                        device_id: session.device_id.clone().into(),
                    };
                    info!("before restore");
                    let cloned_client = client.clone();
                    spawn_local(async move {
                        if let Err(e) = cloned_client.restore_login(session).await {
                            error!("{}", e);
                            // TODO find a way to get this back up in the function tree
                        }
                    });
                    info!("after restore");
                    Ok(client)
                }
                Err(e) => Err(MatrixError::SDKError(e.to_string())),
            }
        }
        Err(e) => Err(MatrixError::UrlParseError(e.to_string())),
    }
}
