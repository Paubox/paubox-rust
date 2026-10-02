use paubox::{PauboxClient, PauboxError};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const EMAIL_ID: &str = "0f8e4c2a-6b1d-4e7f-9a3c-5d2b8e1f4a60";
const ATTACHMENT_ID: &str = "b2c4d6e8-1a3f-4b5d-8c7e-9f0a1b2c3d4e";

fn detail_json() -> serde_json::Value {
    serde_json::json!({
        "email_id": EMAIL_ID,
        "from": [{"name": "Sender", "address": "sender@example.com"}],
        "to": [{"name": null, "address": "support@test.inbound.paubox.email"}],
        "cc": [{"name": null, "address": "cc@example.com"}],
        "subject": "Test Email",
        "date": "Thu, 01 Oct 2026 10:29:58 +0000",
        "received_at": "2026-10-01T10:30:00Z",
        "message_id": ["<abc@example.com>"],
        "in_reply_to": null,
        "references": [],
        "spam": false,
        "spam_score": 1.5,
        "text_body": "Hello world",
        "html_body": "<p>Hello world</p>",
        "attachments": [
            {
                "id": ATTACHMENT_ID,
                "filename": "report.pdf",
                "content_type": "application/pdf",
                "size": 1024,
                "content_id": null,
                "download_url": format!(
                    "https://api.paubox.com/v1/email/receiving/{EMAIL_ID}/attachments/{ATTACHMENT_ID}"
                )
            },
            {
                "id": "c3d5e7f9-2b4a-4c6e-9d8f-0a1b2c3d4e5f",
                "filename": null,
                "content_type": null,
                "size": null,
                "content_id": "logo@example.com",
                "download_url": "https://example.test/inline"
            }
        ],
        "size": 4096,
        "authentication": {"spf": "pass", "dkim": "pass", "dmarc": "fail"},
        "domain": "test.inbound.paubox.email",
        "headers": [{"name": "Subject", "value": "Test Email"}]
    })
}

async fn make_client(server: &MockServer) -> PauboxClient {
    let base_url = url::Url::parse(&format!("{}/v1/", server.uri())).unwrap();
    PauboxClient::builder()
        .api_key("test-key")
        .base_url(base_url)
        .build()
        .unwrap()
}

// ---------------------------------------------------------------------------
// list_receiving_domains
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_receiving_domains_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/receiving/domains"))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {"id": 1, "domain": "test.inbound.paubox.email", "status": "active"},
                {"id": 2, "domain": "other.inbound.paubox.email"}
            ]
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let domains = client.list_receiving_domains().await.unwrap();

    assert_eq!(domains.len(), 2);
    assert_eq!(domains[0].id, 1);
    assert_eq!(
        domains[0].domain.as_deref(),
        Some("test.inbound.paubox.email")
    );
    assert_eq!(domains[0].status.as_deref(), Some("active"));
    assert_eq!(domains[1].id, 2);
}

#[tokio::test]
async fn list_receiving_domains_401_returns_auth_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/receiving/domains"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client.list_receiving_domains().await.unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}

#[tokio::test]
async fn list_receiving_domains_malformed_json() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/receiving/domains"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client.list_receiving_domains().await.unwrap_err();

    assert!(matches!(err, PauboxError::Deserialize(_)));
}

// ---------------------------------------------------------------------------
// create_receiving_domain
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_receiving_domain_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/receiving/domains"))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {"id": 1, "domain": "abc123.inbound.paubox.email"}
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let domain = client.create_receiving_domain(None).await.unwrap();

    assert_eq!(domain.id, 1);
    assert_eq!(
        domain.domain.as_deref(),
        Some("abc123.inbound.paubox.email")
    );
}

#[tokio::test]
async fn create_receiving_domain_with_slug_sends_body() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/receiving/domains"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {"id": 3, "domain": "myslug.inbound.paubox.email", "slug": "myslug"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let domain = client
        .create_receiving_domain(Some("myslug"))
        .await
        .unwrap();

    assert_eq!(domain.id, 3);
    assert_eq!(domain.slug.as_deref(), Some("myslug"));

    let received = server.received_requests().await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(body["slug"], "myslug");
}

