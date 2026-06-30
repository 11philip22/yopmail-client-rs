//! HTTP and protocol constants used to emulate the YOPmail web UI.
//!
//! Most constants here exist only to support the internal request/parse logic.
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::time::Duration;

/// Default base URL used by [`YopmailClient`](crate::YopmailClient).
pub const BASE_URL: &str = "https://yopmail.com";
/// Default domain used when building full YOPmail addresses.
pub const DEFAULT_DOMAIN: &str = "yopmail.com";
/// Version parameter used by the YOPmail inbox/mail endpoints.
pub(crate) const VERSION: &str = "9.3";
/// "ad" query parameter used by the YOPmail inbox/mail endpoints.
pub(crate) const AD_PARAM: i32 = 0;
/// Default timeout in seconds used by [`default_timeout`].
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Build the default HTTP headers used by this client.
///
/// This converts [`DEFAULT_HEADERS`] into a `reqwest::header::HeaderMap`, skipping any values that
/// fail to parse.
pub(crate) fn default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    for (k, v) in DEFAULT_HEADERS {
        let name = HeaderName::from_static(k);
        if let Ok(val) = HeaderValue::from_str(v) {
            headers.insert(name, val);
        }
    }
    headers
}

/// Default header pairs used by [`default_headers`].
pub(crate) const DEFAULT_HEADERS: &[(&str, &str)] = &[
    (
        "user-agent",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    ),
    (
        "accept",
        "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
    ),
    ("accept-language", "en-US,en;q=0.5"),
    ("accept-encoding", "gzip, deflate"),
    ("connection", "keep-alive"),
    ("upgrade-insecure-requests", "1"),
];

/// Extra headers used for inbox listing requests.
pub(crate) const INBOX_HEADERS: &[(&str, &str)] = &[
    ("referer", "https://yopmail.com/en/wm"),
    ("sec-fetch-dest", "iframe"),
    ("sec-fetch-mode", "navigate"),
    ("sec-fetch-site", "same-origin"),
];

/// Extra headers used for mail fetch requests.
pub(crate) const MAIL_HEADERS: &[(&str, &str)] = &[
    ("referer", "https://yopmail.com/en/wm"),
    ("sec-fetch-dest", "iframe"),
    ("sec-fetch-mode", "navigate"),
    ("sec-fetch-site", "same-origin"),
    ("sec-fetch-user", "?1"),
    ("upgrade-insecure-requests", "1"),
];

/// Extra headers used for message sending requests.
pub(crate) const SEND_HEADERS: &[(&str, &str)] = &[
    ("content-type", "application/x-www-form-urlencoded"),
    ("origin", "https://yopmail.com"),
    ("referer", "https://yopmail.com/wm"),
    ("accept", "*/*"),
    ("accept-language", "en-US,en;q=0.9,de-DE;q=0.8,de;q=0.7"),
    ("accept-encoding", "gzip, deflate, br, zstd"),
    ("sec-fetch-dest", "empty"),
    ("sec-fetch-mode", "cors"),
    ("sec-fetch-site", "same-origin"),
    ("priority", "u=1, i"),
];

/// Fallback `yp` token used when the login page does not contain an extractable value.
pub(crate) const FALLBACK_YP_TOKEN: &str = "ZAGplZmp0ZmR3ZQN4ZGx1ZGR";

/// Default request timeout used by the client.
pub fn default_timeout() -> Duration {
    Duration::from_secs(DEFAULT_TIMEOUT_SECS)
}
