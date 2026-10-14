use serde::{Deserialize, Serialize};

/// A webhook endpoint registered for event notifications.
///
/// `id` is a UUID. Timestamps are RFC 3339 with microsecond precision and an
/// explicit `+00:00` offset, kept as strings so no precision is lost on the
/// way through.
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookEndpoint {
    pub id: String,
    pub target_url: String,
    /// `"active"` or `"disabled"`.
    pub status: String,
    #[serde(default)]
    pub events: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// The result of [`crate::webhooks::WebhooksClient::create_webhook_endpoint`].
///
/// `signing_secret` is returned once, at creation, and never again — not by
/// `get_webhook_endpoint` and not by `list_webhook_endpoints`. Store it when
/// you receive it; recovering from a lost secret means replacing the endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct CreatedWebhookEndpoint {
    #[serde(flatten)]
    pub endpoint: WebhookEndpoint,
    pub signing_secret: String,
}

/// Request body for creating a webhook endpoint.
///
/// Events are not validated client-side on purpose: the catalog is owned by
/// the service and grows without a crate release. An event this key is not
/// scoped for is refused with 403, an unrecognised one with 422.
#[derive(Debug, Serialize)]
pub(crate) struct CreateWebhookEndpointRequest {
    pub target_url: String,
    pub events: Vec<String>,
}

/// Fields to update on an existing webhook endpoint.
///
/// Every field is optional and only those set are sent, so an update that
/// moves `target_url` leaves `events` and `status` untouched.
#[derive(Debug, Default, Serialize)]
pub struct UpdateWebhookEndpoint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_url: Option<String>,
    /// `"active"` or `"disabled"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<String>>,
}

/// Pagination metadata.
///
/// `count` is the total number of endpoints matching the request, not the
/// length of the returned page.
#[derive(Debug, Clone, Deserialize)]
pub struct PageInfo {
    pub count: i64,
    pub items: i64,
}

/// The result of [`crate::webhooks::WebhooksClient::list_webhook_endpoints`].
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookEndpointList {
    pub data: Vec<WebhookEndpoint>,
    pub page_info: PageInfo,
}

/// Optional query parameters for listing endpoints.
#[derive(Debug, Default, Clone)]
pub struct ListWebhookEndpointsParams {
    pub page: Option<u32>,
    pub items: Option<u32>,
}

/// Single resources arrive wrapped in a `data` envelope.
#[derive(Debug, Deserialize)]
pub(crate) struct DataEnvelope<T> {
    pub data: T,
}
