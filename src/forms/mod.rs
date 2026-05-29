//! Paubox Forms API — retrieve form definitions and submit responses.
//!
//! The Forms API endpoints are **public** (no API key required) and are
//! intended to be called on behalf of form respondents.
//!
//! # Example
//! ```no_run
//! use paubox::forms::{FormsClient, FormSubmission};
//! use serde_json::json;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = FormsClient::new();
//!
//! // Retrieve a form definition
//! let form = client.get_form("550e8400-e29b-41d4-a716-446655440000").await?;
//! println!("Form title: {}", form.title);
//!
//! // Submit a response
//! let submission = FormSubmission::builder()
//!     .form_data(json!({"first_name": "Jane", "last_name": "Doe"}))
//!     .build()?;
//! client.submit_form("550e8400-e29b-41d4-a716-446655440000", &submission).await?;
//! # Ok(())
//! # }
//! ```

pub mod form;
pub mod submission;

pub use form::Form;
pub use submission::{FormAttachment, FormSubmission, FormSubmissionBuilder};

use url::Url;

use crate::client::{ensure_trailing_slash, FORMS_BASE_URL};
use crate::error::PauboxError;

/// Client for the Paubox Forms API.
///
/// The Forms API does not require authentication.  Create a standalone
/// instance with [`FormsClient::new`], or obtain one from an existing
/// [`crate::PauboxClient`] via [`crate::PauboxClient::forms`] (which reuses
/// the underlying connection pool).
#[derive(Debug, Clone)]
pub struct FormsClient {
    http: reqwest::Client,
    base_url: Url,
}

impl FormsClient {
    /// Create a new `FormsClient` using the default Forms API base URL.
    pub fn new() -> Self {
        Self::with_http(reqwest::Client::new())
    }

    /// Create a `FormsClient` with a custom base URL.
    ///
    /// Primarily useful for tests that point at a mock server.
    pub fn with_base_url(mut base_url: Url) -> Self {
        ensure_trailing_slash(&mut base_url);
        Self {
            http: reqwest::Client::new(),
            base_url,
        }
    }

    /// Create a `FormsClient` that reuses an existing `reqwest::Client`.
    ///
    /// Called internally by [`crate::PauboxClient::forms`].
    pub(crate) fn with_http(http: reqwest::Client) -> Self {
        let mut base_url = Url::parse(FORMS_BASE_URL).expect("hardcoded URL is valid");
        ensure_trailing_slash(&mut base_url);
        Self { http, base_url }
    }

    /// Retrieve a form definition by its UUID.
    ///
    /// # Errors
    /// - [`PauboxError::Http`] — form not found (404) or server error
    /// - [`PauboxError::Request`] — network failure
    /// - [`PauboxError::Deserialize`] — unexpected response shape
    ///
    /// # Example
    /// ```no_run
    /// use paubox::forms::FormsClient;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = FormsClient::new();
    /// let form = client.get_form("550e8400-e29b-41d4-a716-446655440000").await?;
    /// println!("{}: active={}", form.title, form.active);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_form(&self, form_id: &str) -> Result<Form, PauboxError> {
        let url = self
            .base_url
            .join(&format!("public/form_data/{}", form_id))?;

        let resp = self
            .http
            .get(url)
            .header("Accept", "application/json")
            .send()
            .await?;

        handle_response::<Form>(resp).await
    }

    /// Submit a respondent's answers for a form.
    ///
    /// Returns `Ok(())` on success (HTTP 201).  The maximum request size is
    /// 250 MB (including attachments).
    ///
    /// # Errors
    /// - [`PauboxError::Validation`] — `form_data` is null or empty (validated
    ///   before the network call)
    /// - [`PauboxError::Http`] — form not found (404), bad request (400), or
    ///   server error
    /// - [`PauboxError::Request`] — network failure
    ///
    /// # Example
    /// ```no_run
    /// use paubox::forms::{FormsClient, FormSubmission};
    /// use serde_json::json;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = FormsClient::new();
    /// let submission = FormSubmission::builder()
    ///     .form_data(json!({"email": "jane@example.com", "consent": true}))
    ///     .build()?;
    /// client.submit_form("550e8400-e29b-41d4-a716-446655440000", &submission).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn submit_form(
        &self,
        form_id: &str,
        submission: &FormSubmission,
    ) -> Result<(), PauboxError> {
        let url = self
            .base_url
            .join(&format!("api/forms/{}/submissions", form_id))?;

        let resp = self
            .http
            .post(url)
            .header("Content-Type", "application/json")
            .json(submission)
            .send()
            .await?;

        let status = resp.status().as_u16();
        if resp.status().is_success() {
            Ok(())
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(PauboxError::Http { status, body })
        }
    }
}

impl Default for FormsClient {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Shared response handler
// ---------------------------------------------------------------------------

async fn handle_response<T>(resp: reqwest::Response) -> Result<T, PauboxError>
where
    T: serde::de::DeserializeOwned,
{
    let status = resp.status();
    if status.is_success() {
        let text = resp.text().await?;
        let parsed = serde_json::from_str::<T>(&text)?;
        Ok(parsed)
    } else {
        let status_u16 = status.as_u16();
        let body = resp.text().await.unwrap_or_default();
        Err(PauboxError::Http {
            status: status_u16,
            body,
        })
    }
}
