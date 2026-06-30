//! Error types for this crate.
use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
/// Errors returned by this crate.
pub enum Error {
    /// A `reqwest` error (request creation, transport error, or body decoding).
    ///
    /// Timeouts are reported through this variant as well. If you need to detect timeouts, inspect
    /// the inner `reqwest::Error` (for example `err.is_timeout()`).
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    /// A parsing error (typically missing or malformed YOPmail response data).
    #[error("parse error: {0}")]
    Parse(String),
    /// An authentication/session-related error.
    #[error("authentication error: {0}")]
    Auth(String),
    /// The recipient address is not accepted by the current implementation.
    #[error("invalid recipient domain")]
    InvalidRecipient,
    /// The server returned a non-success HTTP status.
    ///
    /// The `body` field contains the response body captured as text (when available).
    #[error("unexpected status {status}: {body}")]
    Status { status: StatusCode, body: String },
}

/// Convenience result type used by this crate.
pub type Result<T> = std::result::Result<T, Error>;
