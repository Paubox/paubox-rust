//! Paubox Email API — send HIPAA-compliant email and track delivery.

pub mod message;
pub mod response;

pub use message::{Attachment, Message, MessageBuilder};
pub use response::{ApiError, DispositionResponse, MessageDelivery, SendResponse};

use crate::client::PauboxClient;
use crate::error::PauboxError;
use response::DispositionWire;

impl PauboxClient {
    /// Send a message through the Paubox Email API.
    ///
    /// Returns a [`SendResponse`] containing a `source_tracking_id` that can
    /// be passed to [`PauboxClient::get_email_disposition`].
    ///
    /// # Errors
    /// - [`PauboxError::Auth`] — invalid or missing API credentials (HTTP 401)
    /// - [`PauboxError::Http`] — any other non-2xx response
    /// - [`PauboxError::Request`] — network or TLS failure
    /// - [`PauboxError::Deserialize`] — unexpected response body shape
    ///
    /// # Example
    /// ```no_run
    /// use paubox::{PauboxClient, email::Message};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = PauboxClient::new("api-key", "api-user");
    /// let msg = Message::builder()
    ///     .from("you@example.com")
    ///     .to(["patient@example.com"])
    ///     .subject("Your results are ready")
    ///     .text_content("Please log in to view your results.")
    ///     .build()?;
    ///
    /// let resp = client.send_message(&msg).await?;
    /// println!("Tracking ID: {}", resp.source_tracking_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_message(&self, message: &Message) -> Result<SendResponse, PauboxError> {
        let url = self.base_url.join("messages")?;
        let body = message.to_wire();

        let resp = self
            .http
            .post(url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        handle_response::<SendResponse>(resp).await
    }

    /// Retrieve the delivery disposition of a previously sent message.
    ///
    /// # Errors
    /// Same variants as [`PauboxClient::send_message`].
    ///
    /// # Example
    /// ```no_run
    /// use paubox::PauboxClient;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = PauboxClient::new("api-key", "api-user");
    /// let disposition = client.get_email_disposition("tracking-id-here").await?;
    /// for delivery in &disposition.message_deliveries {
    ///     println!("{}: {}", delivery.recipient, delivery.delivery_status);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_email_disposition(
        &self,
        source_tracking_id: &str,
    ) -> Result<DispositionResponse, PauboxError> {
        let mut url = self.base_url.join("message_receipt")?;
        url.query_pairs_mut()
            .append_pair("sourceTrackingId", source_tracking_id);

        let resp = self
            .http
            .get(url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/json")
            .send()
            .await?;

        let wire = handle_response::<DispositionWire>(resp).await?;
        Ok(wire.into_response())
    }

    /// Check the health / status of the Paubox Email API.
    ///
    /// Returns `Ok(())` when the API is reachable and the credentials are
    /// valid.
    ///
    /// # Errors
    /// Same variants as [`PauboxClient::send_message`].
    pub async fn api_status(&self) -> Result<(), PauboxError> {
        let resp = self
            .http
            .get(self.base_url.clone())
            .header("Authorization", self.auth_header())
            .header("Accept", "application/json")
            .send()
            .await?;

        // We only care about success vs. failure; discard the body.
        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            if status == 401 {
                Err(PauboxError::Auth(body))
            } else {
                Err(PauboxError::Http { status, body })
            }
        }
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
        if status_u16 == 401 {
            Err(PauboxError::Auth(body))
        } else {
            Err(PauboxError::Http { status: status_u16, body })
        }
    }
}
