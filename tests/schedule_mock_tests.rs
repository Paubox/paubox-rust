use paubox::{email::Message, PauboxClient, PauboxError};
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

fn simple_message() -> Message {
    Message::builder()
        .from("sender@example.com")
        .to(["recipient@example.com"])
        .subject("Test subject")
        .text_content("Hello!")
        .build()
        .unwrap()
}

#[tokio::test]
async fn schedule_message_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/schedule"))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sourceTrackingId": "tid-sched",
            "scheduledAt": "2025-12-25T15:00:00Z",
            "state": "pending",
            "data": "Service OK"
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let resp = client
        .schedule_message(&simple_message(), "2025-12-25T15:00:00Z")
        .await
        .unwrap();

    assert_eq!(resp.source_tracking_id, "tid-sched");
    assert_eq!(resp.scheduled_at, "2025-12-25T15:00:00Z");
    assert_eq!(resp.state, "pending");
}

#[tokio::test]
async fn schedule_message_sends_scheduled_at_in_body() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/schedule"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sourceTrackingId": "x",
            "scheduledAt": "2025-12-25T15:00:00Z",
            "state": "pending",
            "data": "OK"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let _ = client
        .schedule_message(&simple_message(), "2025-12-25T15:00:00Z")
        .await
        .unwrap();

    let received = server.received_requests().await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(body["data"]["scheduled_at"], "2025-12-25T15:00:00Z");
    assert!(body["data"]["message"].is_object());
}

#[tokio::test]
async fn schedule_message_401_returns_auth_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/schedule"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client
        .schedule_message(&simple_message(), "2025-12-25T15:00:00Z")
        .await
        .unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}

#[tokio::test]
async fn get_scheduled_message_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/schedule/tid-123"))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sourceTrackingId": "tid-123",
            "scheduledAt": "2025-12-25T15:00:00Z",
            "state": "pending",
            "messageId": 42
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let resp = client.get_scheduled_message("tid-123").await.unwrap();

    assert_eq!(resp.source_tracking_id, "tid-123");
    assert_eq!(resp.state, "pending");
    assert_eq!(resp.message_id, 42);
}

#[tokio::test]
async fn get_scheduled_message_404() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/schedule/nonexistent"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client
        .get_scheduled_message("nonexistent")
        .await
        .unwrap_err();

    match err {
        PauboxError::Http { status, .. } => assert_eq!(status, 404),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn reschedule_message_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/v1/schedule/tid-456"))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sourceTrackingId": "tid-456",
            "scheduledAt": "2025-12-26T10:00:00Z",
            "data": "Rescheduled"
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let resp = client
        .reschedule_message("tid-456", "2025-12-26T10:00:00Z")
        .await
        .unwrap();

    assert_eq!(resp.source_tracking_id, "tid-456");
    assert_eq!(resp.scheduled_at, "2025-12-26T10:00:00Z");
    assert_eq!(resp.data, "Rescheduled");
}

#[tokio::test]
async fn reschedule_message_sends_correct_body() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/v1/schedule/tid-456"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sourceTrackingId": "tid-456",
            "scheduledAt": "2025-12-26T10:00:00Z",
            "data": "Rescheduled"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let _ = client
        .reschedule_message("tid-456", "2025-12-26T10:00:00Z")
        .await
        .unwrap();

    let received = server.received_requests().await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&received[0].body).unwrap();
    assert_eq!(body["scheduled_at"], "2025-12-26T10:00:00Z");
    assert!(body.get("data").is_none());
}

#[tokio::test]
async fn cancel_scheduled_message_happy_path() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/schedule/tid-789/cancel"))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sourceTrackingId": "tid-789",
            "state": "cancelled",
            "data": "Cancelled"
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let resp = client.cancel_scheduled_message("tid-789").await.unwrap();

    assert_eq!(resp.source_tracking_id, "tid-789");
    assert_eq!(resp.state, "cancelled");
    assert_eq!(resp.data, "Cancelled");
}

#[tokio::test]
async fn cancel_scheduled_message_404() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/schedule/nonexistent/cancel"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client
        .cancel_scheduled_message("nonexistent")
        .await
        .unwrap_err();

    match err {
        PauboxError::Http { status, .. } => assert_eq!(status, 404),
        other => panic!("unexpected error: {other:?}"),
    }
}
