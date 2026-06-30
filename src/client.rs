//! High-level async client for interacting with YOPmail via its web endpoints.
//!
//! The client is intentionally "browser-like": it maintains a cookie jar, sends default headers,
//! and extracts a `yp` token from the login page. Most operations will automatically call
//! [`YopmailClient::open_inbox`] the first time they need a session.
use crate::constants::*;
use crate::error::{Error, Result};
use crate::models::{Attachment, Message, MessageContent};
use rand::{Rng, distributions::Alphanumeric};
use regex::Regex;
use reqwest::{
    Client, ClientBuilder, StatusCode,
    cookie::Jar,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use scraper::{ElementRef, Html, Selector};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn normalize_mailbox(mailbox: &str) -> Result<String> {
    if mailbox.contains('@') {
        return Err(Error::InvalidRecipient);
    }
    Ok(mailbox.trim().to_lowercase())
}

fn build_headers(base: &[(&str, &str)], extras: &[(&str, &str)]) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for (k, v) in base.iter().chain(extras.iter()) {
        if let Ok(name) = HeaderName::from_bytes(k.as_bytes()) {
            if let Ok(val) = HeaderValue::from_str(v) {
                headers.insert(name, val);
            }
        }
    }
    headers
}

/// A stateful YOPmail client backed by `reqwest`.
///
/// The client maintains a cookie jar (to emulate the web UI) and an extracted `yp` token used by
/// inbox/mail endpoints. Methods take `&mut self` because session state is refreshed on demand.
pub struct YopmailClient {
    mailbox: String,
    base_url: String,
    jar: Arc<Jar>,
    client: Client,
    yp_token: Option<String>,
    yj_token: Option<String>,
    version: Option<String>,
}

/// Builder for [`YopmailClient`].
///
/// This configures the underlying HTTP client (base URL, timeout, optional proxy) and constructs a
/// `reqwest::Client` with a shared cookie jar.
pub struct YopmailClientBuilder {
    mailbox: String,
    base_url: String,
    timeout: Duration,
    proxy_url: Option<String>,
}

impl YopmailClientBuilder {
    /// Create a builder for `mailbox`.
    ///
    /// The mailbox is the local part only, trimmed and lowercased.
    pub fn new(mailbox: impl AsRef<str>) -> Self {
        Self {
            mailbox: mailbox.as_ref().to_string(),
            base_url: BASE_URL.to_string(),
            timeout: default_timeout(),
            proxy_url: None,
        }
    }

    /// Override the base URL (defaults to [`BASE_URL`]).
    ///
    /// This is primarily useful for testing or for alternative frontends that mimic YOPmail's
    /// endpoints.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Override the request timeout (defaults to [`default_timeout`]).
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Configure an HTTP proxy URL for the underlying `reqwest` client.
    ///
    /// The value is passed to `reqwest::Proxy::all`.
    pub fn proxy_url(mut self, proxy_url: impl Into<String>) -> Self {
        self.proxy_url = Some(proxy_url.into());
        self
    }

    /// Build the [`YopmailClient`].
    ///
    /// This parses and normalizes the mailbox, builds a `reqwest::Client` with cookies enabled,
    /// and initializes session state (no network requests are made).
    pub fn build(self) -> Result<YopmailClient> {
        let mailbox = normalize_mailbox(&self.mailbox)?;
        let jar = Arc::new(Jar::default());

        let mut builder = ClientBuilder::new()
            .cookie_provider(jar.clone())
            .timeout(self.timeout)
            .default_headers(default_headers());

        if let Some(proxy) = &self.proxy_url {
            builder = builder.proxy(reqwest::Proxy::all(proxy).map_err(Error::Http)?);
        }

        let client = builder.build().map_err(Error::Http)?;

        Ok(YopmailClient {
            mailbox,
            base_url: self.base_url,
            jar,
            client,
            yp_token: None,
            yj_token: None,
            version: None,
        })
    }
}