#[tokio::test]
async fn create_receiving_domain_401() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/receiving/domains"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client.create_receiving_domain(None).await.unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}

// ---------------------------------------------------------------------------
// get_receiving_domain
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_receiving_domain_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/receiving/domains/1"))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": 1, "domain": "test.inbound.paubox.email", "status": "active"}
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let domain = client.get_receiving_domain(1).await.unwrap();

    assert_eq!(domain.id, 1);
    assert_eq!(domain.domain.as_deref(), Some("test.inbound.paubox.email"));
}

#[tokio::test]
async fn get_receiving_domain_404() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/receiving/domains/999"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client.get_receiving_domain(999).await.unwrap_err();

    match err {
        PauboxError::Http { status, .. } => assert_eq!(status, 404),
        other => panic!("unexpected error: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// delete_receiving_domain
// ---------------------------------------------------------------------------

#[tokio::test]
async fn delete_receiving_domain_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/receiving/domains/1"))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    client.delete_receiving_domain(1).await.unwrap();
}

#[tokio::test]
async fn delete_receiving_domain_404() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/receiving/domains/999"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client.delete_receiving_domain(999).await.unwrap_err();

    match err {
        PauboxError::Http { status, .. } => assert_eq!(status, 404),
        other => panic!("unexpected error: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// list_receiving_mailboxes
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_receiving_mailboxes_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/receiving/domains/1/mailboxes"))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {"id": 1, "email": "support@test.inbound.paubox.email", "name": "support"},
                {"id": 2, "email": "info@test.inbound.paubox.email", "name": "info"}
            ]
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let mailboxes = client.list_receiving_mailboxes(1).await.unwrap();

    assert_eq!(mailboxes.len(), 2);
    assert_eq!(mailboxes[0].id, 1);
    assert_eq!(
        mailboxes[0].email_address.as_deref(),
        Some("support@test.inbound.paubox.email")
    );
    assert_eq!(mailboxes[0].name.as_deref(), Some("support"));
}

#[tokio::test]
async fn list_receiving_mailboxes_401() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/receiving/domains/1/mailboxes"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client.list_receiving_mailboxes(1).await.unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}

// ---------------------------------------------------------------------------
// create_receiving_mailbox
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_receiving_mailbox_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/receiving/domains/1/mailboxes"))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {"id": 5, "email": "support@test.inbound.paubox.email", "name": "support"}
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let mailbox = client
        .create_receiving_mailbox(1, "support", "secret123", None)
        .await
        .unwrap();

    assert_eq!(mailbox.id, 5);
    assert_eq!(
        mailbox.email_address.as_deref(),
        Some("support@test.inbound.paubox.email")
    );
}

#[tokio::test]
async fn create_receiving_mailbox_sends_correct_body() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/receiving/domains/1/mailboxes"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {"id": 6, "name": "admin"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let _ = client
        .create_receiving_mailbox(1, "admin", "pass", Some(1_000_000))
        .await
        .unwrap();

    let received = server.received_requests().await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(body["name"], "admin");
    assert_eq!(body["password"], "pass");
    assert_eq!(body["quota_bytes"], 1_000_000);
}

#[tokio::test]
async fn create_receiving_mailbox_400() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/receiving/domains/1/mailboxes"))
        .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client
        .create_receiving_mailbox(1, "", "", None)
        .await
        .unwrap_err();

    match err {
        PauboxError::Http { status, .. } => assert_eq!(status, 400),
        other => panic!("unexpected error: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// get_receiving_mailbox
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_receiving_mailbox_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/receiving/domains/1/mailboxes/2"))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": 2, "email": "info@test.inbound.paubox.email", "name": "info"}
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let mailbox = client.get_receiving_mailbox(1, 2).await.unwrap();

    assert_eq!(mailbox.id, 2);
    assert_eq!(
        mailbox.email_address.as_deref(),
        Some("info@test.inbound.paubox.email")
    );
}

