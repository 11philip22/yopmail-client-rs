pub mod client;
pub mod constants;
pub mod error;
pub mod models;

pub use client::{
    check_inbox, check_inbox_page, generate_random_mailbox, get_inbox_count, get_inbox_count_page,
    get_inbox_summary, get_inbox_summary_page, get_last_message, get_last_message_content,
    get_message_by_id, get_message_by_id_full, get_rss_feed_data, get_rss_feed_url, YopmailClient,
};
pub use constants::{default_headers, default_timeout, DEFAULT_HEADERS, DEFAULT_TIMEOUT_SECS};
pub use error::Error;
pub use models::{default_config, Attachment, Config, Message, MessageContent, RssItem};
