use paubox::webhooks::UpdateWebhookEndpoint;
use paubox::{PauboxClient, PauboxError};
use wiremock::matchers::{header, method, path};
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
// list_webhook_endpoints
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_webhook_endpoints_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/webhook_endpoints"))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": 1,
                    "target_url": "https://example.com/hook1",
                    "events": ["inbound_mail_received"],
                    "active": true
                },
                {
                    "id": 2,
                    "target_url": "https://example.com/hook2",
                    "events": ["api_mail_log_delivered", "api_mail_log_opened"],
                    "active": false
                }
            ]
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let endpoints = client.list_webhook_endpoints().await.unwrap();

    assert_eq!(endpoints.len(), 2);
    assert_eq!(endpoints[0].id, 1);
    assert_eq!(
        endpoints[0].target_url.as_deref(),
        Some("https://example.com/hook1")
    );
    assert_eq!(endpoints[0].events, vec!["inbound_mail_received"]);
    assert_eq!(endpoints[0].active, Some(true));
    assert_eq!(endpoints[1].id, 2);
    assert_eq!(endpoints[1].active, Some(false));
}

#[tokio::test]
async fn list_webhook_endpoints_401() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/webhook_endpoints"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client.list_webhook_endpoints().await.unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}

#[tokio::test]
async fn list_webhook_endpoints_malformed_json() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/webhook_endpoints"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client.list_webhook_endpoints().await.unwrap_err();

    assert!(matches!(err, PauboxError::Deserialize(_)));
}

// ---------------------------------------------------------------------------
// create_webhook_endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_webhook_endpoint_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/webhook_endpoints"))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "message": "Webhook created!",
            "data": {
                "id": 42,
                "target_url": "https://example.com/webhooks/paubox",
                "events": ["inbound_mail_received", "api_mail_log_delivered"],
                "active": true,
                "signing_key": "whsec_abc123"
            }
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let endpoint = client
        .create_webhook_endpoint(
            "https://example.com/webhooks/paubox",
            &["inbound_mail_received", "api_mail_log_delivered"],
            Some("whsec_abc123"),
            Some(true),
        )
        .await
        .unwrap();

    assert_eq!(endpoint.id, 42);
    assert_eq!(
        endpoint.target_url.as_deref(),
        Some("https://example.com/webhooks/paubox")
    );
    assert_eq!(endpoint.events.len(), 2);
    assert_eq!(endpoint.signing_key.as_deref(), Some("whsec_abc123"));
    assert_eq!(endpoint.active, Some(true));
}

#[tokio::test]
async fn create_webhook_endpoint_sends_correct_body() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/webhook_endpoints"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "message": "Webhook created!",
            "data": {"id": 10, "target_url": "https://hook.test", "events": ["api_mail_log_delivered"]}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let _ = client
        .create_webhook_endpoint("https://hook.test", &["api_mail_log_delivered"], None, None)
        .await
        .unwrap();

    let received = server.received_requests().await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(body["target_url"], "https://hook.test");
    assert_eq!(
        body["events"],
        serde_json::json!(["api_mail_log_delivered"])
    );
    assert!(body.get("signing_key").is_none());
    assert!(body.get("active").is_none());
}

#[tokio::test]
async fn create_webhook_endpoint_401() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/webhook_endpoints"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client
        .create_webhook_endpoint("https://hook.test", &["inbound_mail_received"], None, None)
        .await
        .unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}

// ---------------------------------------------------------------------------
// get_webhook_endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_webhook_endpoint_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/webhook_endpoints/42"))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": 42,
                "target_url": "https://example.com/webhooks/paubox",
                "events": ["inbound_mail_received"],
                "active": true,
                "signing_key": "whsec_abc123",
                "created_at": "2026-09-16T10:00:00.000Z",
                "updated_at": "2026-09-16T10:00:00.000Z"
            }
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let endpoint = client.get_webhook_endpoint(42).await.unwrap();

    assert_eq!(endpoint.id, 42);
    assert_eq!(
        endpoint.target_url.as_deref(),
        Some("https://example.com/webhooks/paubox")
    );
    assert_eq!(endpoint.events, vec!["inbound_mail_received"]);
    assert_eq!(endpoint.active, Some(true));
    assert_eq!(
        endpoint.created_at.as_deref(),
        Some("2026-09-16T10:00:00.000Z")
    );
}

#[tokio::test]
async fn get_webhook_endpoint_404() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/webhook_endpoints/999"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client.get_webhook_endpoint(999).await.unwrap_err();

    match err {
        PauboxError::Http { status, .. } => assert_eq!(status, 404),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn get_webhook_endpoint_malformed_json() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/webhook_endpoints/42"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not valid"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client.get_webhook_endpoint(42).await.unwrap_err();

    assert!(matches!(err, PauboxError::Deserialize(_)));
}

// ---------------------------------------------------------------------------
// update_webhook_endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn update_webhook_endpoint_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/v1/webhook_endpoints/42"))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Webhook updated!",
            "data": {
                "id": 42,
                "target_url": "https://new-url.example.com/hook",
                "events": ["api_mail_log_delivered"],
                "active": false
            }
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let changes = UpdateWebhookEndpoint {
        target_url: Some("https://new-url.example.com/hook".into()),
        active: Some(false),
        ..Default::default()
    };
    let endpoint = client.update_webhook_endpoint(42, changes).await.unwrap();

    assert_eq!(endpoint.id, 42);
    assert_eq!(
        endpoint.target_url.as_deref(),
        Some("https://new-url.example.com/hook")
    );
    assert_eq!(endpoint.active, Some(false));
}

#[tokio::test]
async fn update_webhook_endpoint_sends_only_present_fields() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/v1/webhook_endpoints/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Webhook updated!",
            "data": {"id": 42, "active": false}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let changes = UpdateWebhookEndpoint {
        active: Some(false),
        ..Default::default()
    };
    let _ = client.update_webhook_endpoint(42, changes).await.unwrap();

    let received = server.received_requests().await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(body["active"], false);
    assert!(body.get("target_url").is_none());
    assert!(body.get("events").is_none());
    assert!(body.get("api_key").is_none());
}

#[tokio::test]
async fn update_webhook_endpoint_404() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/v1/webhook_endpoints/999"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client
        .update_webhook_endpoint(999, UpdateWebhookEndpoint::default())
        .await
        .unwrap_err();

    match err {
        PauboxError::Http { status, .. } => assert_eq!(status, 404),
        other => panic!("unexpected error: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// delete_webhook_endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn delete_webhook_endpoint_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/webhook_endpoints/42"))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Webhook deleted!",
            "data": {"id": 42}
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    client.delete_webhook_endpoint(42).await.unwrap();
}

#[tokio::test]
async fn delete_webhook_endpoint_404() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/webhook_endpoints/999"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client.delete_webhook_endpoint(999).await.unwrap_err();

    match err {
        PauboxError::Http { status, .. } => assert_eq!(status, 404),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn delete_webhook_endpoint_401() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/webhook_endpoints/42"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client.delete_webhook_endpoint(42).await.unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}
