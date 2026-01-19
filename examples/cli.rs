use clap::{Parser, Subcommand};
use yopmail_client_rs::{
    check_inbox, get_inbox_summary, get_message_by_id, get_rss_feed_data, get_rss_feed_url, Error,
    YopmailClient,
};

#[derive(Parser, Debug)]
#[command(
    name = "yopmail-client",
    about = "Interact with YOPmail disposable inboxes (unofficial)",
    arg_required_else_help = true
)]
struct Cli {
    #[arg(short, long, required = true, help = "Mailbox name (without @yopmail.com)")]
    mailbox: String,

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
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    let config = build_config(cli.proxy.clone());

    match cli.command {
        Commands::List { page: _, details } => {
            let messages = check_inbox(&cli.mailbox, config.clone())?;
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
        Commands::Fetch { id } => {
            let content = get_message_by_id(&cli.mailbox, &id, config.clone())?;
            println!("{}", content);
        }
        Commands::Send { to, subject, body } => {
            let mut client = YopmailClient::new(&cli.mailbox, config.clone())?;
            client.open_inbox()?;
            client.send_message(&to, &subject, &body)?;
            println!("Message sent to {}", to);
        }
        Commands::RssUrl { mailbox } => {
            let url = get_rss_feed_url(mailbox.as_deref().unwrap_or(&cli.mailbox), config.clone())?;
            println!("{url}");
        }
        Commands::RssData { mailbox } => {
            let (url, items) =
                get_rss_feed_data(mailbox.as_deref().unwrap_or(&cli.mailbox), config.clone())?;
            println!("RSS URL: {url}");
            println!("{} message(s)", items.len());
            for (idx, item) in items.iter().enumerate() {
                println!("{}. {} (from: {})", idx + 1, item.subject, item.sender);
                println!("   Date: {}", item.date);
                println!("   URL: {}", item.url);
            }
        }
        Commands::Info => {
            let (count, latest) = get_inbox_summary(&cli.mailbox, config.clone())?;
            println!("Mailbox: {}@yopmail.com", cli.mailbox);
            println!("Messages: {}", count);
            if let Some(msg) = latest {
                println!("Latest: {}", msg.subject);
                if let Some(sender) = msg.sender {
                    println!("From: {}", sender);
                }
            }
        }
    }

    Ok(())
}

fn build_config(proxy: Option<String>) -> Option<yopmail_client_rs::models::Config> {
    let mut cfg = yopmail_client_rs::models::Config::default();
    cfg.proxy_url = proxy;
    Some(cfg)
}
