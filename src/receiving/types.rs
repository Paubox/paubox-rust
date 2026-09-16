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
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub quota_bytes: Option<i64>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReceivedEmailList {
    #[serde(default)]
    pub object: String,
    #[serde(default)]
    pub data: Vec<ReceivedEmail>,
    #[serde(default)]
    pub has_more: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailAddress {
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReceivedEmail {
    pub email_id: String,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub from: Option<Vec<EmailAddress>>,
    #[serde(default)]
    pub to: Option<Vec<EmailAddress>>,
    #[serde(default)]
    pub received_at: Option<String>,
    #[serde(default)]
    pub body_text: Option<String>,
    #[serde(default)]
    pub body_html: Option<String>,
    #[serde(default)]
    pub has_attachment: Option<bool>,
    #[serde(default)]
    pub size: Option<i64>,
    #[serde(default)]
    pub spam: Option<bool>,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub attachments: Vec<AttachmentMeta>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AttachmentMeta {
    pub blob_id: String,
    #[serde(default)]
    pub file_name: Option<String>,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub size: Option<i64>,
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
