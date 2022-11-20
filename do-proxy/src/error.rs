use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A simple error type for do-proxy.
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum Error {
    #[error("json: {0}")]
    Json(String),
    #[error("worker: {0}")]
    Worker(String),
    #[error("expected object response")]
    ExpectedObjectResponse,
    #[error("expected object response")]
    ExpectedObjectInitialized,
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err.to_string())
    }
}

impl From<worker::Error> for Error {
    fn from(err: worker::Error) -> Self {
        Error::Worker(err.to_string())
    }
}

impl From<Error> for worker::Error {
    fn from(err: Error) -> Self {
        worker::Error::from(err.to_string())
    }
}

/// An enum of either a [`crate::Error`] or a user provided error, usually a [`crate::DoProxy::Error`].
#[derive(Debug, Error)]
pub enum CrateOrObjectError<ObjectError> {
    Crate(#[from] Error),
    Object(ObjectError),
}

impl<ObjectError: std::error::Error> From<CrateOrObjectError<ObjectError>> for worker::Error {
    fn from(err: CrateOrObjectError<ObjectError>) -> Self {
        match err {
            CrateOrObjectError::Crate(err) => err.into(),
            CrateOrObjectError::Object(err) => worker::Error::from(err.to_string()),
        }
    }
}
