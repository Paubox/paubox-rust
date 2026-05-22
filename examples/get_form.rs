//! Retrieve a Paubox form definition by UUID.
//!
//! No API key is required for the Forms API.
//!
//! ```sh
//! cargo run --example get_form -- <form_uuid>
//! ```

use paubox::forms::FormsClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let form_id = env::args()
        .nth(1)
        .expect("Usage: get_form <form_uuid>");

    let client = FormsClient::new();

    println!("Fetching form: {form_id}");
    let form = client.get_form(&form_id).await?;

    println!("Title:            {}", form.title);
    println!("Active:           {}", form.active);
    println!("Signable:         {}", form.signable);
    println!("Submissions:      {}", form.submission_count);
    println!("Created at:       {}", form.created_at);
    if let Some(ref desc) = form.description {
        println!("Description:      {desc}");
    }
    if let Some(ref schema) = form.form_json {
        println!("Schema fields:    {}", serde_json::to_string_pretty(schema)?);
    }

    Ok(())
}
