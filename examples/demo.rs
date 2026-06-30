//! Comprehensive example showcasing the YOPmail client.
//!
//! Features demonstrated:
//! - Creating a client (with optional proxy support)
//! - Creating a random temporary mailbox
//! - Polling for incoming messages
//! - Fetching full email content
//! - Downloading attachments

use std::io::Write;
use std::time::{Duration, Instant};
use yopmail_client::{DEFAULT_DOMAIN, Error, YopmailClient, generate_random_mailbox};

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("YOPmail Rust Client - Full Demo");
    println!("{}", "=".repeat(50));

    println!("\nCreating client...");
    let mailbox = generate_random_mailbox(12);
    let address = format!("{mailbox}@{DEFAULT_DOMAIN}");
    let mut client = YopmailClient::new(&mailbox)?;

    // With proxy (uncomment to use):
    // let mut client = YopmailClient::builder(&mailbox)
    //     .proxy_url("http://127.0.0.1:8080")
    //     .build()?;

    client.open_inbox().await?;
    println!("   Connected to YOPmail");
    println!("   Mailbox: {address}");

    println!("\nWaiting for messages...");
    println!("   Send an email to: {address}");
    println!("   Polling for up to 2 minutes");

    let start = Instant::now();
    let timeout = Duration::from_secs(120);
    let poll_interval = Duration::from_secs(5);

    loop {
        let messages = client.list_messages(1).await?;

        if !messages.is_empty() {
            println!("\n\nReceived {} message(s)!", messages.len());

            for msg in &messages {
                println!("\n{}", "-".repeat(50));
                println!("Message ID: {}", msg.id);
                println!("Subject:    {}", msg.subject);
                if let Some(sender) = &msg.sender {
                    println!("From:       {sender}");
                }
                if let Some(time) = &msg.time {
                    println!("Time:       {time}");
                }

                println!("\nFetching full email body...");
                match client.fetch_message_full(&msg.id).await {
                    Ok(content) => {
                        println!("   Text length: {} characters", content.text.len());
                        println!("   HTML length: {} characters", content.html.len());
                        println!("   Preview:");
                        println!("   {}", "-".repeat(40));

                        let preview: String = content.text.chars().take(500).collect();
                        if preview.trim().is_empty() {
                            println!("   (empty text body)");
                        } else {
                            for line in preview.lines().take(10) {
                                println!("   {line}");
                            }
                            if content.text.len() > 500 {
                                println!("   ... (truncated)");
                            }
                        }

                        if !content.attachments.is_empty() {
                            println!("\nFound {} attachment(s)", content.attachments.len());
                            for (idx, attachment) in content.attachments.iter().enumerate() {
                                let name =
                                    attachment.name.as_deref().unwrap_or("unnamed attachment");
                                println!("   {}. {} -> {}", idx + 1, name, attachment.url);
                                match client.download_attachment(attachment).await {
                                    Ok(bytes) => println!("      Downloaded {} bytes", bytes.len()),
                                    Err(e) => eprintln!("      Download failed: {e}"),
                                }
                            }
                        }
                    }
                    Err(e) => eprintln!("   Failed to fetch: {e}"),
                }
            }
            break;
        }

        if start.elapsed() >= timeout {
            println!("\n\nTimeout: no messages received");
            break;
        }

        let remaining = timeout.saturating_sub(start.elapsed()).as_secs();
        print!("\r   Checking... {remaining} seconds remaining   ");
        std::io::stdout().flush()?;
        tokio::time::sleep(poll_interval).await;
    }

    println!("\n{}", "=".repeat(50));
    println!("Demo complete!");

    Ok(())
}
