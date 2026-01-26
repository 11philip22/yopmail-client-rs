pub mod client;
pub mod constants;
pub mod error;
pub mod models;

pub use client::{
    check_inbox, check_inbox_page, check_inbox_page_with, check_inbox_with, generate_random_mailbox,
    get_inbox_count, get_inbox_count_page, get_inbox_count_page_with, get_inbox_count_with,
    get_inbox_summary, get_inbox_summary_page, get_inbox_summary_page_with, get_inbox_summary_with,
    get_last_message, get_last_message_content, get_last_message_content_with, get_last_message_with,
    get_message_by_id, get_message_by_id_full, get_message_by_id_full_with, get_message_by_id_with,
    get_rss_feed_data, get_rss_feed_data_with, get_rss_feed_url, get_rss_feed_url_with,
    YopmailClient, YopmailClientBuilder,
};
pub use constants::{
    default_headers, default_timeout, ALT_DOMAINS, DEFAULT_DOMAIN, DEFAULT_HEADERS,
    DEFAULT_TIMEOUT_SECS,
};
pub use error::Error;
pub use models::{Attachment, Message, MessageContent, RssItem};
