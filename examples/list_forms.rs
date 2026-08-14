//! List forms and print aggregate statistics.
//!
//! Requires an API key with the `"forms"` scope, read from `PAUBOX_API_KEY`,
//! and the customer ID the key belongs to, read from `PAUBOX_CUSTOMER_ID`
//! (the server rejects `list_forms` requests without a `customer_id`).
//!
//! ```sh
//! export PAUBOX_API_KEY="your-key"
//! export PAUBOX_CUSTOMER_ID="42"
//! cargo run --example list_forms
//! ```

use paubox::forms::{FormListParams, FormsClient};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("PAUBOX_API_KEY").expect("PAUBOX_API_KEY must be set");
    let customer_id: i64 = env::var("PAUBOX_CUSTOMER_ID")
        .expect("PAUBOX_CUSTOMER_ID must be set")
        .parse()
        .expect("PAUBOX_CUSTOMER_ID must be an integer");

    let client = FormsClient::with_api_key(api_key);

    // List active, non-archived forms, most recently updated first.
    // `customer_id` is required for API-key callers.
    let params = FormListParams::default()
        .customer_id(customer_id)
        .active(true)
        .archived(false)
        .order_by("updated_at")
        .order("desc")
        .items(25);

    let page = client.list_forms(&params).await?;

    println!(
        "Page {}/{} — {} forms total",
        page.page_info.page, page.page_info.pages, page.page_info.count
    );
    for form in &page.results {
        println!(
            "  {}  {:40}  submissions: {}",
            form.id, form.title, form.submission_count
        );
    }

    // Aggregate stats for the customer that owns the API key.
    let stats = client.form_stats(None).await?;
    println!();
    println!("Active forms:            {}", stats.active_form_count);
    println!("Total submissions:       {}", stats.total_submission_count);
    println!("Submissions last 7 days: {}", stats.submissions_last_7_days);

    Ok(())
}