impl YopmailClient {
    /// Start building a client for `mailbox`.
    ///
    /// Equivalent to [`YopmailClientBuilder::new`].
    pub fn builder(mailbox: impl AsRef<str>) -> YopmailClientBuilder {
        YopmailClientBuilder::new(mailbox)
    }

    /// Construct a client for `mailbox` with default settings.
    ///
    /// This does not perform any network requests. Most operations will implicitly call
    /// [`open_inbox`](Self::open_inbox) on first use.
    pub fn new(mailbox: impl AsRef<str>) -> Result<Self> {
        YopmailClientBuilder::new(mailbox).build()
    }

    /// Initialize the session by performing the same flow as the YOPmail web UI.
    ///
    /// This method:
    /// - sets default cookies (including `ytime` and `ywm`)
    /// - loads the login page to extract a `yp` token (falling back to an internal default)
    /// - posts the login form to establish the session cookies
    ///
    /// This is a best-effort initialization. The method does not validate the login response;
    /// subsequent operations may still fail with [`Error::Status`].
    ///
    /// # Errors
    /// - Returns [`Error::Http`] for transport failures and timeouts.
    /// - This method does not currently validate HTTP status codes, so a non-2xx response may not
    ///   be reported as [`Error::Status`].
    pub async fn open_inbox(&mut self) -> Result<()> {
        self.set_default_cookies();

        // Use the same flow as the web UI: load the login page (with yp token) then post the form
        let login_url = format!("{}/en/?login={}", self.base_url, self.mailbox);
        let resp = self.client.get(&login_url).send().await?;
        let body = resp.text().await?;

        self.yp_token = extract_yp_token(&body);
        if self.yp_token.is_none() {
            self.yp_token = Some(FALLBACK_YP_TOKEN.to_string());
        }

        // Submit the login form to establish the session (mirrors the hidden auto-submit form)
        if let Some(ref yp) = self.yp_token {
            let form = [
                ("login", self.mailbox.clone()),
                ("id", String::new()),
                ("yp", yp.clone()),
            ];
            let resp = self
                .client
                .post(format!("{}/en/", self.base_url))
                .headers(default_headers())
                .form(&form)
                .send()
                .await?;
            let body = resp.text().await?;
            self.version = extract_webmail_version(&body);
            self.yj_token = Some(self.fetch_yj_token(&body).await?);
        }
        Ok(())
    }

    /// List messages in the inbox for `page`.
    ///
    /// If the session is not yet initialized, this calls [`open_inbox`](Self::open_inbox)
    /// automatically.
    ///
    /// The returned [`Message`] values are derived from the YOPmail inbox HTML.
    ///
    /// # Errors
    /// - Returns [`Error::Http`] for transport failures and timeouts.
    /// - Returns [`Error::Status`] if the inbox endpoint responds with a non-success HTTP status.
    pub async fn list_messages(&mut self, page: i32) -> Result<Vec<Message>> {
        if self.yp_token.is_none() || self.yj_token.is_none() {
            self.open_inbox().await?;
        }

        let yp = self
            .yp_token
            .clone()
            .unwrap_or_else(|| FALLBACK_YP_TOKEN.to_string());
        let yj = self
            .yj_token
            .clone()
            .ok_or_else(|| Error::Parse("missing yj token".into()))?;
        let version = self.version.clone().unwrap_or_else(|| VERSION.to_string());

        let params = [
            ("login", self.mailbox.as_str()),
            ("p", &page.to_string()),
            ("d", ""),
            ("ctrl", ""),
            ("yp", yp.as_str()),
            ("yj", yj.as_str()),
            ("v", version.as_str()),
            ("r_c", ""),
            ("id", ""),
            ("ad", &AD_PARAM.to_string()),
        ];

        let headers = build_headers(DEFAULT_HEADERS, INBOX_HEADERS);
        let url = format!("{}/en/inbox", self.base_url);
        let resp = self
            .client
            .get(&url)
            .headers(headers)
            .query(&params)
            .send()
            .await?;
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(Error::Status { status, body });
        }

