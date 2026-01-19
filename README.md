# yopmail-client-rs

Unofficial blocking Rust client for YOPmail. It mirrors the web UI flow (cookies + `yp` tokens) to list inboxes, fetch message bodies (text + HTML), download attachments, send mails, and work with RSS feeds. Ships with a small CLI example.

## Install
```bash
cargo add yopmail-client-rs
```

## Quickstart (library)
```rust
use yopmail_client_rs::{
    check_inbox_page, get_message_by_id, get_message_by_id_full, generate_random_mailbox,
    YopmailClient, Config,
};

let mailbox = "mytempbox";
let cfg = Config::default();

// List first page
let messages = check_inbox_page(mailbox, 1, Some(cfg.clone()))?;

// Fetch plain text
let body = get_message_by_id(mailbox, &messages[0].id, Some(cfg.clone()))?;

// Fetch full content (html/raw/attachments)
let content = get_message_by_id_full(mailbox, &messages[0].id, Some(cfg.clone()))?;
for att in &content.attachments {
    println!("attachment: {} -> {}", att.name.clone().unwrap_or_default(), att.url);
}

// Download an attachment
let mut client = YopmailClient::new(mailbox, Some(cfg.clone()))?;
client.open_inbox()?;
let bytes = client.download_attachment(&content.attachments[0])?;

// Generate a random mailbox name
let random_box = generate_random_mailbox(12);
println!("{random_box}@yopmail.com");
```

## CLI (examples/cli.rs)
```bash
cargo run --example cli -- --mailbox mytempbox list --details
cargo run --example cli -- --mailbox mytempbox fetch --id <message-id>
cargo run --example cli -- --mailbox mytempbox fetch --id <message-id> --html
cargo run --example cli -- --mailbox mytempbox fetch --id <message-id> --attachments
cargo run --example cli -- --mailbox mytempbox fetch --id <message-id> --download-attachments downloads/
cargo run --example cli -- random --len 10
```

Commands: `list`, `fetch`, `send`, `rss-url`, `rss-data`, `info`, `random`. Use `--proxy` to tunnel through a proxy.

## Features
- Inbox: list with paging (`check_inbox_page`, `list_messages`).
- Fetch: text/HTML/raw plus attachment discovery (`get_message_by_id_full`, `fetch_message_full`).
- Attachments: download via `download_attachment`.
- Send: post to another `@yopmail.com` address.
- RSS: get feed URL and parse items.
- Helpers: inbox counts/summaries, random mailbox generator.

## Notes
- Network is live scraping of YOPmail; availability and captcha/rate limits are outside this client’s control.
- Only `@yopmail.com` recipients are accepted by `send_message`.
- Attachment parsing uses the webmail DOM (links with class `pj` or `downmail` URLs).
