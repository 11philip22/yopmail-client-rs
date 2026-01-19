pub mod client;
pub mod constants;
pub mod error;
pub mod models;

pub use client::{check_inbox, get_inbox_count, get_inbox_summary, get_last_message, get_last_message_content, get_message_by_id, get_rss_feed_data, get_rss_feed_url, YopmailClient};
pub use constants::{default_headers, default_timeout, DEFAULT_HEADERS, DEFAULT_TIMEOUT_SECS};
pub use error::Error;
pub use models::{default_config, Config, Message, RssItem};
