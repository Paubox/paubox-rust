use paubox::{PauboxClient, PauboxError};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

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
                {"email_id": "ea001", "subject": "Hello"},
                {"email_id": "ea002", "subject": "World"}
            ],
            "has_more": false
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let list = client.list_received_emails(None, None, None).await.unwrap();

    assert_eq!(list.object, "list");
    assert_eq!(list.data.len(), 2);
    assert_eq!(list.data[0].email_id, "ea001");
    assert_eq!(list.data[0].subject.as_deref(), Some("Hello"));
    assert!(!list.has_more);
}

#[tokio::test]
async fn list_received_emails_with_params() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/receiving"))
        .and(query_param("limit", "10"))
        .and(query_param("after", "cursor-abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": "list",
            "data": [],
            "has_more": true
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let list = client
        .list_received_emails(Some(10), Some("cursor-abc"), None)
        .await
        .unwrap();

    assert!(list.data.is_empty());
    assert!(list.has_more);
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
        .and(path("/v1/receiving/ea001"))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "email_id": "ea001",
                "subject": "Test Email",
                "from": [{"address": "sender@example.com", "name": null}],
                "to": [{"address": "recipient@example.com", "name": null}],
                "received_at": "2026-01-15T10:30:00Z",
                "body_text": "Hello world"
            }
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let email = client.get_received_email("ea001").await.unwrap();

    assert_eq!(email.email_id, "ea001");
    assert_eq!(email.subject.as_deref(), Some("Test Email"));
    assert_eq!(
        email
            .from
            .as_ref()
            .and_then(|v| v.first())
            .and_then(|a| a.address.as_deref()),
        Some("sender@example.com")
    );
    assert_eq!(email.body_text.as_deref(), Some("Hello world"));
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
        .and(path("/v1/receiving/ea001"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not valid"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client.get_received_email("ea001").await.unwrap_err();

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
        .and(path("/v1/receiving/ea001/attachments/blob123"))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(raw_bytes.clone())
                .insert_header("Content-Type", "image/png"),
        )
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let bytes = client
        .get_received_email_attachment("ea001", "blob123")
        .await
        .unwrap();

    assert_eq!(bytes, raw_bytes);
}

#[tokio::test]
async fn get_received_email_attachment_404() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/receiving/ea001/attachments/missing"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client
        .get_received_email_attachment("ea001", "missing")
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
        .and(path("/v1/receiving/ea001/attachments/blob123"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client
        .get_received_email_attachment("ea001", "blob123")
        .await
        .unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}
