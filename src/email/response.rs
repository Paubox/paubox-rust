//! Response types for the Paubox Email API.

use serde::Deserialize;

// ---------------------------------------------------------------------------
// Send message response
// ---------------------------------------------------------------------------

/// Response from [`crate::PauboxClient::send_message`].
#[derive(Debug, Clone, Deserialize)]
pub struct SendResponse {
    /// Tracking ID for this message.  Pass to
    /// [`crate::PauboxClient::get_email_disposition`] to check delivery status.
    #[serde(rename = "sourceTrackingId")]
    pub source_tracking_id: String,

    /// Human-readable confirmation message from the API.
    pub message: String,
}

// ---------------------------------------------------------------------------
// Disposition / status response
// ---------------------------------------------------------------------------

/// Response from [`crate::PauboxClient::get_email_disposition`].
#[derive(Debug, Clone, Deserialize)]
pub struct DispositionResponse {
    /// The tracking ID that was queried.
    #[serde(rename = "sourceTrackingId")]
    pub source_tracking_id: String,

    /// Per-recipient delivery records.
    #[serde(default)]
    pub message_deliveries: Vec<MessageDelivery>,

    /// API-level errors, if any.
    #[serde(default)]
    pub errors: Vec<ApiError>,
}

/// Delivery record for a single recipient.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageDelivery {
    /// Recipient email address.
    pub recipient: String,

    /// Delivery status string (e.g. `"delivered"`, `"bounced"`).
    #[serde(rename = "deliveryStatus")]
    pub delivery_status: String,

    /// ISO 8601 delivery timestamp, or `None` if not yet delivered.
    #[serde(rename = "deliveryTime", deserialize_with = "empty_string_as_none")]
    pub delivery_time: Option<String>,

    /// Open status: `"opened"` or `"unopened"`.
    #[serde(rename = "openedStatus")]
    pub opened_status: String,

    /// ISO 8601 open timestamp, or `None` if not yet opened.
    #[serde(rename = "openedTime", deserialize_with = "empty_string_as_none")]
    pub opened_time: Option<String>,
}

/// An API-level error object returned in error arrays.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiError {
    /// Numeric error code.
    pub code: Option<serde_json::Value>,
    /// Short status label.
    pub status: Option<String>,
    /// Human-readable title.
    pub title: Option<String>,
    /// Additional detail.
    pub details: Option<String>,
}

// ---------------------------------------------------------------------------
// Wire deserialization helpers
// ---------------------------------------------------------------------------

/// The API returns empty strings instead of `null` for absent timestamps.
/// This deserialiser converts `""` → `None`.
fn empty_string_as_none<'de, D>(d: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(d)?;
    Ok(s.filter(|v| !v.is_empty()))
}

// ---------------------------------------------------------------------------
// Wire wrapper types used only during deserialization
// ---------------------------------------------------------------------------

/// Top-level shape of the `GET /message_receipt` response.
#[derive(Debug, Deserialize)]
pub(crate) struct DispositionWire {
    #[serde(rename = "sourceTrackingId")]
    pub source_tracking_id: String,
    pub data: Option<DispositionData>,
    #[serde(default)]
    pub errors: Vec<ApiError>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DispositionData {
    pub message: Option<DispositionMessage>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DispositionMessage {
    #[serde(default, rename = "message_deliveries")]
    pub message_deliveries: Vec<MessageDeliveryWire>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct MessageDeliveryWire {
    pub recipient: String,
    pub status: DeliveryStatusWire,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DeliveryStatusWire {
    #[serde(rename = "deliveryStatus", default)]
    pub delivery_status: String,
    #[serde(rename = "deliveryTime", deserialize_with = "empty_string_as_none")]
    pub delivery_time: Option<String>,
    #[serde(rename = "openedStatus", default)]
    pub opened_status: String,
    #[serde(rename = "openedTime", deserialize_with = "empty_string_as_none")]
    pub opened_time: Option<String>,
}

impl DispositionWire {
    /// Flatten wire format into the public [`DispositionResponse`].
    pub(crate) fn into_response(self) -> DispositionResponse {
        let deliveries = self
            .data
            .and_then(|d| d.message)
            .map(|m| {
                m.message_deliveries
                    .into_iter()
                    .map(|d| MessageDelivery {
                        recipient: d.recipient,
                        delivery_status: d.status.delivery_status,
                        delivery_time: d.status.delivery_time,
                        opened_status: d.status.opened_status,
                        opened_time: d.status.opened_time,
                    })
                    .collect()
            })
            .unwrap_or_default();

        DispositionResponse {
            source_tracking_id: self.source_tracking_id,
            message_deliveries: deliveries,
            errors: self.errors,
        }
    }
}
