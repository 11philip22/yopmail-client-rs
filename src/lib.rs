pub mod client;
pub mod constants;
pub mod error;
pub mod models;

pub use client::{
    generate_random_mailbox, YopmailClient, YopmailClientBuilder,
};
pub use constants::{
    default_headers, default_timeout, ALT_DOMAINS, DEFAULT_DOMAIN, DEFAULT_HEADERS,
    DEFAULT_TIMEOUT_SECS,
};
pub use error::Error;
pub use models::{Attachment, Message, MessageContent, RssItem};
