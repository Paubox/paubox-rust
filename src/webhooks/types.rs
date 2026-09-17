use serde::{Deserialize, Serialize};

/// A webhook endpoint registered for event notifications.
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookEndpoint {
    pub id: i64,
    #[serde(default)]
    pub target_url: Option<String>,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default)]
    pub signing_key: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Request body for creating a webhook endpoint.
#[derive(Debug, Serialize)]
pub(crate) struct CreateWebhookEndpointRequest {
    pub target_url: String,
    pub events: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signing_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

/// Fields to update on an existing webhook endpoint.
#[derive(Debug, Default, Serialize)]
pub struct UpdateWebhookEndpoint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}
