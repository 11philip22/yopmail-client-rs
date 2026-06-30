use yopmail_client::{DEFAULT_DOMAIN, Error, YopmailClient, generate_random_mailbox};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mailbox = generate_random_mailbox(12);
    let address = format!("{mailbox}@{DEFAULT_DOMAIN}");
    let mut client = YopmailClient::new(&mailbox)?;

    client.open_inbox().await?;
    client
        .send_message(
            &address,
            "yopmail-client self-send",
            "Hello from yopmail-client.",
        )
        .await?;

    println!("Sent test message to {address}");
    Ok(())
}