#[tokio::test]
async fn get_receiving_mailbox_404() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/receiving/domains/1/mailboxes/999"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client.get_receiving_mailbox(1, 999).await.unwrap_err();

    match err {
        PauboxError::Http { status, .. } => assert_eq!(status, 404),
        other => panic!("unexpected error: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// delete_receiving_mailbox
// ---------------------------------------------------------------------------

#[tokio::test]
async fn delete_receiving_mailbox_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/receiving/domains/1/mailboxes/2"))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    client.delete_receiving_mailbox(1, 2).await.unwrap();
}

#[tokio::test]
async fn delete_receiving_mailbox_404() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/receiving/domains/1/mailboxes/999"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client.delete_receiving_mailbox(1, 999).await.unwrap_err();

    match err {
        PauboxError::Http { status, .. } => assert_eq!(status, 404),
        other => panic!("unexpected error: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// list_received_emails
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_received_emails_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/receiving"))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": "list",
            "data": [
                {
                    "email_id": EMAIL_ID,
                    "from": [{"name": "Sender", "address": "sender@example.com"}],
                    "to": [{"name": null, "address": "support@test.inbound.paubox.email"}],
                    "subject": "Hello",
                    "received_at": "2026-10-01T10:30:00Z",
                    "has_attachment": true,
                    "spam": false,
                    "size": 2048,
                    "domain": "test.inbound.paubox.email"
                },
                {
                    "email_id": "7d3c1b9e-2f4a-4c8d-9e6b-0a1b2c3d4e5f",
                    "from": [],
                    "to": [],
                    "subject": null,
                    "received_at": null,
                    "has_attachment": null,
                    "spam": true,
                    "size": null,
                    "domain": "test.inbound.paubox.email"
                }
            ],
            "has_more": false
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let list = client.list_received_emails(None, None, None).await.unwrap();

    assert_eq!(list.object, "list");
    assert_eq!(list.data.len(), 2);
    assert!(!list.has_more);

    let first = &list.data[0];
    assert_eq!(first.email_id, EMAIL_ID);
    assert_eq!(first.subject.as_deref(), Some("Hello"));
    assert_eq!(
        first
            .from
            .as_ref()
            .and_then(|v| v.first())
            .and_then(|a| a.name.as_deref()),
        Some("Sender")
    );
    assert_eq!(first.received_at.as_deref(), Some("2026-10-01T10:30:00Z"));
    assert_eq!(first.has_attachment, Some(true));
    assert_eq!(first.spam, Some(false));
    assert_eq!(first.size, Some(2048));
    assert_eq!(first.domain.as_deref(), Some("test.inbound.paubox.email"));
    assert!(first.attachments.is_empty());
    assert!(first.text_body.is_none());

    let second = &list.data[1];
    assert!(second.subject.is_none());
    assert!(second.has_attachment.is_none());
    assert_eq!(second.spam, Some(true));
}

#[tokio::test]
async fn list_received_emails_with_params() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/receiving"))
        .and(query_param("limit", "10"))
        .and(query_param("after", EMAIL_ID))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": "list",
            "data": [],
            "has_more": true
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let list = client
        .list_received_emails(Some(10), Some(EMAIL_ID), None)
        .await
        .unwrap();

    assert!(list.data.is_empty());
    assert!(list.has_more);
}

#[tokio::test]
async fn list_received_emails_before_cursor() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/receiving"))
        .and(query_param("before", EMAIL_ID))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": "list",
            "data": [],
            "has_more": false
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let list = client
        .list_received_emails(None, None, Some(EMAIL_ID))
        .await
        .unwrap();

    assert!(list.data.is_empty());
}

#[tokio::test]
async fn list_received_emails_401() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/receiving"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client
        .list_received_emails(None, None, None)
        .await
        .unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}

#[tokio::test]
async fn list_received_emails_malformed_json() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/receiving"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{bad"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client
        .list_received_emails(None, None, None)
        .await
        .unwrap_err();

    assert!(matches!(err, PauboxError::Deserialize(_)));
}

