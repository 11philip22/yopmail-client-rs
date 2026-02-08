//! Unofficial async client for [YOPmail](https://yopmail.com).
//!
//! `yopmail-client` drives the same HTTP endpoints as the YOPmail web UI to:
//! - list inbox messages
//! - fetch message content (text/HTML/raw) and attachments
//! - generate and fetch RSS feeds
//! - send messages (currently only to `@yopmail.com`)
//!
//! This crate is not affiliated with YOPmail. The YOPmail site is HTML-driven and may change at any
//! time; parsing is best-effort and may break without warning.
//!
//! **Quick start**
//! ```rust,no_run
//! use yopmail_client::{generate_random_mailbox, YopmailClient};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), yopmail_client::Error> {
//!     // A mailbox can be provided as `local` or `local@domain`.
//!     // Generating a random mailbox helps avoid collisions in shared inbox namespaces.
//!     let mailbox = generate_random_mailbox(12);
//!     let mut client = YopmailClient::new(&mailbox)?;
//!
//!     // Establish a session (cookies + `yp` token). Most methods do this lazily, but calling it
//!     // explicitly makes the stateful behavior obvious.
//!     client.open_inbox().await?;
//!
//!     // Page numbers are passed through to YOPmail as-is (typically page 1 is the newest).
//!     let messages = client.list_messages(1).await?;
//!     if let Some(msg) = messages.first() {
//!         // Use the `Message::id` returned by `list_messages`.
//!         let content = client.fetch_message_full(&msg.id).await?;
//!         println!("{}", content.text);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! **Notes for experienced users**
//! - The client is stateful: it keeps a cookie jar and an extracted `yp` token. Methods take
//!   `&mut self` because the session state is updated on demand.
//! - `Message::date` is currently always `None` because the inbox HTML parser does not extract it.
//! - Network calls are performed with `reqwest`; non-2xx HTTP responses are typically surfaced as
//!   [`Error::Status`], with the response body captured for debugging.
//! - There is no built-in polling loop. Convenience methods like
//!   [`YopmailClient::check_inbox`](crate::YopmailClient::check_inbox) perform a single request; if
//!   you want to poll, implement a loop with `tokio::time::sleep`.
//!
//! **Error behavior**
//! - Transport errors, TLS failures, DNS issues, and timeouts are returned as [`Error::Http`]
//!   (`reqwest::Error`). The request timeout is configured via
//!   [`YopmailClientBuilder::timeout`](crate::YopmailClientBuilder::timeout).
//! - Some methods explicitly validate HTTP status codes and return [`Error::Status`] for non-2xx
//!   responses (for example [`YopmailClient::list_messages`](crate::YopmailClient::list_messages),
//!   [`YopmailClient::fetch_message_full`](crate::YopmailClient::fetch_message_full),
//!   [`YopmailClient::send_message`](crate::YopmailClient::send_message), and
//!   [`YopmailClient::download_attachment`](crate::YopmailClient::download_attachment)).
//! - Other methods currently do not check `StatusCode` and may return `Ok(...)` even if the server
//!   responded with a non-2xx status (notably [`YopmailClient::open_inbox`](crate::YopmailClient::open_inbox)
//!   and [`YopmailClient::get_rss_feed_data`](crate::YopmailClient::get_rss_feed_data)). If you need
//!   strict status handling, wrap the crate or open an issue.
//! - `fetch_message_full` has a limited retry mechanism: on HTTP 400 it retries the mail fetch with
//!   a few different message ID encodings. There is no network retry/backoff logic.
//! - External system failures are expected: YOPmail HTML/RSS structure can change, endpoints can be
//!   rate-limited or blocked, and responses may include unexpected markup. These typically surface
//!   as [`Error::Status`], [`Error::Http`], or best-effort empty parses depending on the endpoint.
pub mod client;
pub mod constants;
pub mod error;
pub mod models;

pub use client::{generate_random_mailbox, YopmailClient, YopmailClientBuilder};
pub use constants::{
    default_headers, default_timeout, ALT_DOMAINS, DEFAULT_DOMAIN, DEFAULT_HEADERS,
    DEFAULT_TIMEOUT_SECS,
};
pub use error::Error;
pub use models::{Attachment, Message, MessageContent, RssItem};
