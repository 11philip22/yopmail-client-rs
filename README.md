<h1 align="center">yopmail-client</h1>

<p align="center">
  Unofficial async Rust client for disposable <a href="https://yopmail.com">YOPmail</a> inboxes.
</p>

<p align="center">
  <a href="https://crates.io/crates/yopmail-client"><img src="https://img.shields.io/crates/v/yopmail-client?style=flat-square" alt="Crates.io"></a>
  <a href="https://docs.rs/yopmail-client"><img src="https://img.shields.io/docsrs/yopmail-client?style=flat-square" alt="Documentation"></a>
  <img src="https://img.shields.io/badge/Rust-2024-orange?style=flat-square&logo=rust" alt="Rust 2024">
  <a href="LICENSE"><img src="https://img.shields.io/badge/MIT-blue?style=flat-square" alt="MIT"></a>
</p>

<p align="center">
  <a href="#features">Features</a> &middot;
  <a href="#install">Install</a> &middot;
  <a href="#quickstart">Quickstart</a> &middot;
  <a href="#examples">Examples</a> &middot;
  <a href="#api-overview">API Overview</a> &middot;
  <a href="#notes">Notes</a> &middot;
  <a href="#resources">Resources</a>
</p>

---

`yopmail-client` is a small async Rust crate for working with temporary YOPmail inboxes from tests, demos, and automation. It can list inbox pages, fetch message content, download attachments, and send YOPmail-to-YOPmail messages.

> [!NOTE]
> This crate is unofficial. It talks to YOPmail's public web endpoints, so changes to YOPmail's HTML or request flow can require parser updates.

## Features

- List inbox messages with paging.
- Fetch plain text, extracted HTML, raw HTML, and attachment links.
- Download message attachments as bytes.
- Send messages from a YOPmail mailbox to `@yopmail.com` recipients.
- Generate random mailbox names for disposable test flows.
- Configure timeout, proxy, and base URL through `YopmailClientBuilder`.
- Work with simple `serde`-serializable models.

## Install

```bash
cargo add yopmail-client
```

The client is async. The examples below use Tokio:

```bash
cargo add tokio --features macros,rt-multi-thread
```

## Quickstart

```rust
use yopmail_client::{generate_random_mailbox, YopmailClient};

#[tokio::main]
async fn main() -> Result<(), yopmail_client::Error> {
    let mailbox = generate_random_mailbox(12);
    let mut client = YopmailClient::new(&mailbox)?;

    let messages = client.list_messages(1).await?;
    println!("{} message(s) in {mailbox}@yopmail.com", messages.len());

    if let Some(message) = messages.first() {
        let content = client.fetch_message_full(&message.id).await?;

        println!("Subject: {}", message.subject);
        println!("{}", content.text);

        for attachment in &content.attachments {
            let bytes = client.download_attachment(attachment).await?;
            println!("downloaded {} bytes from {}", bytes.len(), attachment.url);
        }
    }

    Ok(())
}
```

Most methods open the session lazily. Call `open_inbox()` yourself when you want that state transition to be explicit:

```rust
let mut client = YopmailClient::new("my-temp-inbox")?;
client.open_inbox().await?;
let messages = client.check_inbox().await?;
```

Customize the underlying HTTP client with the builder:

```rust
use std::time::Duration;
use yopmail_client::YopmailClient;

let mut client = YopmailClient::builder("my-temp-inbox")
    .timeout(Duration::from_secs(20))
    .proxy_url("http://127.0.0.1:8080")
    .build()?;
```

## Examples

This repository ships two examples:

```bash
cargo run --example self_send
cargo run --example demo
```

## API Overview

| Task | Method |
| --- | --- |
| Create a client | `YopmailClient::new`, `YopmailClient::builder` |
| Initialize session | `open_inbox` |
| List inbox | `list_messages`, `check_inbox`, `get_inbox_info` |
| Fetch content | `fetch_message`, `fetch_message_full` |
| Download files | `download_attachment` |
| Send mail | `send_message` |
| Helpers | `get_last_message`, `get_inbox_summary`, `generate_random_mailbox` |

Mailboxes are local parts only. The domain is always `yopmail.com`.

## Notes

> [!TIP]
> Prefer `generate_random_mailbox()` for tests and demos so multiple runs do not collide in a shared public inbox.

- `YopmailClient` is stateful and methods take `&mut self` because cookies and the `yp` token are refreshed on demand.
- `Message::date` is currently always `None`; `Message::time` is parsed when available.
- `send_message` currently accepts only recipients ending in `@yopmail.com`.
- `fetch_message_full` retries a few YOPmail message ID variants after HTTP 400 responses, but the crate does not implement network retry or polling loops.
- Non-success responses from core inbox, mail, send, and attachment requests are returned as `Error::Status` with the response body captured for debugging.

## Resources

- [API documentation on docs.rs](https://docs.rs/yopmail-client)
- [Package on crates.io](https://crates.io/crates/yopmail-client)
- [YOPmail](https://yopmail.com)
- [Python yopmail-client](https://pypi.org/project/yopmail-client/1.2.3/), which inspired this Rust port
