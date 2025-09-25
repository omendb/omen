//! Error types for learned index operations

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Model training failed: {0}")]
    TrainingFailed(String),

    #[error("Key not found")]
    KeyNotFound,

    #[error("Index error: {0}")]
    IndexError(String),
}

pub type Result<T> = std::result::Result<T, Error>;