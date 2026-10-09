//! Paubox webhooks — manage which URL Paubox notifies, for which events.
//!
//! Authenticates with a **scoped API key** sent as `Authorization: Bearer
//! <key>`, which is the same scheme as the Forms API and a different one from
//! the Email API's `Token token=`. Which events a key may subscribe to is
//! decided by its scopes: asking for one it is not scoped for is refused with
//! 403, and an unrecognised event with 422.
//!
//! # Example
//! ```no_run
//! use paubox::webhooks::{UpdateWebhookEndpoint, WebhooksClient};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = WebhooksClient::new("your-scoped-api-key");
//!
//! let created = client
//!     .create_webhook_endpoint("https://hooks.example.com/paubox", &["forms.submission.created"])
//!     .await?;
//! // The service returns the signing secret only here — store it now.
//! println!("signing secret: {}", created.signing_secret);
//!
//! client
//!     .update_webhook_endpoint(
//!         &created.endpoint.id,
//!         UpdateWebhookEndpoint {
//!             status: Some("disabled".to_string()),
//!             ..Default::default()
//!         },
//!     )
//!     .await?;
//! # Ok(())
//! # }
//! ```

pub mod types;

pub use types::{
    CreatedWebhookEndpoint, ListWebhookEndpointsParams, PageInfo, UpdateWebhookEndpoint,
    WebhookEndpoint, WebhookEndpointList,
};

use url::Url;

use crate::client::{ensure_trailing_slash, WEBHOOKS_BASE_URL};
use crate::error::PauboxError;
use types::{CreateWebhookEndpointRequest, DataEnvelope};

/// Client for the Paubox webhooks service.
///
/// Every endpoint here is authenticated, so unlike [`crate::forms::FormsClient`]
/// there is no keyless mode.
#[derive(Debug, Clone)]
pub struct WebhooksClient {
    http: reqwest::Client,
    base_url: Url,
    api_key: String,
}

impl WebhooksClient {
    /// Create a client using the default base URL.
    pub fn new(api_key: impl Into<String>) -> Self {
        let mut base_url = Url::parse(WEBHOOKS_BASE_URL).expect("hardcoded URL is valid");
        ensure_trailing_slash(&mut base_url);
        Self {
            http: reqwest::Client::new(),
            base_url,
            api_key: api_key.into(),
        }
    }

    /// Create a client with a custom base URL.
    ///
    /// Primarily useful for tests that point at a mock server.
    pub fn with_base_url(api_key: impl Into<String>, mut base_url: Url) -> Self {
        ensure_trailing_slash(&mut base_url);
        Self {
            http: reqwest::Client::new(),
            base_url,
            api_key: api_key.into(),
        }
    }

    /// Create a client that reuses an existing [`reqwest::Client`].
    pub fn with_http(api_key: impl Into<String>, http: reqwest::Client) -> Self {
        let mut base_url = Url::parse(WEBHOOKS_BASE_URL).expect("hardcoded URL is valid");
        ensure_trailing_slash(&mut base_url);
        Self {
            http,
            base_url,
            api_key: api_key.into(),
        }
    }

