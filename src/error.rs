use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("network error: {0}")]
    Network(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("authentication error: {0}")]
    Auth(String),
    #[error("invalid recipient domain")]
    InvalidRecipient,
    #[error("unsupported operation: {0}")]
    Unsupported(String),
    #[error("unexpected status {status}: {body}")]
    Status { status: StatusCode, body: String },
}

pub type Result<T> = std::result::Result<T, Error>;
