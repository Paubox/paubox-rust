//! Mocked unit tests for the Paubox Forms API client.

use paubox::{
    forms::{FormAttachment, FormSubmission, FormsClient},
    PauboxError,
};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn make_client(server: &MockServer) -> FormsClient {
    let base_url = url::Url::parse(&server.uri()).unwrap();
    FormsClient::with_base_url(base_url)
}

// ---------------------------------------------------------------------------
// get_form — happy path
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_form_parses_response() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/public/form_data/test-uuid"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "test-uuid",
            "title": "Patient Intake",
            "description": "Initial intake form",
            "form_html": "<form>...</form>",
            "form_json": {"fields": ["first_name", "last_name"]},
            "form_css": "body { font-family: sans-serif; }",
            "active": true,
            "signable": false,
            "submission_count": 42,
            "customer_id": 1,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-06-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let form = client.get_form("test-uuid").await.unwrap();

    assert_eq!(form.id, "test-uuid");
    assert_eq!(form.title, "Patient Intake");
    assert_eq!(form.description.as_deref(), Some("Initial intake form"));
    assert!(form.active);
    assert!(!form.signable);
    assert_eq!(form.submission_count, 42);
}

// ---------------------------------------------------------------------------
// get_form — 404
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_form_404_returns_http_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/public/form_data/missing"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client.get_form("missing").await.unwrap_err();

    match err {
        PauboxError::Http { status, .. } => assert_eq!(status, 404),
        other => panic!("unexpected: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// get_form — malformed JSON
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_form_malformed_json_returns_deserialize_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/public/form_data/bad"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{invalid json"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let err = client.get_form("bad").await.unwrap_err();

    assert!(matches!(err, PauboxError::Deserialize(_)));
}

// ---------------------------------------------------------------------------
// submit_form — happy path
// ---------------------------------------------------------------------------

#[tokio::test]
async fn submit_form_201_returns_ok() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/forms/test-uuid/submissions"))
        .respond_with(ResponseTemplate::new(201))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let submission = FormSubmission::builder()
        .form_data(json!({"first_name": "Jane"}))
        .build()
        .unwrap();

    client.submit_form("test-uuid", &submission).await.unwrap();
}

// ---------------------------------------------------------------------------
// submit_form — 400 bad request
// ---------------------------------------------------------------------------

#[tokio::test]
async fn submit_form_400_returns_http_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/forms/test-uuid/submissions"))
        .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
        .mount(&server)
        .await;

    let client = make_client(&server).await;
    let submission = FormSubmission::builder()
        .form_data(json!({"x": "y"}))
        .build()
        .unwrap();

    let err = client
        .submit_form("test-uuid", &submission)
        .await
        .unwrap_err();

    match err {
        PauboxError::Http { status, .. } => assert_eq!(status, 400),
        other => panic!("unexpected: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// FormSubmission builder — validation
// ---------------------------------------------------------------------------

#[test]
fn form_submission_builder_missing_data_fails() {
    let err = FormSubmission::builder().build().unwrap_err();
    assert!(matches!(err, PauboxError::Validation(_)));
}

#[test]
fn form_submission_builder_null_data_fails() {
    let err = FormSubmission::builder()
        .form_data(json!(null))
        .build()
        .unwrap_err();
    assert!(matches!(err, PauboxError::Validation(_)));
}

// ---------------------------------------------------------------------------
// FormAttachment helpers
// ---------------------------------------------------------------------------

#[test]
fn form_attachment_from_bytes_encodes_correctly() {
    let att = FormAttachment::from_bytes("test.txt", b"hello");
    // base64 of "hello" is "aGVsbG8="
    assert_eq!(att.content, "aGVsbG8=");
    assert_eq!(att.name, "test.txt");
}

#[test]
fn form_attachment_from_base64_stores_as_is() {
    let att = FormAttachment::from_base64("file.pdf", "aGVsbG8=");
    assert_eq!(att.content, "aGVsbG8=");
}

// ---------------------------------------------------------------------------
// Integration test stubs (no credentials needed — Forms API is public)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires a live Paubox form UUID in env var PAUBOX_FORM_ID"]
async fn live_get_form() {
    let form_id = std::env::var("PAUBOX_FORM_ID").expect("PAUBOX_FORM_ID not set");
    let client = FormsClient::new();
    let form = client.get_form(&form_id).await.unwrap();
    println!("{:#?}", form);
}
