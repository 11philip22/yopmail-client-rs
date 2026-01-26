use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub id: String,
    pub subject: String,
    pub sender: Option<String>,
    pub date: Option<String>,
    pub time: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RssItem {
    pub subject: String,
    pub sender: String,
    pub date: String,
    pub url: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Attachment {
    pub name: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageContent {
    pub text: String,
    pub html: String,
    pub raw: String,
    pub attachments: Vec<Attachment>,
}
