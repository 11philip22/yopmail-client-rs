use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use yopmail_client::{
    check_inbox_page, generate_random_mailbox, get_inbox_summary, get_rss_feed_data,
    get_rss_feed_url, Error, YopmailClient,
};

#[derive(Parser, Debug)]
#[command(
    name = "yopmail-client",
    about = "Interact with YOPmail disposable inboxes (unofficial)",
    arg_required_else_help = true
)]
struct Cli {
    #[arg(short, long, help = "Mailbox name (without @yopmail.com)")]
    mailbox: Option<String>,

    #[arg(long, help = "Proxy URL (optional)")]
    proxy: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List messages in the inbox
    List {
        #[arg(long, default_value_t = 1)]
        page: i32,
        #[arg(long)]
        details: bool,
    },
    /// Fetch a message body by ID
    Fetch {
        #[arg(long)]
        id: String,
        #[arg(long, help = "Return HTML instead of plain text")]
        html: bool,
        #[arg(long, help = "Return raw HTML (debug)")]
        raw: bool,
        #[arg(long, help = "List attachment names/URLs")]
        attachments: bool,
        #[arg(long, value_name = "DIR", help = "Download attachments to directory")]
        download_attachments: Option<PathBuf>,
    },
    /// Send an email to another YOPmail address
    Send {
        #[arg(long)]
        to: String,
        #[arg(long)]
        subject: String,
        #[arg(long)]
        body: String,
    },
    /// Show RSS feed URL
    RssUrl {
        #[arg(long)]
        mailbox: Option<String>,
    },
    /// Fetch RSS feed data
    RssData {
        #[arg(long)]
        mailbox: Option<String>,
    },
    /// Show inbox summary
    Info,
    /// Generate a random mailbox name
    Random {
        #[arg(long, default_value_t = 10, help = "Length of mailbox (6-32)")]
        len: usize,
    },
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    let config = build_config(cli.proxy.clone());
    let mailbox_opt = cli.mailbox.clone();

    match cli.command {
        Commands::List { page, details } => {
            let mailbox = require_mailbox(&mailbox_opt);
            let messages = check_inbox_page(&mailbox, page, config.clone())?;
            if messages.is_empty() {
                println!("No messages found.");
            } else {
                println!("Found {} message(s):", messages.len());
                for (idx, msg) in messages.iter().enumerate() {
                    println!("{}. {}", idx + 1, msg.subject);
                    if details {
                        println!("   ID: {}", msg.id);
                        if let Some(sender) = &msg.sender {
                            println!("   From: {}", sender);
                        }
                        if let Some(time) = &msg.time {
                            println!("   Time: {}", time);
                        }
                    }
                }
            }
        }
        Commands::Fetch {
            id,
            html,
            raw,
            attachments,
            download_attachments,
        } => {
            let mailbox = require_mailbox(&mailbox_opt);
            let mut client = YopmailClient::new(&mailbox, config.clone())?;
            client.open_inbox()?;
            let content = client.fetch_message_full(&id)?;

            if raw {
                println!("{}", content.raw);
            } else if html {
                println!("{}", content.html);
            } else {
                println!("{}", content.text);
            }

            if attachments || download_attachments.is_some() {
                if content.attachments.is_empty() {
                    println!("No attachments found.");
                } else {
                    println!("{} attachment(s):", content.attachments.len());
                    for (idx, att) in content.attachments.iter().enumerate() {
                        let name = att
                            .name
                            .clone()
                            .unwrap_or_else(|| format!("attachment_{idx}"));
                        println!("{}. {} -> {}", idx + 1, name, att.url);
                    }
                }
            }

            if let Some(dir) = download_attachments {
                if content.attachments.is_empty() {
                    println!("No attachments to download.");
                } else {
                    fs::create_dir_all(&dir)?;
                    for (idx, att) in content.attachments.iter().enumerate() {
                        let name = att
                            .name
                            .clone()
                            .unwrap_or_else(|| format!("attachment_{idx}"));
                        let path = dir.join(name);
                        let bytes = client.download_attachment(att)?;
                        fs::write(&path, &bytes)?;
                        println!("Saved {} ({} bytes)", path.display(), bytes.len());
                    }
                }
            }
        }
        Commands::Send { to, subject, body } => {
            let mailbox = require_mailbox(&mailbox_opt);
            let mut client = YopmailClient::new(&mailbox, config.clone())?;
            client.open_inbox()?;
            client.send_message(&to, &subject, &body)?;
            println!("Message sent to {}", to);
        }
        Commands::RssUrl { mailbox } => {
            let fallback = require_mailbox(&mailbox_opt);
            let url =
                get_rss_feed_url(mailbox.as_deref().unwrap_or(&fallback), config.clone())?;
            println!("{url}");
        }
        Commands::RssData { mailbox } => {
            let fallback = require_mailbox(&mailbox_opt);
            let (url, items) =
                get_rss_feed_data(mailbox.as_deref().unwrap_or(&fallback), config.clone())?;
            println!("RSS URL: {url}");
            println!("{} message(s)", items.len());
            for (idx, item) in items.iter().enumerate() {
                println!("{}. {} (from: {})", idx + 1, item.subject, item.sender);
                println!("   Date: {}", item.date);
                println!("   URL: {}", item.url);
            }
        }
        Commands::Info => {
            let mailbox = require_mailbox(&mailbox_opt);
            let (count, latest) = get_inbox_summary(&mailbox, config.clone())?;
            let display = if mailbox.contains('@') {
                mailbox.clone()
            } else {
                format!("{mailbox}@{}", yopmail_client_rs::DEFAULT_DOMAIN)
            };
            println!("Mailbox: {}", display);
            println!("Messages: {}", count);
            if let Some(msg) = latest {
                println!("Latest: {}", msg.subject);
                if let Some(sender) = msg.sender {
                    println!("From: {}", sender);
                }
            }
        }
        Commands::Random { len } => {
            let mailbox = generate_random_mailbox(len);
            println!("{mailbox}@yopmail.com");
        }
    }

    Ok(())
}

fn build_config(proxy: Option<String>) -> Option<yopmail_client::models::Config> {
    let mut cfg = yopmail_client::models::Config::default();
    cfg.proxy_url = proxy;
    Some(cfg)
}

fn require_mailbox(mailbox: &Option<String>) -> String {
    if let Some(mb) = mailbox {
        mb.clone()
    } else {
        eprintln!("--mailbox is required for this command");
        std::process::exit(2);
    }
}