    /// Authorization header value: `Bearer {api_key}`.
    fn auth_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }

    fn endpoints_url(&self) -> Result<Url, PauboxError> {
        Ok(self.base_url.join("endpoints")?)
    }

    /// Validates an id before it reaches a URL.
    ///
    /// Without this a value containing `..` or `/` would change which endpoint
    /// is called, and since the Authorization header goes on the same host the
    /// key would ride along on the retargeted request.
    fn endpoint_url(&self, id: &str) -> Result<Url, PauboxError> {
        if id.is_empty() {
            return Err(PauboxError::Validation("endpoint id is required".into()));
        }
        if uuid_shaped(id) {
            Ok(self.base_url.join(&format!("endpoints/{id}"))?)
        } else {
            Err(PauboxError::Validation(format!(
                "endpoint id must be a UUID, got {id:?}"
            )))
        }
    }

    /// List the endpoints this key can act on.
    ///
    /// Endpoints carrying an event the key is not scoped for are filtered out
    /// by the service, so the result is what this key may manage rather than
    /// everything on the account.
    ///
    /// # Errors
    /// - [`PauboxError::Auth`] — the key was rejected (401)
    /// - [`PauboxError::Http`] — any other non-2xx response
    pub async fn list_webhook_endpoints(
        &self,
        params: &ListWebhookEndpointsParams,
    ) -> Result<WebhookEndpointList, PauboxError> {
        let mut url = self.endpoints_url()?;
        {
            let mut query = url.query_pairs_mut();
            if let Some(page) = params.page {
                query.append_pair("page", &page.to_string());
            }
            if let Some(items) = params.items {
                query.append_pair("items", &items.to_string());
            }
        }

        let resp = self
            .http
            .get(url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        handle_response(resp).await
    }

    /// Subscribe a URL to one or more events.
    ///
    /// `target_url` must be an https URL resolving to a publicly routable
    /// address. The result carries `signing_secret`, which the service returns
    /// only here.
    ///
    /// # Errors
    /// - [`PauboxError::Validation`] — `events` is empty
    /// - [`PauboxError::Http`] — 422 for a bad target or unknown event, 403
    ///   for an event this key is not scoped for
    pub async fn create_webhook_endpoint(
        &self,
        target_url: &str,
        events: &[&str],
    ) -> Result<CreatedWebhookEndpoint, PauboxError> {
        if target_url.is_empty() {
            return Err(PauboxError::Validation("target_url is required".into()));
        }
        if events.is_empty() {
            return Err(PauboxError::Validation(
                "at least one event is required".into(),
            ));
        }

        let body = CreateWebhookEndpointRequest {
            target_url: target_url.to_string(),
            events: events.iter().map(|e| (*e).to_string()).collect(),
        };

        let resp = self
            .http
            .post(self.endpoints_url()?)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await?;
        let envelope: DataEnvelope<CreatedWebhookEndpoint> = handle_response(resp).await?;
        Ok(envelope.data)
    }

    /// Retrieve a single endpoint by its UUID.
    ///
    /// The response does not include the signing secret. A malformed UUID and
    /// another tenant's id both return 404 with the same message, so the two
    /// are not distinguishable.
    pub async fn get_webhook_endpoint(&self, id: &str) -> Result<WebhookEndpoint, PauboxError> {
        let resp = self
            .http
            .get(self.endpoint_url(id)?)
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        let envelope: DataEnvelope<WebhookEndpoint> = handle_response(resp).await?;
        Ok(envelope.data)
    }

    /// Update an existing endpoint. Only the fields set on `changes` are sent.
    pub async fn update_webhook_endpoint(
        &self,
        id: &str,
        changes: UpdateWebhookEndpoint,
    ) -> Result<WebhookEndpoint, PauboxError> {
        let resp = self
            .http
            .patch(self.endpoint_url(id)?)
            .header("Authorization", self.auth_header())
            .json(&changes)
            .send()
            .await?;
        let envelope: DataEnvelope<WebhookEndpoint> = handle_response(resp).await?;
        Ok(envelope.data)
    }

    /// Delete an endpoint, stopping every event on it.
    ///
    /// The service answers 204 with no body, so there is nothing to return.
    pub async fn delete_webhook_endpoint(&self, id: &str) -> Result<(), PauboxError> {
        let resp = self
            .http
            .delete(self.endpoint_url(id)?)
            .header("Authorization", self.auth_header())
            .send()
            .await?;
        handle_empty_response(resp).await
    }
}

/// Canonical UUID shape (8-4-4-4-12 hex), checked without pulling in a
/// `uuid` dependency for one comparison.
fn uuid_shaped(value: &str) -> bool {
    let groups = [8, 4, 4, 4, 12];
    let parts: Vec<&str> = value.split('-').collect();
    parts.len() == groups.len()
        && parts
            .iter()
            .zip(groups)
            .all(|(part, len)| part.len() == len && part.chars().all(|c| c.is_ascii_hexdigit()))
}

/// Non-2xx responses carry `{"message": "..."}`; that text is promoted onto
/// the error so a caller sees `target_url: must be an https URL` rather than
/// an opaque status.
fn error_for(status: u16, body: String) -> PauboxError {
    let message = serde_json::from_str::<serde_json::Value>(&body)
        .ok()
        .and_then(|v| v.get("message")?.as_str().map(str::to_string))
        .unwrap_or(body);

    if status == 401 {
        PauboxError::Auth(message)
    } else {
        PauboxError::Http {
            status,
            body: message,
        }
    }
}

async fn handle_response<T>(resp: reqwest::Response) -> Result<T, PauboxError>
where
    T: serde::de::DeserializeOwned,
{
    let status = resp.status();
    if status.is_success() {
        let text = resp.text().await?;
        Ok(serde_json::from_str::<T>(&text)?)
    } else {
        let status_u16 = status.as_u16();
        let body = resp.text().await.unwrap_or_default();
        Err(error_for(status_u16, body))
    }
}

async fn handle_empty_response(resp: reqwest::Response) -> Result<(), PauboxError> {
    let status = resp.status();
    if status.is_success() {
        Ok(())
    } else {
        let status_u16 = status.as_u16();
        let body = resp.text().await.unwrap_or_default();
        Err(error_for(status_u16, body))
    }
}

#[cfg(test)]
mod tests {
    use super::uuid_shaped;

    #[test]
    fn accepts_a_canonical_uuid() {
        assert!(uuid_shaped("2ec66c21-bf48-48eb-8d28-f80b2d6b77c7"));
    }

    #[test]
    fn rejects_traversal_and_malformed_ids() {
        for bad in [
            "../endpoints",
            "",
            "not-a-uuid",
            "2ec66c21bf4848eb8d28f80b2d6b77c7",
            "2ec66c21-bf48-48eb-8d28-f80b2d6b77c7-extra",
            "zzzzzzzz-bf48-48eb-8d28-f80b2d6b77c7",
        ] {
            assert!(!uuid_shaped(bad), "{bad:?} should be rejected");
        }
    }
}
