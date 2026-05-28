//! Core client: credential storage and shared HTTP transport.

#[cfg(feature = "email")]
use std::env;
#[cfg(feature = "email")]
use url::Url;

#[cfg(feature = "email")]
use crate::error::PauboxError;

#[cfg(all(feature = "email", feature = "forms"))]
use crate::forms::FormsClient;

#[cfg(feature = "email")]
const DEFAULT_EMAIL_BASE: &str = "https://api.paubox.net/v1/";
#[cfg(feature = "forms")]
const DEFAULT_FORMS_BASE: &str = "https://apx.paubox.com/forms";

/// Client for the Paubox Email API.
///
/// Holds API credentials and a shared [`reqwest::Client`] for connection
/// pooling.  Create once and reuse across requests.
///
/// # Example
/// ```no_run
/// use paubox::PauboxClient;
///
/// let client = PauboxClient::new("my-api-key", "my-api-user");
/// ```
#[cfg(feature = "email")]
#[derive(Debug, Clone)]
pub struct PauboxClient {
    pub(crate) api_key: String,
    pub(crate) http: reqwest::Client,
    /// Per-customer base URL: `https://api.paubox.net/v1/{api_user}`.
    pub(crate) base_url: Url,
}

#[cfg(feature = "email")]
impl PauboxClient {
    /// Create a new client with the given API key and API user (endpoint name).
    ///
    /// The base URL defaults to `https://api.paubox.net/v1/{api_user}`.
    ///
    /// # Panics
    /// Panics if the default base URL cannot be constructed from the provided
    /// `api_user` string (e.g. it contains characters that are invalid in a
    /// URL path segment).  Use [`PauboxClient::builder`] for fallible
    /// construction.
    pub fn new(api_key: impl Into<String>, api_user: impl Into<String>) -> Self {
        let api_key = api_key.into();
        let api_user = api_user.into();
        let base_url = Url::parse(&format!("{}{}", DEFAULT_EMAIL_BASE, api_user))
            .expect("invalid api_user for URL construction");
        Self {
            api_key,
            http: reqwest::Client::new(),
            base_url,
        }
    }

    /// Create a [`PauboxClientBuilder`] for configuring optional parameters
    /// such as a custom base URL or request timeout.
    pub fn builder() -> PauboxClientBuilder {
        PauboxClientBuilder::default()
    }

    /// Create a client from environment variables.
    ///
    /// Reads:
    /// - `PAUBOX_API_KEY` — your API key
    /// - `PAUBOX_API_USER` — your API user / endpoint name
    ///
    /// # Errors
    /// Returns [`PauboxError::EnvVar`] if either variable is absent or empty.
    pub fn from_env() -> Result<Self, PauboxError> {
        let api_key = env_required("PAUBOX_API_KEY")?;
        let api_user = env_required("PAUBOX_API_USER")?;
        Ok(Self::new(api_key, api_user))
    }

    /// Return a [`FormsClient`] that reuses this client's HTTP connection pool.
    ///
    /// The Forms API requires no authentication; this is a convenience accessor.
    #[cfg(feature = "forms")]
    pub fn forms(&self) -> FormsClient {
        FormsClient::with_http(self.http.clone())
    }

    /// Authorization header value: `Token token={api_key}`.
    pub(crate) fn auth_header(&self) -> String {
        format!("Token token={}", self.api_key)
    }
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// Builder for [`PauboxClient`] supporting optional overrides.
///
/// # Example
/// ```no_run
/// use paubox::PauboxClient;
/// use url::Url;
/// use std::time::Duration;
///
/// let client = PauboxClient::builder()
///     .api_key("my-key")
///     .api_user("my-user")
///     .timeout(Duration::from_secs(30))
///     .build()
///     .unwrap();
/// ```
#[cfg(feature = "email")]
#[derive(Debug, Default)]
pub struct PauboxClientBuilder {
    api_key: Option<String>,
    api_user: Option<String>,
    base_url: Option<Url>,
    timeout: Option<std::time::Duration>,
}

#[cfg(feature = "email")]
impl PauboxClientBuilder {
    /// Set the API key.
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Set the API user (endpoint name).
    pub fn api_user(mut self, user: impl Into<String>) -> Self {
        self.api_user = Some(user.into());
        self
    }

    /// Override the Email API base URL (useful for testing).
    pub fn base_url(mut self, url: Url) -> Self {
        self.base_url = Some(url);
        self
    }

    /// Set a request timeout applied to every HTTP call.
    pub fn timeout(mut self, duration: std::time::Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Consume the builder and produce a [`PauboxClient`].
    ///
    /// # Errors
    /// Returns [`PauboxError::Validation`] if `api_key` or `api_user` were not
    /// set, or [`PauboxError::Url`] if the base URL cannot be constructed.
    pub fn build(self) -> Result<PauboxClient, PauboxError> {
        let api_key = self
            .api_key
            .ok_or_else(|| PauboxError::Validation("api_key is required".into()))?;
        let api_user = self
            .api_user
            .ok_or_else(|| PauboxError::Validation("api_user is required".into()))?;

        let base_url = match self.base_url {
            Some(u) => u,
            None => Url::parse(&format!("{}{}", DEFAULT_EMAIL_BASE, api_user))?,
        };

        let mut builder = reqwest::Client::builder();
        if let Some(t) = self.timeout {
            builder = builder.timeout(t);
        }
        let http = builder.build().map_err(PauboxError::Request)?;

        Ok(PauboxClient {
            api_key,
            http,
            base_url,
        })
    }
}

/// Return env var value or `PauboxError::EnvVar` if absent/empty.
#[cfg(feature = "email")]
pub(crate) fn env_required(name: &str) -> Result<String, PauboxError> {
    env::var(name)
        .ok()
        .filter(|v| !v.is_empty())
        .ok_or_else(|| PauboxError::EnvVar(name.to_owned()))
}

/// Default Forms base URL constant, re-exported for `FormsClient`.
#[cfg(feature = "forms")]
pub(crate) const FORMS_BASE_URL: &str = DEFAULT_FORMS_BASE;
