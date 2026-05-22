//! Error types for the Paubox SDK.

use thiserror::Error;

/// All errors that can be returned by Paubox SDK methods.
#[derive(Debug, Error)]
pub enum PauboxError {
    /// The server returned a non-2xx HTTP status.
    ///
    /// `status` is the numeric status code; `body` is the raw response body
    /// (may be empty for network-level errors).
    #[error("HTTP {status}: {body}")]
    Http {
        /// HTTP status code returned by the server.
        status: u16,
        /// Raw response body.
        body: String,
    },

    /// The request was rejected due to invalid or missing credentials (HTTP 401).
    #[error("authentication error: {0}")]
    Auth(String),

    /// An error occurred while executing the HTTP request.
    #[error("request error: {0}")]
    Request(#[from] reqwest::Error),

    /// The server returned a response body that could not be deserialized.
    #[error("deserialization error: {0}")]
    Deserialize(#[from] serde_json::Error),

    /// A URL could not be parsed or constructed.
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),

    /// A required environment variable was absent or empty.
    #[error("missing environment variable: {0}")]
    EnvVar(String),

    /// A method argument failed validation before any network call was made.
    #[error("validation error: {0}")]
    Validation(String),
}