        let messages = parse_messages(&body);
        Ok(messages)
    }

    /// Fetch the text body of a message.
    ///
    /// This is a convenience wrapper around [`fetch_message_full`](Self::fetch_message_full) that
    /// returns only [`MessageContent::text`].
    pub async fn fetch_message(&mut self, message_id: &str) -> Result<String> {
        let content = self.fetch_message_full(message_id).await?;
        Ok(content.text)
    }

    /// Fetch a message and return parsed text/HTML, raw HTML, and attachment links.
    ///
    /// `message_id` should typically be the [`Message::id`] returned by [`Self::list_messages`].
    /// The implementation accepts several ID variants and will retry (on HTTP 400) using multiple
    /// common encodings used by YOPmail (for example `m_...`, `me_...`, and `e_...` prefixes).
    ///
    /// On success:
    /// - `text` is extracted and whitespace-normalized
    /// - `html` is extracted from the message container (best-effort)
    /// - `raw` is the full HTML response body
    /// - `attachments` is a best-effort list of unique download URLs
    ///
    /// # Retry behavior
    /// This method may retry the mail fetch on HTTP 400 by attempting a few different message ID
    /// encodings. It does not perform transport-level retries or backoff.
    ///
    /// # Errors
    /// - Returns [`Error::Http`] for transport failures and timeouts.
    /// - Returns [`Error::Status`] when all ID variants fail (the captured body is from the last
    ///   attempt).
    pub async fn fetch_message_full(&mut self, message_id: &str) -> Result<MessageContent> {
        if self.yp_token.is_none() {
            self.open_inbox().await?;
        }

        // Refresh cookies (ytime/ywm) just before the mail fetch
        self.set_default_cookies();

        let headers = build_headers(DEFAULT_HEADERS, MAIL_HEADERS);
        let mail_url = format!("{}/en/mail", self.base_url);
        let raw_id = message_id.trim();
        let main_id = if raw_id.starts_with('m') {
            raw_id.to_string()
        } else if raw_id.starts_with("e_") {
            format!("m{}", raw_id)
        } else {
            format!("m_{}", raw_id.trim_start_matches("m_"))
        };
        let alt_id =
            if raw_id.starts_with("e_") || raw_id.starts_with("me_") || raw_id.starts_with("m_") {
                raw_id.to_string()
            } else {
                format!("e_{raw_id}")
            };

        let variants = [main_id, alt_id, raw_id.to_string()];

        let mut last_status = None;
        let mut last_body = None;
        for id in variants {
            let params = [("b", self.mailbox.as_str()), ("id", id.as_str())];

            let resp = self
                .client
                .get(&mail_url)
                .headers(headers.clone())
                .query(&params)
                .send()
                .await?;
            let status = resp.status();
            let body = resp.text().await?;
            if status.is_success() {
                let attachments = extract_attachments(&body, &self.base_url);
                let html = extract_message_html(&body);
                return Ok(MessageContent {
                    text: extract_message_body(&body),
                    html,
                    raw: body,
                    attachments,
                });
            }

            last_status = Some(status);
            last_body = Some(body);
            if status != StatusCode::BAD_REQUEST {
                break;
            }
        }

        Err(Error::Status {
            status: last_status.unwrap_or(StatusCode::BAD_REQUEST),
            body: last_body.unwrap_or_else(|| "mail fetch failed".into()),
        })
    }

    /// Send a message from this mailbox.
    ///
    /// The recipient must be a YOPmail address ending with `@yopmail.com`, otherwise
    /// [`Error::InvalidRecipient`] is returned.
    ///
    /// Delivery and success are determined by a simple, case-insensitive substring check over the
    /// response body (for example `sent successfully` or `ok|`). If the HTTP request succeeds but
    /// no success marker is found, this returns [`Error::Auth`].
    ///
    /// # Errors
    /// - Returns [`Error::InvalidRecipient`] if `to` is not accepted by the current implementation.
    /// - Returns [`Error::Http`] for transport failures and timeouts.
    /// - Returns [`Error::Status`] if the write endpoint responds with a non-success HTTP status.
    /// - Returns [`Error::Auth`] if the response body does not contain a recognized success marker.
    pub async fn send_message(&mut self, to: &str, subject: &str, body: &str) -> Result<()> {
        if !to.ends_with("@yopmail.com") {
            return Err(Error::InvalidRecipient);
        }
        if self.yp_token.is_none() {
            self.open_inbox().await?;
        }

        let form = [
            ("msgfrom", format!("{}@{}", self.mailbox, DEFAULT_DOMAIN)),
            ("msgto", to.to_string()),
            ("msgsubject", subject.to_string()),
            ("msgbody", body.to_string()),
        ];

        let headers = build_headers(DEFAULT_HEADERS, SEND_HEADERS);
        let url = format!("{}/writepost", self.base_url);
        let resp = self
            .client
            .post(&url)
            .headers(headers)
            .form(&form)
            .send()
            .await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            return Err(Error::Status {
                status,
                body: text.clone(),
            });
        }

        let lower = text.to_lowercase();
        let success = ["msgto|", "sent successfully", "message sent", "ok|"]
            .iter()
            .any(|needle| lower.contains(needle));
        if success {
            Ok(())
        } else {
            Err(Error::Auth(format!("send failed: {}", text)))
        }
    }

    /// Fetch the first page of the inbox and return `(count, messages)`.
    ///
    /// `count` is the number of messages returned by the parsed page, not a server-reported total.
    pub async fn get_inbox_info(&mut self) -> Result<(usize, Vec<Message>)> {
        let messages = self.list_messages(1).await?;
        let count = messages.len();
        Ok((count, messages))
    }

    /// Convenience wrapper for `list_messages(1)`.
    /// Convenience wrapper for `list_messages(1)`.
    pub async fn check_inbox(&mut self) -> Result<Vec<Message>> {
        self.list_messages(1).await
    }

    /// Return the first message from `list_messages(1)`, if any.
    pub async fn get_last_message(&mut self) -> Result<Option<Message>> {
        let messages = self.list_messages(1).await?;
        Ok(messages.into_iter().next())
    }

    /// Fetch and return the text of the first message from `list_messages(1)`, if any.
    pub async fn get_last_message_content(&mut self) -> Result<Option<String>> {
        let messages = self.list_messages(1).await?;
        if let Some(msg) = messages.first() {
            let content = self.fetch_message(&msg.id).await?;
            Ok(Some(content))
        } else {
            Ok(None)
        }
    }

    /// Return the number of messages in `list_messages(1)`.
    pub async fn get_inbox_count(&mut self) -> Result<usize> {
        let messages = self.list_messages(1).await?;
        Ok(messages.len())
    }

    /// Return the number of messages in `list_messages(page)`.
    pub async fn get_inbox_count_page(&mut self, page: i32) -> Result<usize> {
        let messages = self.list_messages(page).await?;
        Ok(messages.len())
    }

    /// Return `(count, latest)` for `list_messages(1)`.
    ///
    /// `latest` is the first message in the returned vector, if any.
    pub async fn get_inbox_summary(&mut self) -> Result<(usize, Option<Message>)> {
        let messages = self.list_messages(1).await?;
        let count = messages.len();
        let latest = messages.get(0).cloned();
        Ok((count, latest))
    }

    /// Return `(count, latest)` for `list_messages(page)`.
    ///
    /// `latest` is the first message in the returned vector, if any.
    pub async fn get_inbox_summary_page(&mut self, page: i32) -> Result<(usize, Option<Message>)> {
        let messages = self.list_messages(page).await?;
        let count = messages.len();
        let latest = messages.get(0).cloned();
        Ok((count, latest))
    }

    /// Download an attachment and return its raw bytes.
    ///
    /// The `attachment` URL is typically obtained from [`MessageContent::attachments`].
    /// If the URL is relative, it is resolved against the client's `base_url`.
    ///
    /// # Errors
    /// - Returns [`Error::Http`] for transport failures and timeouts.
    /// - Returns [`Error::Status`] if the download endpoint responds with a non-success HTTP status.
    pub async fn download_attachment(&mut self, attachment: &Attachment) -> Result<Vec<u8>> {
        if self.yp_token.is_none() {
            self.open_inbox().await?;
        }
        self.set_default_cookies();

        let headers = build_headers(DEFAULT_HEADERS, MAIL_HEADERS);
        let url = normalize_url(&attachment.url, &self.base_url);
        let resp = self.client.get(url).headers(headers).send().await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;
        if !status.is_success() {
            return Err(Error::Status {
                status,
                body: format!("failed to download attachment: {}", status),
            });
        }
        Ok(bytes.to_vec())
    }

    fn set_default_cookies(&self) {
        let base: reqwest::Url = self.base_url.parse().expect("base URL should be valid");
        let time_now = current_time_cookie();
        self.jar.add_cookie_str(
            &format!("ytime={}; Domain=.yopmail.com; Path=/", time_now),
            &base,
        );
        self.jar.add_cookie_str(
            &format!("ywm={}; Domain=.yopmail.com; Path=/", self.mailbox),
            &base,
        );
    }

    async fn fetch_yj_token(&self, webmail_body: &str) -> Result<String> {
        let script_url = extract_webmail_script_url(webmail_body, &self.base_url)
            .unwrap_or_else(|| format!("{}/ver/{}/webmail.js", self.base_url, VERSION));
        let resp = self.client.get(script_url).send().await?;
        let status = resp.status();
        let body = resp.text().await?;
        if !status.is_success() {
            return Err(Error::Status { status, body });
        }
        extract_yj_token(&body).ok_or_else(|| Error::Parse("missing yj token".into()))
    }
}

