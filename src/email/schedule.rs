use serde::{Deserialize, Serialize};

use crate::client::PauboxClient;
use crate::error::PauboxError;

use super::message::Message;
use super::handle_response;

#[derive(Debug, Serialize)]
struct ScheduleWire {
    data: ScheduleWireData,
}

#[derive(Debug, Serialize)]
struct ScheduleWireData {
    message: serde_json::Value,
    scheduled_at: String,
}

/// Response from [`PauboxClient::schedule_message`].
#[derive(Debug, Clone, Deserialize)]
pub struct ScheduleResponse {
    #[serde(rename = "sourceTrackingId")]
    pub source_tracking_id: String,
    #[serde(rename = "scheduledAt")]
    pub scheduled_at: String,
    pub state: String,
    #[serde(rename = "data")]
    pub message: String,
}

/// Status of a scheduled message from [`PauboxClient::get_scheduled_message`].
#[derive(Debug, Clone, Deserialize)]
pub struct ScheduledMessageStatus {
    #[serde(rename = "sourceTrackingId")]
    pub source_tracking_id: String,
    #[serde(rename = "scheduledAt")]
    pub scheduled_at: String,
    pub state: String,
    #[serde(rename = "messageId")]
    pub message_id: i64,
    #[serde(rename = "sentAt")]
    pub sent_at: Option<String>,
    #[serde(rename = "cancelledAt")]
    pub cancelled_at: Option<String>,
    #[serde(rename = "errorMessage")]
    pub error_message: Option<String>,
}

/// Response from [`PauboxClient::reschedule_message`].
#[derive(Debug, Clone, Deserialize)]
pub struct RescheduleResponse {
    #[serde(rename = "sourceTrackingId")]
    pub source_tracking_id: String,
    #[serde(rename = "scheduledAt")]
    pub scheduled_at: String,
    pub data: String,
}

/// Response from [`PauboxClient::cancel_scheduled_message`].
#[derive(Debug, Clone, Deserialize)]
pub struct CancelScheduledResponse {
    #[serde(rename = "sourceTrackingId")]
    pub source_tracking_id: String,
    pub state: String,
    pub data: String,
}

#[derive(Debug, Serialize)]
struct RescheduleWire {
    scheduled_at: String,
}

impl PauboxClient {
    /// Schedule a message for future delivery.
    ///
    /// `scheduled_at` must be an ISO 8601 UTC datetime string (e.g.
    /// `"2025-12-25T15:00:00Z"`). The time must be in the future and within
    /// 30 days.
    pub async fn schedule_message(
        &self,
        message: &Message,
        scheduled_at: &str,
    ) -> Result<ScheduleResponse, PauboxError> {
        let url = self.base_url.join("schedule")?;
        let wire_msg = message.to_wire();
        let msg_data = wire_msg
            .get("data")
            .and_then(|d| d.get("message"))
            .cloned()
            .unwrap_or(serde_json::Value::Object(Default::default()));

        let body = ScheduleWire {
            data: ScheduleWireData {
                message: msg_data,
                scheduled_at: scheduled_at.to_string(),
            },
        };

        let resp = self
            .http
            .post(url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        handle_response::<ScheduleResponse>(resp).await
    }

    /// Retrieve the status of a scheduled message.
    pub async fn get_scheduled_message(
        &self,
        source_tracking_id: &str,
    ) -> Result<ScheduledMessageStatus, PauboxError> {
        let url = self.base_url.join(&format!("schedule/{source_tracking_id}"))?;

        let resp = self
            .http
            .get(url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/json")
            .send()
            .await?;

        handle_response::<ScheduledMessageStatus>(resp).await
    }

    /// Reschedule a pending message to a new time.
    pub async fn reschedule_message(
        &self,
        source_tracking_id: &str,
        scheduled_at: &str,
    ) -> Result<RescheduleResponse, PauboxError> {
        let url = self.base_url.join(&format!("schedule/{source_tracking_id}"))?;
        let body = RescheduleWire {
            scheduled_at: scheduled_at.to_string(),
        };

        let resp = self
            .http
            .patch(url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        handle_response::<RescheduleResponse>(resp).await
    }

    /// Cancel a pending scheduled message.
    pub async fn cancel_scheduled_message(
        &self,
        source_tracking_id: &str,
    ) -> Result<CancelScheduledResponse, PauboxError> {
        let url = self.base_url.join(&format!("schedule/{source_tracking_id}/cancel"))?;

        let resp = self
            .http
            .post(url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .send()
            .await?;

        handle_response::<CancelScheduledResponse>(resp).await
    }
}
