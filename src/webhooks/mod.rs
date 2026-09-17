pub mod types;

pub use types::{UpdateWebhookEndpoint, WebhookEndpoint};

use serde::Deserialize;

use crate::client::PauboxClient;
use crate::error::PauboxError;

use types::CreateWebhookEndpointRequest;

#[derive(Deserialize)]
struct DataWrapper<T> {
    data: T,
}

impl PauboxClient {
    /// List all webhook endpoints.
    pub async fn list_webhook_endpoints(&self) -> Result<Vec<WebhookEndpoint>, PauboxError> {
        let url = self.base_url.join("webhook_endpoints")?;

        let resp = self
            .http
            .get(url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/json")
            .send()
            .await?;

        let wrapper = handle_response::<DataWrapper<Vec<WebhookEndpoint>>>(resp).await?;
        Ok(wrapper.data)
    }

    /// Create a new webhook endpoint.
    pub async fn create_webhook_endpoint(
        &self,
        target_url: &str,
        events: &[&str],
        signing_key: Option<&str>,
        active: Option<bool>,
    ) -> Result<WebhookEndpoint, PauboxError> {
        let url = self.base_url.join("webhook_endpoints")?;
        let body = CreateWebhookEndpointRequest {
            target_url: target_url.to_owned(),
            events: events.iter().map(|s| (*s).to_owned()).collect(),
            signing_key: signing_key.map(String::from),
            active,
        };

        let resp = self
            .http
            .post(url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let wrapper = handle_response::<DataWrapper<WebhookEndpoint>>(resp).await?;
        Ok(wrapper.data)
    }

    /// Get a single webhook endpoint by ID.
    pub async fn get_webhook_endpoint(&self, id: i64) -> Result<WebhookEndpoint, PauboxError> {
        let url = self.base_url.join(&format!("webhook_endpoints/{id}"))?;

        let resp = self
            .http
            .get(url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/json")
            .send()
            .await?;

        let wrapper = handle_response::<DataWrapper<WebhookEndpoint>>(resp).await?;
        Ok(wrapper.data)
    }

    /// Update an existing webhook endpoint.
    pub async fn update_webhook_endpoint(
        &self,
        id: i64,
        changes: UpdateWebhookEndpoint,
    ) -> Result<WebhookEndpoint, PauboxError> {
        let url = self.base_url.join(&format!("webhook_endpoints/{id}"))?;

        let resp = self
            .http
            .patch(url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&changes)
            .send()
            .await?;

        let wrapper = handle_response::<DataWrapper<WebhookEndpoint>>(resp).await?;
        Ok(wrapper.data)
    }

    /// Delete a webhook endpoint by ID.
    pub async fn delete_webhook_endpoint(&self, id: i64) -> Result<(), PauboxError> {
        let url = self.base_url.join(&format!("webhook_endpoints/{id}"))?;

        let resp = self
            .http
            .delete(url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        handle_empty_response(resp).await
    }
}

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
        if status_u16 == 401 {
            Err(PauboxError::Auth(body))
        } else {
            Err(PauboxError::Http {
                status: status_u16,
                body,
            })
        }
    }
}

async fn handle_empty_response(resp: reqwest::Response) -> Result<(), PauboxError> {
    let status = resp.status();
    if status.is_success() {
        Ok(())
    } else {
        let status_u16 = status.as_u16();
        let body = resp.text().await.unwrap_or_default();
        if status_u16 == 401 {
            Err(PauboxError::Auth(body))
        } else {
            Err(PauboxError::Http {
                status: status_u16,
                body,
            })
        }
    }
}
