//! Error types for the Yopmail client.

use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
/// Error type for all Yopmail client operations.
pub enum Error {
    /// Underlying HTTP client error.
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    /// IO error when reading or writing data.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// Generic network error with a message.
    #[error("network error: {0}")]
    Network(String),
    /// Parse error with a message.
    #[error("parse error: {0}")]
    Parse(String),
    /// Authentication or authorization failure.
    #[error("authentication error: {0}")]
    Auth(String),
    /// Recipient domain is not allowed.
    #[error("invalid recipient domain")]
    InvalidRecipient,
    /// Operation is not supported by the client.
    #[error("unsupported operation: {0}")]
    Unsupported(String),
    /// HTTP response returned a non-success status with body.
    #[error("unexpected status {status}: {body}")]
    Status { status: StatusCode, body: String },
}

/// Result type for Yopmail client operations.
pub type Result<T> = std::result::Result<T, Error>;