// ---------------------------------------------------------------------------
// get_received_email
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_received_email_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/v1/receiving/{EMAIL_ID}")))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "data": detail_json() })),
        )
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let email = client.get_received_email(EMAIL_ID).await.unwrap();

    assert_eq!(email.email_id, EMAIL_ID);
    assert_eq!(email.subject.as_deref(), Some("Test Email"));
    assert_eq!(
        email
            .from
            .as_ref()
            .and_then(|v| v.first())
            .and_then(|a| a.address.as_deref()),
        Some("sender@example.com")
    );
    assert_eq!(
        email
            .cc
            .as_ref()
            .and_then(|v| v.first())
            .and_then(|a| a.address.as_deref()),
        Some("cc@example.com")
    );
    assert_eq!(
        email.date.as_deref(),
        Some("Thu, 01 Oct 2026 10:29:58 +0000")
    );
    assert_eq!(email.received_at.as_deref(), Some("2026-10-01T10:30:00Z"));
    assert_eq!(
        email.message_id.as_deref(),
        Some(&["<abc@example.com>".to_owned()][..])
    );
    assert!(email.in_reply_to.is_none());
    assert_eq!(email.references.as_ref().map(Vec::len), Some(0));
    assert_eq!(email.spam, Some(false));
    assert_eq!(email.spam_score, Some(1.5));
    assert_eq!(email.text_body.as_deref(), Some("Hello world"));
    assert_eq!(email.html_body.as_deref(), Some("<p>Hello world</p>"));
    assert_eq!(email.size, Some(4096));
    assert_eq!(email.domain.as_deref(), Some("test.inbound.paubox.email"));

    let auth = email.authentication.as_ref().unwrap();
    assert_eq!(auth.spf, "pass");
    assert_eq!(auth.dkim, "pass");
    assert_eq!(auth.dmarc, "fail");

    let headers = email.headers.as_ref().unwrap();
    assert_eq!(headers[0].name, "Subject");
    assert_eq!(headers[0].value, "Test Email");

    assert_eq!(email.attachments.len(), 2);
    let att = &email.attachments[0];
    assert_eq!(att.id, ATTACHMENT_ID);
    assert_eq!(att.filename.as_deref(), Some("report.pdf"));
    assert_eq!(att.content_type.as_deref(), Some("application/pdf"));
    assert_eq!(att.size, Some(1024));
    assert!(att.content_id.is_none());
    assert_eq!(
        att.download_url,
        format!("https://api.paubox.com/v1/email/receiving/{EMAIL_ID}/attachments/{ATTACHMENT_ID}")
    );

    let inline = &email.attachments[1];
    assert!(inline.filename.is_none());
    assert!(inline.content_type.is_none());
    assert!(inline.size.is_none());
    assert_eq!(inline.content_id.as_deref(), Some("logo@example.com"));
}

#[tokio::test]
#[allow(deprecated)]
async fn get_received_email_populates_deprecated_aliases() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/v1/receiving/{EMAIL_ID}")))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "data": detail_json() })),
        )
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let email = client.get_received_email(EMAIL_ID).await.unwrap();

    assert_eq!(email.body_text.as_deref(), Some("Hello world"));
    assert_eq!(email.body_html.as_deref(), Some("<p>Hello world</p>"));
    assert_eq!(email.attachments[0].blob_id, ATTACHMENT_ID);
    assert_eq!(
        email.attachments[0].file_name.as_deref(),
        Some("report.pdf")
    );
}

