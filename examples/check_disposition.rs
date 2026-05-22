//! Check the delivery status of a previously sent email.
//!
//! ```sh
//! export PAUBOX_API_KEY="your-api-key"
//! export PAUBOX_API_USER="your-api-user"
//! cargo run --example check_disposition -- <source_tracking_id>
//! ```

use paubox::PauboxClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tracking_id = env::args()
        .nth(1)
        .expect("Usage: check_disposition <source_tracking_id>");

    let client = PauboxClient::from_env()?;

    println!("Fetching disposition for: {tracking_id}");
    let resp = client.get_email_disposition(&tracking_id).await?;

    println!("Source tracking ID: {}", resp.source_tracking_id);
    if resp.message_deliveries.is_empty() {
        println!("No delivery records yet.");
    } else {
        for delivery in &resp.message_deliveries {
            println!(
                "  {} — status: {}, opened: {}",
                delivery.recipient, delivery.delivery_status, delivery.opened_status
            );
            if let Some(ref t) = delivery.delivery_time {
                println!("    Delivered at: {t}");
            }
            if let Some(ref t) = delivery.opened_time {
                println!("    Opened at:    {t}");
            }
        }
    }

    if !resp.errors.is_empty() {
        for err in &resp.errors {
            eprintln!("API error: {:?}", err);
        }
    }

    Ok(())
}
