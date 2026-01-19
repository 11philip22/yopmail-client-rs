use serde::Serialize;
use std::time::Duration;

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

#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: String,
    pub timeout: Duration,
    pub proxy_url: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_url: crate::constants::BASE_URL.to_string(),
            timeout: crate::constants::default_timeout(),
            proxy_url: None,
        }
    }
}

pub fn default_config() -> Config {
    Config::default()
}