#[tokio::test]
async fn get_received_email_with_minimal_and_null_fields() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/v1/receiving/{EMAIL_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "email_id": EMAIL_ID,
                "from": [],
                "to": [],
                "cc": [],
                "subject": null,
                "date": null,
                "received_at": null,
                "message_id": null,
                "in_reply_to": null,
                "references": null,
                "spam": false,
                "spam_score": null,
                "text_body": null,
                "html_body": null,
                "attachments": [
                    {"id": ATTACHMENT_ID, "download_url": "https://example.test/a"}
                ],
                "size": null,
                "authentication": {"spf": "none", "dkim": "none", "dmarc": "none"},
                "domain": "test.inbound.paubox.email",
                "headers": null
            }
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let email = client.get_received_email(EMAIL_ID).await.unwrap();

    assert!(email.subject.is_none());
    assert!(email.message_id.is_none());
    assert!(email.spam_score.is_none());
    assert!(email.text_body.is_none());
    assert!(email.headers.is_none());
    assert_eq!(email.attachments.len(), 1);
    assert_eq!(email.attachments[0].id, ATTACHMENT_ID);
    assert!(email.attachments[0].filename.is_none());
}

#[tokio::test]
async fn get_received_email_404() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/receiving/nonexistent"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client.get_received_email("nonexistent").await.unwrap_err();

    match err {
        PauboxError::Http { status, .. } => assert_eq!(status, 404),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn get_received_email_malformed_json() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/v1/receiving/{EMAIL_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_string("not valid"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client.get_received_email(EMAIL_ID).await.unwrap_err();

    assert!(matches!(err, PauboxError::Deserialize(_)));
}

// ---------------------------------------------------------------------------
// get_received_email_attachment
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_received_email_attachment_happy_path() {
    let server = MockServer::start().await;

    let raw_bytes: Vec<u8> = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

    Mock::given(method("GET"))
        .and(path(format!(
            "/v1/receiving/{EMAIL_ID}/attachments/{ATTACHMENT_ID}"
        )))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(raw_bytes.clone())
                .insert_header("Content-Type", "image/png")
                .insert_header("Content-Disposition", "attachment; filename=\"logo.png\""),
        )
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let bytes = client
        .get_received_email_attachment(EMAIL_ID, ATTACHMENT_ID)
        .await
        .unwrap();

    assert_eq!(bytes, raw_bytes);
}

#[tokio::test]
async fn get_received_email_attachment_returns_json_looking_bytes_verbatim() {
    let server = MockServer::start().await;

    let raw_bytes = br#"{"data":"not parsed"}"#.to_vec();

    Mock::given(method("GET"))
        .and(path(format!(
            "/v1/receiving/{EMAIL_ID}/attachments/{ATTACHMENT_ID}"
        )))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(raw_bytes.clone())
                .insert_header("Content-Type", "application/json"),
        )
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let bytes = client
        .get_received_email_attachment(EMAIL_ID, ATTACHMENT_ID)
        .await
        .unwrap();

    assert_eq!(bytes, raw_bytes);
}

#[tokio::test]
#[allow(deprecated)]
async fn get_received_email_attachment_by_id_from_detail() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/v1/receiving/{EMAIL_ID}")))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "data": detail_json() })),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path(format!(
            "/v1/receiving/{EMAIL_ID}/attachments/{ATTACHMENT_ID}"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"%PDF-1.7".to_vec()))
        .expect(2)
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let email = client.get_received_email(EMAIL_ID).await.unwrap();
    let att = &email.attachments[0];

    let by_id = client
        .get_received_email_attachment(&email.email_id, &att.id)
        .await
        .unwrap();
    let by_legacy_field = client
        .get_received_email_attachment(&email.email_id, &att.blob_id)
        .await
        .unwrap();

    assert_eq!(by_id, b"%PDF-1.7");
    assert_eq!(by_legacy_field, b"%PDF-1.7");
}

#[tokio::test]
async fn get_received_email_attachment_404() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!(
            "/v1/receiving/{EMAIL_ID}/attachments/legacy-blob-id"
        )))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client
        .get_received_email_attachment(EMAIL_ID, "legacy-blob-id")
        .await
        .unwrap_err();

    match err {
        PauboxError::Http { status, .. } => assert_eq!(status, 404),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn get_received_email_attachment_401() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!(
            "/v1/receiving/{EMAIL_ID}/attachments/{ATTACHMENT_ID}"
        )))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client
        .get_received_email_attachment(EMAIL_ID, ATTACHMENT_ID)
        .await
        .unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}
