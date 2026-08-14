//! Paubox Forms API — manage forms, retrieve definitions, and handle
//! submissions.
//!
//! # Authentication
//!
//! The Forms API uses **scoped API keys**.  Administrative endpoints —
//! listing, creating, updating, archiving, copying forms, retrieving stats,
//! and listing or exporting submissions — require an API key carrying the
//! `"forms"` scope, sent as a Bearer token (`Authorization: Bearer
//! <api_key>`).  Construct an authenticated client with
//! [`FormsClient::with_api_key`] or [`FormsClient::builder`].
//!
//! The server returns 401 for an invalid or unscoped key (surfaced as
//! [`PauboxError::Auth`]) and 403 for a key that belongs to a different
//! customer (surfaced as [`PauboxError::Http`]).  Calling a protected method
//! on a client that has no API key fails with [`PauboxError::Auth`] before
//! any network request is made.
//!
//! Two endpoints remain **public** (no API key required) and are intended to
//! be called on behalf of form respondents: [`FormsClient::get_form`] and
//! [`FormsClient::submit_form`].
//!
//! # Example
//! ```no_run
//! use paubox::forms::{FormsClient, FormSubmission};
//! use serde_json::json;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Public endpoints need no API key.
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
//!
//! // Administrative endpoints require a scoped API key.
//! let admin = FormsClient::with_api_key("key-with-forms-scope");
//! let stats = admin.form_stats(None).await?;
//! println!("Active forms: {}", stats.active_form_count);
//! # Ok(())
//! # }
//! ```

pub mod admin;
pub mod form;
pub mod submission;

pub use admin::{
    CreateForm, CreateFormBuilder, FormListParams, FormPage, FormStats, PageInfo, Submission,
    SubmissionListParams, SubmissionPage, UpdateForm,
};
pub use form::Form;
pub use submission::{FormAttachment, FormSubmission, FormSubmissionBuilder};

use url::Url;

use crate::client::{ensure_trailing_slash, FORMS_BASE_URL};
use crate::error::PauboxError;

/// Client for the Paubox Forms API.
///
/// The public endpoints ([`FormsClient::get_form`] and
/// [`FormsClient::submit_form`]) require no authentication; every other
/// method requires an API key carrying the `"forms"` scope (see the
/// [module docs](self)).
///
/// Create an unauthenticated instance with [`FormsClient::new`], an
/// authenticated one with [`FormsClient::with_api_key`] or
/// [`FormsClient::builder`], or obtain one from an existing
/// [`crate::PauboxClient`] via [`crate::PauboxClient::forms`] (which reuses
/// the underlying connection pool).
#[derive(Debug, Clone)]
pub struct FormsClient {
    http: reqwest::Client,
    base_url: Url,
    api_key: Option<String>,
}

impl FormsClient {
    /// Create a new unauthenticated `FormsClient` using the default Forms API
    /// base URL.
    ///
    /// Only the public endpoints work on a client created this way; use
    /// [`FormsClient::with_api_key`] or [`FormsClient::builder`] for
    /// administrative endpoints.
    pub fn new() -> Self {
        Self::with_http(reqwest::Client::new())
    }

    /// Create a `FormsClient` authenticated with a scoped API key.
    ///
    /// The key is sent as `Authorization: Bearer <api_key>` on protected
    /// endpoints and must carry the `"forms"` scope.
    ///
    /// # Example
    /// ```no_run
    /// use paubox::forms::FormsClient;
    ///
    /// let client = FormsClient::with_api_key("key-with-forms-scope");
    /// ```
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        let mut client = Self::with_http(reqwest::Client::new());
        client.api_key = Some(api_key.into());
        client
    }

    /// Create a [`FormsClientBuilder`] for configuring optional parameters
    /// such as an API key, a custom base URL, or a request timeout.
    pub fn builder() -> FormsClientBuilder {
        FormsClientBuilder::default()
    }

    /// Create an unauthenticated `FormsClient` with a custom base URL.
    ///
    /// Primarily useful for tests that point at a mock server.  Use
    /// [`FormsClient::builder`] to combine a custom base URL with an API key.
    pub fn with_base_url(mut base_url: Url) -> Self {
        ensure_trailing_slash(&mut base_url);
        Self {
            http: reqwest::Client::new(),
            base_url,
            api_key: None,
        }
    }

    /// Create a `FormsClient` that reuses an existing `reqwest::Client`.
    ///
    /// Called internally by [`crate::PauboxClient::forms`].
    pub(crate) fn with_http(http: reqwest::Client) -> Self {
        let mut base_url = Url::parse(FORMS_BASE_URL).expect("hardcoded URL is valid");
        ensure_trailing_slash(&mut base_url);
        Self {
            http,
            base_url,
            api_key: None,
        }
    }

    /// Authorization header value for protected endpoints, or
    /// [`PauboxError::Auth`] when the client has no API key.
    ///
    /// Called before any network request so a missing key fails fast.
    fn bearer_auth(&self) -> Result<String, PauboxError> {
        match &self.api_key {
            Some(key) => Ok(format!("Bearer {}", key)),
            None => Err(PauboxError::Auth(
                "this endpoint requires an API key with the \"forms\" scope; \
                 construct the client with FormsClient::with_api_key or \
                 FormsClient::builder().api_key(...)"
                    .into(),
            )),
        }
    }

    /// Retrieve a form definition by its UUID.
    ///
    /// This is a public endpoint — no API key required.
    ///
    /// # Errors
    /// - [`PauboxError::Http`] — form not found (404) or server error
    /// - [`PauboxError::Auth`] — the server responded 401 (not expected on
    ///   this public endpoint, but mapped consistently with the
    ///   authenticated methods)
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
    /// This is a public endpoint — no API key required.
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
// Builder
// ---------------------------------------------------------------------------

