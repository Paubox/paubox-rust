use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ReceivingDomain {
    pub id: i64,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub dns_records: Vec<DnsRecord>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DnsRecord {
    #[serde(default, rename = "type")]
    pub record_type: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub priority: Option<i64>,
    #[serde(default)]
    pub verified: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Mailbox {
    pub id: i64,
    #[serde(default, alias = "email")]
    pub email_address: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub quota_bytes: Option<i64>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// A page of received emails from [`crate::PauboxClient::list_received_emails`].
#[derive(Debug, Clone, Deserialize)]
pub struct ReceivedEmailList {
    /// Always `"list"`.
    #[serde(default)]
    pub object: String,
    /// Emails on this page. List items carry summary fields only; bodies,
    /// attachments and headers are populated by
    /// [`crate::PauboxClient::get_received_email`].
    #[serde(default)]
    pub data: Vec<ReceivedEmail>,
    /// Whether more emails exist beyond this page.
    #[serde(default)]
    pub has_more: bool,
}

/// A sender or recipient address.
#[derive(Debug, Clone, Deserialize)]
pub struct EmailAddress {
    /// Email address.
    #[serde(default)]
    pub address: Option<String>,
    /// Display name.
    #[serde(default)]
    pub name: Option<String>,
}

/// SPF, DKIM and DMARC results for a received email.
#[derive(Debug, Clone, Deserialize)]
pub struct EmailAuthentication {
    /// SPF result.
    #[serde(default)]
    pub spf: String,
    /// DKIM result.
    #[serde(default)]
    pub dkim: String,
    /// DMARC result.
    #[serde(default)]
    pub dmarc: String,
}

/// A raw header from a received email.
#[derive(Debug, Clone, Deserialize)]
pub struct EmailHeader {
    /// Header name.
    #[serde(default)]
    pub name: String,
    /// Header value.
    #[serde(default)]
    pub value: String,
}

/// A received (inbound) email.
///
/// Returned both as a list item by
/// [`crate::PauboxClient::list_received_emails`] and as the full detail by
/// [`crate::PauboxClient::get_received_email`]. Fields that only the detail
/// endpoint returns are `None` (or empty) on list items, and `has_attachment`
/// is only set on list items.
#[derive(Debug, Clone, Deserialize)]
#[serde(from = "ReceivedEmailWire")]
pub struct ReceivedEmail {
    /// Paubox email UUID.
    pub email_id: String,
    /// Subject line.
    pub subject: Option<String>,
    /// Senders.
    pub from: Option<Vec<EmailAddress>>,
    /// Recipients.
    pub to: Option<Vec<EmailAddress>>,
    /// CC recipients (detail only).
    pub cc: Option<Vec<EmailAddress>>,
    /// Value of the `Date` header (detail only).
    pub date: Option<String>,
    /// When Paubox received the email.
    pub received_at: Option<String>,
    /// `Message-ID` header values (detail only).
    pub message_id: Option<Vec<String>>,
    /// `In-Reply-To` header values (detail only).
    pub in_reply_to: Option<Vec<String>>,
    /// `References` header values (detail only).
    pub references: Option<Vec<String>>,
    /// Plain-text body (detail only).
    pub text_body: Option<String>,
    /// HTML body (detail only).
    pub html_body: Option<String>,
    /// Deprecated alias of [`ReceivedEmail::text_body`].
    #[deprecated(note = "use `text_body`")]
    pub body_text: Option<String>,
    /// Deprecated alias of [`ReceivedEmail::html_body`].
    #[deprecated(note = "use `html_body`")]
    pub body_html: Option<String>,
    /// Whether the email has attachments (list items only).
    pub has_attachment: Option<bool>,
    /// Size in bytes.
    pub size: Option<i64>,
    /// Whether the email was classified as spam.
    pub spam: Option<bool>,
    /// Spam score (detail only).
    pub spam_score: Option<f64>,
    /// Receiving domain the email was delivered to.
    pub domain: Option<String>,
    /// SPF, DKIM and DMARC results (detail only).
    pub authentication: Option<EmailAuthentication>,
    /// Raw headers (detail only).
    pub headers: Option<Vec<EmailHeader>>,
    /// Attachment metadata (detail only).
    pub attachments: Vec<AttachmentMeta>,
}

#[derive(Deserialize)]
struct ReceivedEmailWire {
    email_id: String,
    #[serde(default)]
    subject: Option<String>,
    #[serde(default)]
    from: Option<Vec<EmailAddress>>,
    #[serde(default)]
    to: Option<Vec<EmailAddress>>,
    #[serde(default)]
    cc: Option<Vec<EmailAddress>>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    received_at: Option<String>,
    #[serde(default)]
    message_id: Option<Vec<String>>,
    #[serde(default)]
    in_reply_to: Option<Vec<String>>,
    #[serde(default)]
    references: Option<Vec<String>>,
    #[serde(default)]
    text_body: Option<String>,
    #[serde(default)]
    html_body: Option<String>,
    #[serde(default)]
    has_attachment: Option<bool>,
    #[serde(default)]
    size: Option<i64>,
    #[serde(default)]
    spam: Option<bool>,
    #[serde(default)]
    spam_score: Option<f64>,
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    authentication: Option<EmailAuthentication>,
    #[serde(default)]
    headers: Option<Vec<EmailHeader>>,
    #[serde(default)]
    attachments: Vec<AttachmentMeta>,
}

impl From<ReceivedEmailWire> for ReceivedEmail {
    #[allow(deprecated)]
    fn from(w: ReceivedEmailWire) -> Self {
        Self {
            body_text: w.text_body.clone(),
            body_html: w.html_body.clone(),
            email_id: w.email_id,
            subject: w.subject,
            from: w.from,
            to: w.to,
            cc: w.cc,
            date: w.date,
            received_at: w.received_at,
            message_id: w.message_id,
            in_reply_to: w.in_reply_to,
            references: w.references,
            text_body: w.text_body,
            html_body: w.html_body,
            has_attachment: w.has_attachment,
            size: w.size,
            spam: w.spam,
            spam_score: w.spam_score,
            domain: w.domain,
            authentication: w.authentication,
            headers: w.headers,
            attachments: w.attachments,
        }
    }
}

/// Metadata for an attachment on a received email.
#[derive(Debug, Clone, Deserialize)]
#[serde(from = "AttachmentMetaWire")]
pub struct AttachmentMeta {
    /// Paubox attachment UUID. Pass to
    /// [`crate::PauboxClient::get_received_email_attachment`] to download it.
    pub id: String,
    /// Original filename.
    pub filename: Option<String>,
    /// MIME type.
    pub content_type: Option<String>,
    /// Size in bytes.
    pub size: Option<i64>,
    /// `Content-ID` for inline attachments.
    pub content_id: Option<String>,
    /// API URL that serves the attachment bytes.
    pub download_url: String,
    /// Deprecated: the API no longer returns mail-server blob ids. This now
    /// holds the same value as [`AttachmentMeta::id`], so existing code that
    /// passes it to [`crate::PauboxClient::get_received_email_attachment`]
    /// keeps working.
    #[deprecated(note = "use `id`")]
    pub blob_id: String,
    /// Deprecated alias of [`AttachmentMeta::filename`].
    #[deprecated(note = "use `filename`")]
    pub file_name: Option<String>,
}

#[derive(Deserialize)]
struct AttachmentMetaWire {
    #[serde(default)]
    id: String,
    #[serde(default)]
    filename: Option<String>,
    #[serde(default)]
    content_type: Option<String>,
    #[serde(default)]
    size: Option<i64>,
    #[serde(default)]
    content_id: Option<String>,
    #[serde(default)]
    download_url: String,
}

impl From<AttachmentMetaWire> for AttachmentMeta {
    #[allow(deprecated)]
    fn from(w: AttachmentMetaWire) -> Self {
        Self {
            blob_id: w.id.clone(),
            file_name: w.filename.clone(),
            id: w.id,
            filename: w.filename,
            content_type: w.content_type,
            size: w.size,
            content_id: w.content_id,
            download_url: w.download_url,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct CreateDomainRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct CreateMailboxRequest {
    pub name: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_bytes: Option<i64>,
}
