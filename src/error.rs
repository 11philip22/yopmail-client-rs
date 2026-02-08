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
    /// An I/O error.
    ///
    /// Note: this crate does not currently perform file I/O. This variant exists so applications
    /// using [`crate::Result`] can conveniently bubble up `std::io::Error` values alongside client
    /// errors.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// A network-related error surfaced by higher-level logic.
    ///
    /// Note: the current implementation does not construct this variant.
    #[error("network error: {0}")]
    Network(String),
    /// A parsing error (typically HTML/RSS parsing of YOPmail responses).
    ///
    /// Note: the current implementation does not construct this variant; parsing is currently
    /// best-effort and typically results in empty/placeholder values rather than an error.
    #[error("parse error: {0}")]
    Parse(String),
    /// An authentication/session-related error.
    #[error("authentication error: {0}")]
    Auth(String),
    /// The recipient address is not accepted by the current implementation.
    #[error("invalid recipient domain")]
    InvalidRecipient,
    /// The requested operation is not supported by this implementation.
    ///
    /// Note: the current implementation does not construct this variant.
    #[error("unsupported operation: {0}")]
    Unsupported(String),
    /// The server returned a non-success HTTP status.
    ///
    /// The `body` field contains the response body captured as text (when available).
    #[error("unexpected status {status}: {body}")]
    Status { status: StatusCode, body: String },
}

/// Convenience result type used by this crate.
pub type Result<T> = std::result::Result<T, Error>;
