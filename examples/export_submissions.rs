//! List a form's submissions and export them as CSV (and optionally PDF).
//!
//! Requires an API key with the `"forms"` scope, read from `PAUBOX_API_KEY`.
//!
//! ```sh
//! export PAUBOX_API_KEY="your-key"
//!
//! # List submissions and write submissions_<form_uuid>.csv:
//! cargo run --example export_submissions -- <form_uuid>
//!
//! # Additionally write submission_<submission_uuid>.pdf for one submission:
//! cargo run --example export_submissions -- <form_uuid> <submission_uuid>
//! ```

use paubox::forms::{FormsClient, SubmissionListParams};
use std::env;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("PAUBOX_API_KEY").expect("PAUBOX_API_KEY must be set");
    let form_id = env::args()
        .nth(1)
        .expect("Usage: export_submissions <form_uuid> [submission_uuid]");
    let submission_id = env::args().nth(2);

    let client = FormsClient::with_api_key(api_key);

    // List the most recent submissions.
    let params = SubmissionListParams::default().order("desc").items(50);
    let page = client.list_submissions(&form_id, &params).await?;

    println!(
        "{} submissions total; showing {}:",
        page.total,
        page.data.len()
    );
    for sub in &page.data {
        println!(
            "  {}  {}  {}",
            sub.id,
            sub.created_at,
            sub.submitter_email.as_deref().unwrap_or("<no email>")
        );
        // form_data is a JSON-encoded string; parse it on demand.
        if let Ok(data) = sub.form_data_json() {
            println!("    {data}");
        }
    }

    // Export every submission as one CSV file.
    let csv = client.export_submissions_csv(&form_id).await?;
    let csv_path = format!("submissions_{form_id}.csv");
    fs::write(&csv_path, csv)?;
    println!("Wrote CSV export to {csv_path}");

    // Optionally export a single submission as PDF.
    if let Some(submission_id) = submission_id {
        let pdf = client
            .export_submission_pdf(&form_id, &submission_id)
            .await?;
        let pdf_path = format!("submission_{submission_id}.pdf");
        fs::write(&pdf_path, pdf)?;
        println!("Wrote PDF export to {pdf_path}");
    }

    Ok(())
}
