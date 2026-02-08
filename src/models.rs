//! Data models returned by YOPmail operations.
//!
//! These types intentionally keep fields as simple strings and `Option<String>` values to reflect
//! the underlying HTML/RSS sources. Fields may be absent or change based on YOPmail's markup.
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
/// A message summary parsed from the inbox HTML.
///
/// This is typically produced by [`YopmailClient::list_messages`](crate::YopmailClient::list_messages).
pub struct Message {
    /// Message identifier as found in the inbox DOM (for example `me_...`).
    ///
    /// This value is suitable for passing to [`YopmailClient::fetch_message_full`](crate::YopmailClient::fetch_message_full).
    pub id: String,
    /// Subject line as displayed in the inbox.
    pub subject: String,
    /// Sender display value as displayed in the inbox (may be missing).
    pub sender: Option<String>,
    /// Date as parsed from the inbox HTML.
    ///
    /// Note: the current inbox parser does not extract a date, so this is always `None`.
    pub date: Option<String>,
    /// Time as displayed in the inbox (may be missing).
    pub time: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
/// An item parsed from the mailbox RSS feed.
pub struct RssItem {
    /// Subject/title of the RSS item.
    pub subject: String,
    /// Sender inferred from the item's description (best-effort).
    pub sender: String,
    /// Publication date string from the RSS feed.
    pub date: String,
    /// Link to the message as provided by the RSS feed.
    pub url: String,
    /// Raw description text (when present).
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
/// An attachment link extracted from a message view.
pub struct Attachment {
    /// Best-effort attachment name (from link title/text), if available.
    pub name: Option<String>,
    /// Download URL for the attachment (absolute or resolved against the client's base URL).
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
/// Full content of a fetched message.
pub struct MessageContent {
    /// Best-effort plain text extracted from the message HTML with whitespace normalized.
    pub text: String,
    /// Best-effort HTML extracted from the message container.
    pub html: String,
    /// Raw HTML response body returned by the server.
    pub raw: String,
    /// Best-effort list of unique attachment links found in the message.
    pub attachments: Vec<Attachment>,
}
