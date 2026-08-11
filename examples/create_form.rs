//! Create a form, then activate it via a partial update.
//!
//! Requires an API key with the `"forms"` scope, read from `PAUBOX_API_KEY`.
//!
//! ```sh
//! export PAUBOX_API_KEY="your-key"
//! cargo run --example create_form -- <customer_id>
//!
//! # Also exercise the archive/unarchive round trip on the new form:
//! cargo run --example create_form -- <customer_id> --archive-roundtrip
//! ```

use paubox::forms::{CreateForm, FormsClient, UpdateForm};
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("PAUBOX_API_KEY").expect("PAUBOX_API_KEY must be set");
    let customer_id: i64 = env::args()
        .nth(1)
        .expect("Usage: create_form <customer_id> [--archive-roundtrip]")
        .parse()
        .expect("customer_id must be an integer");
    let archive_roundtrip = env::args().any(|a| a == "--archive-roundtrip");

    let client = FormsClient::with_api_key(api_key);

    // Build a minimal form: title, customer_id, and form_json are required;
    // version defaults to 1.
    let form = CreateForm::builder()
        .title("Patient intake (SDK example)")
        .customer_id(customer_id)
        .form_json(json!({
            "fields": [
                {"name": "first_name", "type": "text",  "label": "First name"},
                {"name": "last_name",  "type": "text",  "label": "Last name"},
                {"name": "email",      "type": "email", "label": "Email"}
            ]
        }))
        .description("Created by the paubox-rust create_form example")
        .build()?;

    let form_id = client.create_form(&form).await?;
    println!("Created form: {form_id}");

    // Forms are created inactive by default; activate with a partial update.
    // Fields left unset on UpdateForm keep their current values.
    let update = UpdateForm::default().active(true);
    client.update_form(&form_id, &update).await?;
    println!("Form activated.");

    let fetched = client.get_form_by_id(&form_id).await?;
    println!(
        "Fetched back: {} (active: {}, version: {})",
        fetched.title, fetched.active, fetched.version
    );

    // Optional archive/unarchive round trip, gated behind a CLI flag so the
    // default run leaves the new form active.
    if archive_roundtrip {
        client.archive_form(&form_id).await?;
        println!("Form archived.");

        client.unarchive_form(&form_id).await?;
        println!("Form unarchived.");
    }

    Ok(())
}
