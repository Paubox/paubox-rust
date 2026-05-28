//! Mocked unit tests for the Paubox Email API client.
//!
//! Uses `wiremock` to start a local HTTP server and verifies that the client
//! sends correct requests and deserialises responses properly.

use paubox::{email::Message, PauboxClient, PauboxError};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn make_client(server: &MockServer) -> PauboxClient {
    let base_url = url::Url::parse(&format!("{}/", server.uri())).unwrap();
    PauboxClient::builder()
        .api_key("test-key")
        .api_user("test-user")
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

// ---------------------------------------------------------------------------
// send_message — happy path
// ---------------------------------------------------------------------------

#[tokio::test]
async fn send_message_returns_tracking_id() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/messages"))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sourceTrackingId": "abc-123",
            "message": "Service OK"
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let resp = client.send_message(&simple_message()).await.unwrap();

    assert_eq!(resp.source_tracking_id, "abc-123");
    assert_eq!(resp.message, "Service OK");
}

// ---------------------------------------------------------------------------
// send_message — 401 auth error
// ---------------------------------------------------------------------------

#[tokio::test]
async fn send_message_401_returns_auth_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client.send_message(&simple_message()).await.unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}

// ---------------------------------------------------------------------------
// send_message — 500 server error
// ---------------------------------------------------------------------------

#[tokio::test]
async fn send_message_500_returns_http_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client.send_message(&simple_message()).await.unwrap_err();

    match err {
        PauboxError::Http { status, .. } => assert_eq!(status, 500),
        other => panic!("unexpected error: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// send_message — malformed JSON response
// ---------------------------------------------------------------------------

#[tokio::test]
async fn send_message_malformed_json_returns_deserialize_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client.send_message(&simple_message()).await.unwrap_err();

    assert!(matches!(err, PauboxError::Deserialize(_)));
}

// ---------------------------------------------------------------------------
// get_email_disposition — happy path with delivery records
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_email_disposition_parses_response() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/message_receipt"))
        .and(query_param("sourceTrackingId", "track-xyz"))
        .and(header("Authorization", "Token token=test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sourceTrackingId": "track-xyz",
            "data": {
                "message": {
                    "id": "msg-1",
                    "message_deliveries": [
                        {
                            "recipient": "recipient@example.com",
                            "status": {
                                "deliveryStatus": "delivered",
                                "deliveryTime": "2024-01-15T10:30:00Z",
                                "openedStatus": "opened",
                                "openedTime": "2024-01-15T11:00:00Z"
                            }
                        }
                    ]
                }
            },
            "errors": []
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let resp = client.get_email_disposition("track-xyz").await.unwrap();

    assert_eq!(resp.source_tracking_id, "track-xyz");
    assert_eq!(resp.message_deliveries.len(), 1);

    let d = &resp.message_deliveries[0];
    assert_eq!(d.recipient, "recipient@example.com");
    assert_eq!(d.delivery_status, "delivered");
    assert_eq!(d.delivery_time.as_deref(), Some("2024-01-15T10:30:00Z"));
    assert_eq!(d.opened_status, "opened");
    assert_eq!(d.opened_time.as_deref(), Some("2024-01-15T11:00:00Z"));
}

// ---------------------------------------------------------------------------
// get_email_disposition — empty timestamps become None
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_email_disposition_empty_timestamps_become_none() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/message_receipt"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sourceTrackingId": "track-abc",
            "data": {
                "message": {
                    "id": "msg-2",
                    "message_deliveries": [
                        {
                            "recipient": "pending@example.com",
                            "status": {
                                "deliveryStatus": "pending",
                                "deliveryTime": "",
                                "openedStatus": "unopened",
                                "openedTime": ""
                            }
                        }
                    ]
                }
            },
            "errors": []
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let resp = client.get_email_disposition("track-abc").await.unwrap();

    let d = &resp.message_deliveries[0];
    assert!(d.delivery_time.is_none());
    assert!(d.opened_time.is_none());
}

// ---------------------------------------------------------------------------
// Message builder — validation
// ---------------------------------------------------------------------------

#[test]
fn message_builder_missing_from_fails() {
    let err = Message::builder()
        .to(["r@example.com"])
        .subject("hi")
        .build()
        .unwrap_err();
    assert!(matches!(err, PauboxError::Validation(_)));
}

#[test]
fn message_builder_empty_to_fails() {
    let err = Message::builder()
        .from("s@example.com")
        .subject("hi")
        .build()
        .unwrap_err();
    assert!(matches!(err, PauboxError::Validation(_)));
}

#[test]
fn message_builder_missing_subject_fails() {
    let err = Message::builder()
        .from("s@example.com")
        .to(["r@example.com"])
        .build()
        .unwrap_err();
    assert!(matches!(err, PauboxError::Validation(_)));
}

// ---------------------------------------------------------------------------
// PauboxClient::from_env
// ---------------------------------------------------------------------------

#[test]
fn from_env_missing_key_returns_error() {
    // Ensure neither var is set in this test process
    std::env::remove_var("PAUBOX_API_KEY");
    std::env::remove_var("PAUBOX_API_USER");
    let err = PauboxClient::from_env().unwrap_err();
    assert!(matches!(err, PauboxError::EnvVar(_)));
}

// ---------------------------------------------------------------------------
// Integration test stubs (require live credentials)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires live credentials: PAUBOX_API_KEY, PAUBOX_API_USER"]
async fn live_send_and_check_disposition() {
    let client = PauboxClient::from_env().unwrap();

    let msg = Message::builder()
        .from(std::env::var("PAUBOX_FROM").unwrap_or_else(|_| "test@example.com".into()))
        .to([std::env::var("PAUBOX_TO").unwrap_or_else(|_| "test@example.com".into())])
        .subject("Paubox Rust SDK integration test")
        .text_content("Integration test message — please ignore.")
        .build()
        .unwrap();

    let send_resp = client.send_message(&msg).await.unwrap();
    println!("Tracking ID: {}", send_resp.source_tracking_id);

    let disposition = client
        .get_email_disposition(&send_resp.source_tracking_id)
        .await
        .unwrap();
    println!("{:#?}", disposition);
}
