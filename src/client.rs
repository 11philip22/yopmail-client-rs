use crate::constants::*;
use crate::error::{Error, Result};
use crate::models::{Attachment, Config, Message, MessageContent, RssItem};
use regex::Regex;
use reqwest::{
    cookie::Jar,
    header::{HeaderMap, HeaderName, HeaderValue},
    Client, ClientBuilder, StatusCode,
};
use rand::{distributions::Alphanumeric, Rng};
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::SystemTime;

fn parse_mailbox(mailbox: &str) -> (String, String) {
    if let Some((local, domain)) = mailbox.split_once('@') {
        (
            local.trim().to_lowercase(),
            domain.trim().to_lowercase(),
        )
    } else {
        (mailbox.trim().to_lowercase(), DEFAULT_DOMAIN.to_string())
    }
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

pub struct YopmailClient {
    mailbox: String,
    domain: String,
    config: Config,
    jar: Arc<Jar>,
    client: Client,
    yp_token: Option<String>,
}

impl YopmailClient {
    pub fn new(mailbox: impl AsRef<str>, config: Option<Config>) -> Result<Self> {
        let (mailbox, domain) = parse_mailbox(mailbox.as_ref());
        let cfg = config.unwrap_or_default();
        let jar = Arc::new(Jar::default());

        let mut builder = ClientBuilder::new()
            .cookie_provider(jar.clone())
            .timeout(cfg.timeout)
            .default_headers(default_headers());

        if let Some(proxy) = &cfg.proxy_url {
            builder = builder.proxy(reqwest::Proxy::all(proxy).map_err(Error::Http)?);
        }

        let client = builder.build().map_err(Error::Http)?;

        Ok(Self {
            mailbox,
            domain,
            config: cfg,
            jar,
            client,
            yp_token: None,
        })
    }

    pub async fn open_inbox(&mut self) -> Result<()> {
        self.set_default_cookies();

        // Use the same flow as the web UI: load the login page (with yp token) then post the form
        let login_url = format!("{}/en/?login={}", self.config.base_url, self.mailbox);
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
            let _ = self
                .client
                .post(format!("{}/en/", self.config.base_url))
                .headers(default_headers())
                .form(&form)
                .send()
                .await?;
        }
        Ok(())
    }

    pub async fn list_messages(&mut self, page: i32) -> Result<Vec<Message>> {
        if self.yp_token.is_none() {
            self.open_inbox().await?;
        }

        let yp = self
            .yp_token
            .clone()
            .unwrap_or_else(|| FALLBACK_YP_TOKEN.to_string());

        let params = [
            ("login", self.mailbox.as_str()),
            ("p", &page.to_string()),
            ("d", ""),
            ("ctrl", ""),
            ("yp", yp.as_str()),
            ("yj", YJ_TOKEN),
            ("v", VERSION),
            ("r_c", ""),
            ("id", ""),
            ("ad", &AD_PARAM.to_string()),
        ];

        let headers = build_headers(DEFAULT_HEADERS, INBOX_HEADERS);
        let url = format!("{}/inbox", self.config.base_url);
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

    pub async fn fetch_message(&mut self, message_id: &str) -> Result<String> {
        let content = self.fetch_message_full(message_id).await?;
        Ok(content.text)
    }

    pub async fn fetch_message_full(&mut self, message_id: &str) -> Result<MessageContent> {
        if self.yp_token.is_none() {
            self.open_inbox().await?;
        }

        // Refresh cookies (ytime/ywm) just before the mail fetch
        self.set_default_cookies();

        let headers = build_headers(DEFAULT_HEADERS, MAIL_HEADERS);
        let mail_url = format!("{}/en/mail", self.config.base_url);
        let yp = self
            .yp_token
            .clone()
            .unwrap_or_else(|| FALLBACK_YP_TOKEN.to_string());
        let ad_param = AD_PARAM.to_string();

        let raw_id = message_id.trim();
        let main_id = if raw_id.starts_with('m') {
            raw_id.to_string()
        } else if raw_id.starts_with("e_") {
            format!("m{}", raw_id)
        } else {
            format!("m_{}", raw_id.trim_start_matches("m_"))
        };
        let alt_id = if raw_id.starts_with("e_")
            || raw_id.starts_with("me_")
            || raw_id.starts_with("m_")
        {
            raw_id.to_string()
        } else {
            format!("e_{raw_id}")
        };

        let variants = vec![
            (main_id, true),
            (alt_id, true),
            (raw_id.to_string(), false),
        ];

        let mut last_status = None;
        let mut last_body = None;
        for (id, use_full_params) in variants {
            let mut params_owned = vec![
                ("b".to_string(), self.mailbox.clone()),
                ("id".to_string(), id),
            ];
            if use_full_params {
                params_owned.extend_from_slice(&[
                    ("yp".to_string(), yp.clone()),
                    ("yj".to_string(), YJ_TOKEN.to_string()),
                    ("v".to_string(), VERSION.to_string()),
                    ("d".to_string(), "".to_string()),
                    ("ctrl".to_string(), "".to_string()),
                    ("r_c".to_string(), "".to_string()),
                    ("ad".to_string(), ad_param.clone()),
                ]);
            }

            let params: Vec<(&str, &str)> = params_owned
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();

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
                let attachments = extract_attachments(&body, &self.config.base_url);
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

    pub async fn send_message(&mut self, to: &str, subject: &str, body: &str) -> Result<()> {
        if !to.ends_with("@yopmail.com") {
            return Err(Error::InvalidRecipient);
        }
        if self.yp_token.is_none() {
            self.open_inbox().await?;
        }

        let recipient_ok = ALT_DOMAINS.iter().any(|d| to.ends_with(d));
        if !recipient_ok {
            return Err(Error::InvalidRecipient);
        }
        let form = [
            ("msgfrom", format!("{}@{}", self.mailbox, self.domain)),
            ("msgto", to.to_string()),
            ("msgsubject", subject.to_string()),
            ("msgbody", body.to_string()),
        ];

        let headers = build_headers(DEFAULT_HEADERS, SEND_HEADERS);
        let url = format!("{}/writepost", self.config.base_url);
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

    pub async fn get_inbox_info(&mut self) -> Result<(usize, Vec<Message>)> {
        let messages = self.list_messages(1).await?;
        let count = messages.len();
        Ok((count, messages))
    }

    pub async fn download_attachment(&mut self, attachment: &Attachment) -> Result<Vec<u8>> {
        if self.yp_token.is_none() {
            self.open_inbox().await?;
        }
        self.set_default_cookies();

        let headers = build_headers(DEFAULT_HEADERS, MAIL_HEADERS);
        let url = normalize_url(&attachment.url, &self.config.base_url);
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

    pub fn get_rss_feed_url(&self, mailbox: Option<&str>) -> String {
        let target = mailbox.unwrap_or(&self.mailbox);
        format!("{}/rss?login={}", self.config.base_url, target)
    }

    pub async fn get_rss_feed_data(
        &mut self,
        mailbox: Option<&str>,
    ) -> Result<(String, Vec<RssItem>)> {
        let target = mailbox.unwrap_or(&self.mailbox);
        let gen_url = format!("{}/gen-rss?login={}", self.config.base_url, target);

        let resp = self.client.get(&gen_url).send().await?;
        let body = resp.text().await?;
        let rss_url = extract_rss_url(&body, &self.config.base_url, target);

        let rss_resp = self.client.get(&rss_url).send().await?;
        let rss_body = rss_resp.text().await?;
        let items = parse_rss_items(&rss_body);
        Ok((rss_url, items))
    }

    fn set_default_cookies(&self) {
        let base: reqwest::Url = self
            .config
            .base_url
            .parse()
            .expect("base URL should be valid");
        let time_now = current_time_cookie();
        self.jar
            .add_cookie_str(&format!("ytime={}; Domain=.yopmail.com; Path=/", time_now), &base);
        self.jar.add_cookie_str(
            &format!("ywm={}; Domain=.yopmail.com; Path=/", self.mailbox),
            &base,
        );
    }
}

fn current_time_cookie() -> String {
    use chrono::prelude::*;
    let now: DateTime<Utc> = SystemTime::now().into();
    now.format("%H:%M").to_string()
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
                date: None,
                time,
            });
        }
    }
    messages
}

fn extract_message_body(body: &str) -> String {
    let doc = Html::parse_document(body);
    let selectors = [
        "#mailctn #mail",
        "#mailctn",
        "#mail",
        "div.mail-body",
        "div.mail",
        "div.message",
        "div.content",
        "div.body",
    ];
    for sel in selectors {
        if let Ok(selector) = Selector::parse(sel) {
            if let Some(node) = doc.select(&selector).next() {
                let text = node.text().collect::<String>();
                if text.trim().len() > 5 {
                    return clean_text(&text);
                }
            }
        }
    }
    clean_text(body)
}

fn extract_message_html(body: &str) -> String {
    let doc = Html::parse_document(body);
    let selectors = [
        "#mailctn #mail",
        "#mailctn",
        "#mail",
        "div.mail-body",
        "div.mail",
        "div.message",
        "div.content",
        "div.body",
    ];
    for sel in selectors {
        if let Ok(selector) = Selector::parse(sel) {
            if let Some(node) = doc.select(&selector).next() {
                let html = node.inner_html();
                if html.trim().len() > 5 {
                    return html;
                }
            }
        }
    }
    body.to_string()
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
                            if txt.is_empty() {
                                None
                            } else {
                                Some(txt)
                            }
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

fn extract_rss_url(gen_body: &str, base: &str, mailbox: &str) -> String {
    let pattern = format!(r#"href="(/rss\?login={}&h=[^"]+)""#, regex::escape(mailbox));
    let re = Regex::new(&pattern).ok();
    if let Some(re) = re {
        if let Some(caps) = re.captures(gen_body) {
            if let Some(path) = caps.get(1) {
                return format!("{}{}", base, path.as_str());
            }
        }
    }
    format!("{}/rss?login={}", base, mailbox)
}

fn parse_rss_items(body: &str) -> Vec<RssItem> {
    let doc = Html::parse_document(body);
    let item_sel = Selector::parse("item").ok();
    let title_sel = Selector::parse("title").ok();
    let link_sel = Selector::parse("link").ok();
    let date_sel = Selector::parse("pubdate").ok();
    let desc_sel = Selector::parse("description").ok();

    let mut items = Vec::new();
    if let Some(item_sel) = item_sel {
        for node in doc.select(&item_sel) {
            let subject = title_sel
                .as_ref()
                .and_then(|sel| node.select(sel).next())
                .map(|n| n.text().collect::<String>().trim().to_string())
                .unwrap_or_else(|| "No Subject".into());
            let url = link_sel
                .as_ref()
                .and_then(|sel| node.select(sel).next())
                .map(|n| n.text().collect::<String>().trim().to_string())
                .unwrap_or_default();
            let date = date_sel
                .as_ref()
                .and_then(|sel| node.select(sel).next())
                .map(|n| n.text().collect::<String>().trim().to_string())
                .unwrap_or_else(|| "Unknown Date".into());
            let description = desc_sel
                .as_ref()
                .and_then(|sel| node.select(sel).next())
                .map(|n| n.text().collect::<String>().trim().to_string());
            let sender = description
                .as_ref()
                .and_then(|desc| find_email(desc))
                .unwrap_or_else(|| "Unknown".into());

            items.push(RssItem {
                subject,
                sender,
                date,
                url,
                description,
            });
        }
    }
    items
}

fn find_email(text: &str) -> Option<String> {
    let re = Regex::new(r"([a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})").ok()?;
    re.captures(text)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

pub async fn check_inbox(mailbox: &str, config: Option<Config>) -> Result<Vec<Message>> {
    let mut client = YopmailClient::new(mailbox, config)?;
    client.open_inbox().await?;
    client.list_messages(1).await
}

pub async fn check_inbox_page(
    mailbox: &str,
    page: i32,
    config: Option<Config>,
) -> Result<Vec<Message>> {
    let mut client = YopmailClient::new(mailbox, config)?;
    client.open_inbox().await?;
    client.list_messages(page).await
}

pub async fn get_message_by_id(
    mailbox: &str,
    message_id: &str,
    config: Option<Config>,
) -> Result<String> {
    let mut client = YopmailClient::new(mailbox, config)?;
    client.open_inbox().await?;
    client.fetch_message(message_id).await
}

pub async fn get_message_by_id_full(
    mailbox: &str,
    message_id: &str,
    config: Option<Config>,
) -> Result<MessageContent> {
    let mut client = YopmailClient::new(mailbox, config)?;
    client.open_inbox().await?;
    client.fetch_message_full(message_id).await
}

pub async fn get_last_message(mailbox: &str, config: Option<Config>) -> Result<Option<Message>> {
    let mut client = YopmailClient::new(mailbox, config)?;
    client.open_inbox().await?;
    let messages = client.list_messages(1).await?;
    Ok(messages.into_iter().next())
}

pub async fn get_last_message_content(
    mailbox: &str,
    config: Option<Config>,
) -> Result<Option<String>> {
    let mut client = YopmailClient::new(mailbox, config)?;
    client.open_inbox().await?;
    let messages = client.list_messages(1).await?;
    if let Some(msg) = messages.first() {
        let content = client.fetch_message(&msg.id).await?;
        Ok(Some(content))
    } else {
        Ok(None)
    }
}

pub async fn get_inbox_count(mailbox: &str, config: Option<Config>) -> Result<usize> {
    let mut client = YopmailClient::new(mailbox, config)?;
    client.open_inbox().await?;
    let messages = client.list_messages(1).await?;
    Ok(messages.len())
}

pub async fn get_inbox_count_page(mailbox: &str, page: i32, config: Option<Config>) -> Result<usize> {
    let mut client = YopmailClient::new(mailbox, config)?;
    client.open_inbox().await?;
    let messages = client.list_messages(page).await?;
    Ok(messages.len())
}

pub async fn get_inbox_summary(
    mailbox: &str,
    config: Option<Config>,
) -> Result<(usize, Option<Message>)> {
    let mut client = YopmailClient::new(mailbox, config)?;
    client.open_inbox().await?;
    let messages = client.list_messages(1).await?;
    let count = messages.len();
    let latest = messages.get(0).cloned();
    Ok((count, latest))
}

pub async fn get_inbox_summary_page(
    mailbox: &str,
    page: i32,
    config: Option<Config>,
) -> Result<(usize, Option<Message>)> {
    let mut client = YopmailClient::new(mailbox, config)?;
    client.open_inbox().await?;
    let messages = client.list_messages(page).await?;
    let count = messages.len();
    let latest = messages.get(0).cloned();
    Ok((count, latest))
}

pub fn get_rss_feed_url(mailbox: &str, config: Option<Config>) -> Result<String> {
    let client = YopmailClient::new(mailbox, config)?;
    Ok(client.get_rss_feed_url(None))
}

pub async fn get_rss_feed_data(mailbox: &str, config: Option<Config>) -> Result<(String, Vec<RssItem>)> {
    let mut client = YopmailClient::new(mailbox, config)?;
    client.get_rss_feed_data(None).await
}

/// Generate a random mailbox name (alphanumeric, lowercased).
pub fn generate_random_mailbox(len: usize) -> String {
    let length = len.max(6).min(32);
    let mut rng = rand::thread_rng();
    let raw: String = (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect();
    raw.to_lowercase()
}
