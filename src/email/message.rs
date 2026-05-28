//! Request types for the Paubox Email API.

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde::Serialize;

use crate::error::PauboxError;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// An email message to be sent through the Paubox Email API.
///
/// Use [`Message::builder`] to construct instances with validated required
/// fields.
///
/// # Example
/// ```
/// use paubox::email::Message;
///
/// let msg = Message::builder()
///     .from("sender@example.com")
///     .to(["recipient@example.com"])
///     .subject("Hello from Paubox")
///     .text_content("This is a secure message.")
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Message {
    /// Sender email address.
    pub from: String,
    /// Primary recipient email addresses.
    pub to: Vec<String>,
    /// Email subject line.
    pub subject: String,
    /// Reply-to address (optional).
    pub reply_to: Option<String>,
    /// CC recipients (optional).
    pub cc: Option<Vec<String>>,
    /// BCC recipients (optional).
    pub bcc: Option<Vec<String>>,
    /// Plain-text body (optional but recommended).
    pub text_content: Option<String>,
    /// HTML body (optional).  Will be base64-encoded when serialised to the
    /// wire format.
    pub html_content: Option<String>,
    /// File attachments (optional).
    pub attachments: Vec<Attachment>,
    /// Allow delivery over non-TLS connections.  Only set to `true` for
    /// non-PHI messages.
    pub allow_non_tls: Option<bool>,
    /// Force a secure portal notification instead of direct delivery.
    pub force_secure_notification: Option<bool>,
}

impl Message {
    /// Return a new [`MessageBuilder`].
    pub fn builder() -> MessageBuilder {
        MessageBuilder::default()
    }

    /// Serialise this message to the JSON wire format expected by the API.
    pub(crate) fn to_wire(&self) -> serde_json::Value {
        let mut content: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        if let Some(ref txt) = self.text_content {
            content.insert("text/plain".into(), txt.clone().into());
        }
        if let Some(ref html) = self.html_content {
            let encoded = if is_base64(html) {
                html.clone()
            } else {
                B64.encode(html.as_bytes())
            };
            content.insert("text/html".into(), encoded.into());
        }

        let mut headers: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        headers.insert("subject".into(), self.subject.clone().into());
        headers.insert("from".into(), self.from.clone().into());
        if let Some(ref rt) = self.reply_to {
            headers.insert("reply-to".into(), rt.clone().into());
        }

        let mut msg: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        msg.insert(
            "recipients".into(),
            serde_json::Value::Array(self.to.iter().map(|r| r.clone().into()).collect()),
        );
        msg.insert("headers".into(), headers.into());
        msg.insert("content".into(), content.into());

        if let Some(ref cc) = self.cc {
            msg.insert(
                "cc".into(),
                serde_json::Value::Array(cc.iter().map(|e| e.clone().into()).collect()),
            );
        }
        if let Some(ref bcc) = self.bcc {
            msg.insert(
                "bcc".into(),
                serde_json::Value::Array(bcc.iter().map(|e| e.clone().into()).collect()),
            );
        }
        if !self.attachments.is_empty() {
            let attachments: Vec<serde_json::Value> =
                self.attachments.iter().map(|a| a.to_wire()).collect();
            msg.insert("attachments".into(), attachments.into());
        }
        if let Some(v) = self.allow_non_tls {
            msg.insert("allowNonTLS".into(), v.into());
        }
        if let Some(v) = self.force_secure_notification {
            msg.insert("forceSecureNotification".into(), v.into());
        }

        serde_json::json!({ "data": { "message": msg } })
    }
}

/// A file attachment for an email message.
#[derive(Debug, Clone, Serialize)]
pub struct Attachment {
    /// Filename shown to the recipient (e.g. `"report.pdf"`).
    #[serde(rename = "fileName")]
    pub file_name: String,
    /// MIME type (e.g. `"application/pdf"`).
    #[serde(rename = "contentType")]
    pub content_type: String,
    /// Base64-encoded file contents.
    pub content: String,
}

impl Attachment {
    /// Create an attachment from raw bytes, base64-encoding them automatically.
    ///
    /// # Example
    /// ```
    /// use paubox::email::Attachment;
    ///
    /// let data = b"PDF content here";
    /// let att = Attachment::from_bytes("report.pdf", "application/pdf", data);
    /// ```
    pub fn from_bytes(
        file_name: impl Into<String>,
        content_type: impl Into<String>,
        data: &[u8],
    ) -> Self {
        Self {
            file_name: file_name.into(),
            content_type: content_type.into(),
            content: B64.encode(data),
        }
    }

