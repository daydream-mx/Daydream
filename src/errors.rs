use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum MatrixError {
    #[error("No Matrix Client is available yet")]
    MissingClient,
    #[error("Missing required Data")]
    MissingFields,
    #[error("This Matrix Event is not yet supported by Daydream")]
    UnsupportedEvent,
}