/// Builder for [`FormsClient`] supporting optional overrides.
///
/// # Example
/// ```no_run
/// use paubox::forms::FormsClient;
/// use std::time::Duration;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = FormsClient::builder()
///     .api_key("key-with-forms-scope")
///     .timeout(Duration::from_secs(30))
///     .build()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Default)]
pub struct FormsClientBuilder {
    api_key: Option<String>,
    base_url: Option<Url>,
    timeout: Option<std::time::Duration>,
}

impl FormsClientBuilder {
    /// Set the scoped API key used for administrative endpoints.
    ///
    /// The key must carry the `"forms"` scope.  Omitting it produces an
    /// unauthenticated client on which only the public endpoints work.
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Override the Forms API base URL (useful for testing).
    pub fn base_url(mut self, url: Url) -> Self {
        self.base_url = Some(url);
        self
    }

    /// Set a request timeout applied to every HTTP call.
    pub fn timeout(mut self, duration: std::time::Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Consume the builder and produce a [`FormsClient`].
    ///
    /// # Errors
    /// - [`PauboxError::Url`] — the default base URL cannot be parsed
    /// - [`PauboxError::Request`] — the underlying HTTP client cannot be
    ///   constructed
    pub fn build(self) -> Result<FormsClient, PauboxError> {
        let mut base_url = match self.base_url {
            Some(u) => u,
            None => Url::parse(FORMS_BASE_URL)?,
        };
        ensure_trailing_slash(&mut base_url);

        let mut builder = reqwest::Client::builder();
        if let Some(t) = self.timeout {
            builder = builder.timeout(t);
        }
        let http = builder.build().map_err(PauboxError::Request)?;

        Ok(FormsClient {
            http,
            base_url,
            api_key: self.api_key,
        })
    }
}

// ---------------------------------------------------------------------------
// Shared response handlers
// ---------------------------------------------------------------------------

/// Deserialize a JSON success body, mapping 401 to [`PauboxError::Auth`] and
/// any other non-2xx status to [`PauboxError::Http`].
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
        Err(error_for(status.as_u16(), resp).await)
    }
}

/// Discard a success body, mapping non-2xx statuses like [`handle_response`].
async fn handle_unit(resp: reqwest::Response) -> Result<(), PauboxError> {
    let status = resp.status();
    if status.is_success() {
        Ok(())
    } else {
        Err(error_for(status.as_u16(), resp).await)
    }
}

/// Return a raw success body (CSV / PDF exports), mapping non-2xx statuses
/// like [`handle_response`].
async fn handle_bytes(resp: reqwest::Response) -> Result<Vec<u8>, PauboxError> {
    let status = resp.status();
    if status.is_success() {
        Ok(resp.bytes().await?.to_vec())
    } else {
        Err(error_for(status.as_u16(), resp).await)
    }
}

/// Map a non-2xx response to the appropriate [`PauboxError`] variant:
/// 401 becomes [`PauboxError::Auth`], everything else [`PauboxError::Http`].
async fn error_for(status: u16, resp: reqwest::Response) -> PauboxError {
    let body = resp.text().await.unwrap_or_default();
    if status == 401 {
        PauboxError::Auth(body)
    } else {
        PauboxError::Http { status, body }
    }
}