    /// Create an attachment from an already-base64-encoded string.
    pub fn from_base64(
        file_name: impl Into<String>,
        content_type: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            file_name: file_name.into(),
            content_type: content_type.into(),
            content: content.into(),
        }
    }

    fn to_wire(&self) -> serde_json::Value {
        serde_json::json!({
            "fileName": self.file_name,
            "contentType": self.content_type,
            "content": self.content,
        })
    }
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// Builder for [`Message`].
///
/// Required fields: `from`, at least one `to` address, `subject`.
/// At least one of `text_content` or `html_content` should be provided.
#[derive(Debug, Default)]
pub struct MessageBuilder {
    from: Option<String>,
    to: Vec<String>,
    subject: Option<String>,
    reply_to: Option<String>,
    cc: Option<Vec<String>>,
    bcc: Option<Vec<String>>,
    text_content: Option<String>,
    html_content: Option<String>,
    attachments: Vec<Attachment>,
    allow_non_tls: Option<bool>,
    force_secure_notification: Option<bool>,
}

impl MessageBuilder {
    /// Set the sender address.
    pub fn from(mut self, address: impl Into<String>) -> Self {
        self.from = Some(address.into());
        self
    }

    /// Add one or more `To` recipients.
    pub fn to(mut self, addresses: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.to.extend(addresses.into_iter().map(Into::into));
        self
    }

    /// Set the subject line.
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Set the reply-to address.
    pub fn reply_to(mut self, address: impl Into<String>) -> Self {
        self.reply_to = Some(address.into());
        self
    }

    /// Set CC recipients.
    pub fn cc(mut self, addresses: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.cc = Some(addresses.into_iter().map(Into::into).collect());
        self
    }

    /// Set BCC recipients.
    pub fn bcc(mut self, addresses: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.bcc = Some(addresses.into_iter().map(Into::into).collect());
        self
    }

    /// Set the plain-text body.
    pub fn text_content(mut self, text: impl Into<String>) -> Self {
        self.text_content = Some(text.into());
        self
    }

    /// Set the HTML body.
    pub fn html_content(mut self, html: impl Into<String>) -> Self {
        self.html_content = Some(html.into());
        self
    }

    /// Add a single attachment.
    pub fn attachment(mut self, att: Attachment) -> Self {
        self.attachments.push(att);
        self
    }

    /// Add multiple attachments.
    pub fn attachments(mut self, atts: impl IntoIterator<Item = Attachment>) -> Self {
        self.attachments.extend(atts);
        self
    }

    /// Allow delivery over non-TLS connections (non-PHI only).
    pub fn allow_non_tls(mut self, value: bool) -> Self {
        self.allow_non_tls = Some(value);
        self
    }

    /// Force secure portal notification delivery.
    pub fn force_secure_notification(mut self, value: bool) -> Self {
        self.force_secure_notification = Some(value);
        self
    }

    /// Consume the builder, validating required fields.
    ///
    /// # Errors
    /// Returns [`PauboxError::Validation`] if `from`, `to`, or `subject` are
    /// missing.
    pub fn build(self) -> Result<Message, PauboxError> {
        let from = self
            .from
            .ok_or_else(|| PauboxError::Validation("`from` address is required".into()))?;
        if self.to.is_empty() {
            return Err(PauboxError::Validation(
                "at least one `to` address is required".into(),
            ));
        }
        let subject = self
            .subject
            .ok_or_else(|| PauboxError::Validation("`subject` is required".into()))?;

        Ok(Message {
            from,
            to: self.to,
            subject,
            reply_to: self.reply_to,
            cc: self.cc,
            bcc: self.bcc,
            text_content: self.text_content,
            html_content: self.html_content,
            attachments: self.attachments,
            allow_non_tls: self.allow_non_tls,
            force_secure_notification: self.force_secure_notification,
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn is_base64(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_message() -> Message {
        Message::builder()
            .from("sender@example.com")
            .to(["recipient@example.com"])
            .subject("Test subject")
            .text_content("Hello!")
            .build()
            .unwrap()
    }

    #[test]
    fn to_wire_contains_required_fields() {
        let wire = simple_message().to_wire();
        let data = &wire["data"]["message"];
        assert_eq!(data["recipients"][0], "recipient@example.com");
        assert_eq!(data["headers"]["subject"], "Test subject");
        assert_eq!(data["headers"]["from"], "sender@example.com");
        assert_eq!(data["content"]["text/plain"], "Hello!");
    }

    #[test]
    fn to_wire_optional_fields_absent_when_not_set() {
        let wire = simple_message().to_wire();
        let data = &wire["data"]["message"];
        assert!(data["bcc"].is_null());
        assert!(data["cc"].is_null());
        assert!(data["allowNonTLS"].is_null());
        assert!(data["forceSecureNotification"].is_null());
    }

    #[test]
    fn attachment_from_bytes_encodes_correctly() {
        let att = Attachment::from_bytes("file.txt", "text/plain", b"hello");
        assert_eq!(att.content, "aGVsbG8=");
        assert_eq!(att.file_name, "file.txt");
    }
}
