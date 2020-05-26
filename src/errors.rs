use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum MatrixError {
    #[error("No Matrix Client is available yet")]
    MissingClient,
    #[error("Missing required Data")]
    MissingFields,
}
