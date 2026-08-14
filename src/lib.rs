//! # paubox
//!
//! Async Rust SDK for the [Paubox](https://www.paubox.com) Email and Forms
//! APIs.  Paubox is a HITRUST-certified platform for sending HIPAA-compliant
//! email and collecting patient data through secure forms.
//!
//! ## Features
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `email` | ✓ | Paubox Email API (send messages, track delivery) |
//! | `forms` | ✓ | Paubox Forms API (manage forms, list/export submissions, submit responses) |
//!
//! ## Quick start
//!
//! ### Send an email
//!
//! ```no_run
//! use paubox::{PauboxClient, email::Message};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = PauboxClient::new("YOUR_API_KEY");
//!
//!     let msg = Message::builder()
//!         .from("you@yourdomain.com")
//!         .to(["patient@example.com"])
//!         .subject("Your lab results are ready")
//!         .text_content("Please log in to the patient portal to view your results.")
//!         .build()?;
//!
//!     let resp = client.send_message(&msg).await?;
//!     println!("Sent! Tracking ID: {}", resp.source_tracking_id);
//!
//!     let disposition = client.get_email_disposition(&resp.source_tracking_id).await?;
//!     for d in &disposition.message_deliveries {
//!         println!("{}: {} ({})", d.recipient, d.delivery_status, d.opened_status);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ### Submit a form
//!
//! ```no_run
//! use paubox::forms::{FormsClient, FormSubmission};
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = FormsClient::new();
//!
//!     let form = client.get_form("your-form-uuid").await?;
//!     println!("Submitting: {}", form.title);
//!
//!     let submission = FormSubmission::builder()
//!         .form_data(json!({"first_name": "Jane", "last_name": "Doe"}))
//!         .build()?;
//!     client.submit_form("your-form-uuid", &submission).await?;
//!     Ok(())
//! }
//! ```
//!
//! ### Manage forms with a scoped API key
//!
//! Administrative Forms endpoints (list/create/update/archive/copy forms,
//! stats, and submission listing/export) require an API key carrying the
//! `forms` scope, sent as a Bearer token:
//!
//! ```no_run
//! use paubox::forms::{FormListParams, FormsClient};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = FormsClient::with_api_key("key-with-forms-scope");
//!
//!     // `customer_id` is required for API-key callers.
//!     let page = client
//!         .list_forms(&FormListParams::default().customer_id(42).active(true))
//!         .await?;
//!     for form in &page.results {
//!         println!("{} ({} submissions)", form.title, form.submission_count);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ### Credentials from environment variables
//!
//! ```no_run
//! use paubox::PauboxClient;
//!
//! // Reads PAUBOX_API_KEY
//! let client = PauboxClient::from_env().expect("credentials not set");
//! ```

#[cfg(not(any(feature = "email", feature = "forms")))]
compile_error!(
    "the `paubox` crate requires at least one of the `email` or `forms` features to be enabled"
);

pub mod client;
pub mod error;

#[cfg(feature = "email")]
pub mod email;

#[cfg(feature = "forms")]
pub mod forms;

#[cfg(feature = "email")]
pub use client::{PauboxClient, PauboxClientBuilder};
pub use error::PauboxError;
