//! Send a HIPAA-compliant email via the Paubox Email API.
//!
//! Set the following environment variables before running:
//!
//! ```sh
//! export PAUBOX_API_KEY="your-api-key"
//! export PAUBOX_API_USER="your-api-user"
//! cargo run --example send_email
//! ```

use paubox::{email::Message, PauboxClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PauboxClient::from_env()?;

    let msg = Message::builder()
        .from("sender@yourdomain.com")
        .to(["recipient@example.com"])
        .subject("Secure message from Paubox")
        .text_content("This is a HIPAA-compliant message sent via the Paubox Rust SDK.")
        .html_content("<p>This is a <strong>HIPAA-compliant</strong> message sent via the Paubox Rust SDK.</p>")
        .build()?;

    println!("Sending message...");
    let resp = client.send_message(&msg).await?;
    println!("Message sent!");
    println!("Tracking ID: {}", resp.source_tracking_id);
    println!("API message: {}", resp.message);

    Ok(())
}
