//! Public data models returned by the client.

use serde::Serialize;

/// Summary information for a mailbox message.
#[derive(Debug, Clone, Serialize)]
pub struct Message {
    /// Yopmail message identifier.
    pub id: String,
    /// Message subject line.
    pub subject: String,
    /// Sender address if available.
    pub sender: Option<String>,
    /// Message date (as provided by Yopmail).
    pub date: Option<String>,
    /// Message time (as provided by Yopmail).
    pub time: Option<String>,
}

/// RSS item parsed from a Yopmail feed.
#[derive(Debug, Clone, Serialize)]
pub struct RssItem {
    /// Item subject/title.
    pub subject: String,
    /// Sender address parsed from the description.
    pub sender: String,
    /// Publication date string.
    pub date: String,
    /// Link to the message.
    pub url: String,
    /// Raw description HTML/text.
    pub description: Option<String>,
}

/// Attachment metadata from a message.
#[derive(Debug, Clone, Serialize)]
pub struct Attachment {
    /// Attachment display name if available.
    pub name: Option<String>,
    /// Absolute URL to download the attachment.
    pub url: String,
}

/// Full message content including text, HTML, and attachments.
#[derive(Debug, Clone, Serialize)]
pub struct MessageContent {
    /// Plaintext message content.
    pub text: String,
    /// HTML message content.
    pub html: String,
    /// Raw message response body.
    pub raw: String,
    /// Attachments found in the message.
    pub attachments: Vec<Attachment>,
}
