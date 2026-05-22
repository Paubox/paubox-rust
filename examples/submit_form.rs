//! Submit a response to a Paubox form.
//!
//! No API key is required for the Forms API.
//!
//! ```sh
//! cargo run --example submit_form -- <form_uuid>
//! ```
//!
//! The example submits a hardcoded `form_data` payload; adjust the JSON to
//! match the actual schema of your form.

use paubox::forms::{FormsClient, FormSubmission};
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let form_id = env::args()
        .nth(1)
        .expect("Usage: submit_form <form_uuid>");

    let client = FormsClient::new();

    let submission = FormSubmission::builder()
        .form_data(json!({
            "first_name": "Jane",
            "last_name":  "Doe",
            "email":      "jane.doe@example.com",
            "consent":    true
        }))
        .build()?;

    println!("Submitting form {form_id}...");
    client.submit_form(&form_id, &submission).await?;
    println!("Submission accepted (201).");

    Ok(())
}