fn current_time_cookie() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let minutes = (secs / 60) % (24 * 60);
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

fn extract_yp_token(body: &str) -> Option<String> {
    let doc = Html::parse_document(body);
    let selector = Selector::parse("input#yp").ok()?;
    for node in doc.select(&selector) {
        if let Some(value) = node.value().attr("value") {
            return Some(value.to_string());
        }
    }
    None
}

fn extract_webmail_script_url(body: &str, base: &str) -> Option<String> {
    let re = Regex::new(r#"<script[^>]+src=["']([^"']*webmail\.js)["']"#).ok()?;
    let path = re.captures(body)?.get(1)?.as_str();
    Some(normalize_url(path, base))
}

fn extract_webmail_version(body: &str) -> Option<String> {
    let re = Regex::new(r#"var\s+ver=['"]([^'"]+)['"]"#).ok()?;
    re.captures(body)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

fn extract_yj_token(script: &str) -> Option<String> {
    let re = Regex::new(r#"[?&]yj=([A-Za-z0-9]+)&v="#).ok()?;
    re.captures(script)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

fn parse_messages(body: &str) -> Vec<Message> {
    let doc = Html::parse_document(body);
    let message_sel = Selector::parse(".m").ok();
    let subject_sel = Selector::parse(".lsub, .lms").ok();
    let sender_sel = Selector::parse(".lmf").ok();
    let time_sel = Selector::parse(".lmh").ok();

    let mut messages = Vec::new();
    if let Some(msg_sel) = message_sel {
        for el in doc.select(&msg_sel) {
            let id = el
                .value()
                .id()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "".into());
            if id.is_empty() {
                continue;
            }

            let subject = subject_sel
                .as_ref()
                .and_then(|sel| el.select(sel).next())
                .map(|n| n.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            let sender = sender_sel
                .as_ref()
                .and_then(|sel| el.select(sel).next())
                .map(|n| n.text().collect::<String>().trim().to_string());

            let time = time_sel
                .as_ref()
                .and_then(|sel| el.select(sel).next())
                .map(|n| n.text().collect::<String>().trim().to_string());

            messages.push(Message {
                id,
                subject,
                sender,
                time,
            });
        }
    }
    messages
}

const MESSAGE_CONTAINER_SELECTORS: &[&str] = &[
    "#mailctn #mail",
    "#mailctn",
    "#mail",
    "div.mail-body",
    "div.mail",
    "div.message",
    "div.content",
    "div.body",
];

fn extract_from_message_container(
    body: &str,
    mut extract: impl for<'a> FnMut(ElementRef<'a>) -> Option<String>,
) -> Option<String> {
    let doc = Html::parse_document(body);
    for sel in MESSAGE_CONTAINER_SELECTORS {
        if let Ok(selector) = Selector::parse(sel) {
            if let Some(node) = doc.select(&selector).next() {
                if let Some(value) = extract(node) {
                    return Some(value);
                }
            }
        }
    }
    None
}

fn extract_message_body(body: &str) -> String {
    extract_from_message_container(body, |node| {
        let text = node.text().collect::<String>();
        (text.trim().len() > 5).then(|| clean_text(&text))
    })
    .unwrap_or_else(|| clean_text(body))
}

fn extract_message_html(body: &str) -> String {
    extract_from_message_container(body, |node| {
        let html = node.inner_html();
        (html.trim().len() > 5).then_some(html)
    })
    .unwrap_or_else(|| body.to_string())
}

fn extract_attachments(body: &str, base: &str) -> Vec<Attachment> {
    let doc = Html::parse_document(body);
    let mut seen = HashSet::new();
    let mut attachments = Vec::new();
    if let Ok(sel) = Selector::parse("a.pj") {
        for node in doc.select(&sel) {
            if let Some(href) = node.value().attr("href") {
                let url = normalize_url(href, base);
                if seen.insert(url.clone()) {
                    let name = node
                        .value()
                        .attr("title")
                        .map(|s| s.to_string())
                        .or_else(|| {
                            let txt = node.text().collect::<String>().trim().to_string();
                            if txt.is_empty() { None } else { Some(txt) }
                        });
                    attachments.push(Attachment { name, url });
                }
            }
        }
    }

    if let Ok(re) = Regex::new(r#"(/downmail\?[^"' ]+)"#) {
        for cap in re.captures_iter(body) {
            if let Some(m) = cap.get(1) {
                let url = normalize_url(m.as_str(), base);
                if seen.insert(url.clone()) {
                    attachments.push(Attachment { name: None, url });
                }
            }
        }
    }

    attachments
}

fn normalize_url(href: &str, base: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        href.to_string()
    } else if href.starts_with('/') {
        format!("{}{}", base.trim_end_matches('/'), href)
    } else {
        format!("{}/{}", base.trim_end_matches('/'), href)
    }
}

fn clean_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut last_ws = false;
    for c in input.chars() {
        if c.is_whitespace() {
            if !last_ws {
                out.push(' ');
            }
            last_ws = true;
        } else {
            out.push(c);
            last_ws = false;
        }
    }
    out.trim().to_string()
}

/// Generate a random mailbox name.
///
/// The generated value:
/// - uses ASCII alphanumeric characters only
/// - is lowercased
/// - clamps `len` to the inclusive range `6..=32`
pub fn generate_random_mailbox(len: usize) -> String {
    let length = len.clamp(6, 32);
    let mut rng = rand::thread_rng();
    let raw: String = (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect();
    raw.to_lowercase()
}
