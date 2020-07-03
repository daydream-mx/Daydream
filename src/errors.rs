use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Field {
    Homeserver,
    MXID,
    Password,
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Field::Homeserver => write!(f, "Homeserver Field"),
            Field::MXID => write!(f, "MXID Field"),
            Field::Password => write!(f, "Password Field"),
        }
    }
}

// TODO figure out a way to translate this
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum MatrixError {
    #[error("No Matrix Client is available yet")]
    MissingClient,
    #[error("Missing required Data in {0}")]
    MissingFields(Field),

    /// An error occurred in the Matrix client library.
    /// This can't use transparent as we need Clone
    #[error("A Timeout happened on Login")]
    LoginTimeout,

    /// An error occurred in the Matrix client library.
    /// This can't use transparent as we need Clone
    #[error("An error occurred in the Matrix client library: `{0}`")]
    SDKError(String),
}
