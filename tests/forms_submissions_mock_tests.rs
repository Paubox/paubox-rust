//! Mocked integration tests for the Paubox Forms submission endpoints:
//! `list_submissions`, `export_submissions_csv`, `export_submission_csv`,
//! and `export_submission_pdf`.

use paubox::{
    forms::{FormsClient, SubmissionListParams},
    PauboxError,
};
use serde_json::json;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const API_KEY: &str = "test-key-with-forms-scope";
const FORM_ID: &str = "550e8400-e29b-41d4-a716-446655440000";
const SUBMISSION_ID: &str = "9f3c2a10-1234-4bcd-9e00-abcdefabcdef";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// An authenticated client pointed at the mock server.  Includes the `/forms`
/// base segment (without a trailing slash) so these tests assert the client
/// preserves it when joining endpoint paths.
fn authed_client(server: &MockServer) -> FormsClient {
    let base_url = url::Url::parse(&format!("{}/forms", server.uri())).expect("valid URL");
    FormsClient::builder()
        .api_key(API_KEY)
        .base_url(base_url)
        .build()
        .expect("builder succeeds")
}

/// An unauthenticated client pointed at the mock server.
fn anon_client(server: &MockServer) -> FormsClient {
    let base_url = url::Url::parse(&format!("{}/forms", server.uri())).expect("valid URL");
    FormsClient::with_base_url(base_url)
}

/// A realistic submission object as serialized by the server (`form_data` is
/// a JSON-encoded STRING, not a nested object).
fn submission_json() -> serde_json::Value {
    json!({
        "id": SUBMISSION_ID,
        "form_id": FORM_ID,
        "form_data": "{\"first_name\":\"Jane\",\"last_name\":\"Doe\",\"consent\":true}",
        "storage_type": "database",
        "storage_url": null,
        "submitter_email": "jane@example.com",
        "recipients": "intake@clinic.example.com,records@clinic.example.com",
        "attachment": null,
        "attachment_name": "insurance-card.pdf",
        "attachment_url": "https://storage.example.com/insurance-card.pdf",
        "attachment_type": "application/pdf",
        "created_at": "2026-02-01T12:34:56Z"
    })
}

// ---------------------------------------------------------------------------
// list_submissions — happy path
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_submissions_parses_response() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/forms/api/forms/{FORM_ID}/submissions")))
        .and(header("Authorization", format!("Bearer {API_KEY}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [submission_json()],
            "total": 12,
            "page": 1,
            "items": 50
        })))
        .mount(&server)
        .await;

    let client = authed_client(&server);
    let page = client
        .list_submissions(FORM_ID, &SubmissionListParams::default())
        .await
        .unwrap();

    // Pagination fields
    assert_eq!(page.total, 12);
    assert_eq!(page.page, 1);
    assert_eq!(page.items, 50);
    assert_eq!(page.data.len(), 1);

    // Submission fields
    let sub = &page.data[0];
    assert_eq!(sub.id, SUBMISSION_ID);
    assert_eq!(sub.form_id, FORM_ID);
    assert_eq!(sub.storage_type, "database");
    assert_eq!(sub.storage_url, None);
    assert_eq!(sub.submitter_email.as_deref(), Some("jane@example.com"));
    assert_eq!(
        sub.recipients.as_deref(),
        Some("intake@clinic.example.com,records@clinic.example.com")
    );
    assert_eq!(sub.attachment, None);
    assert_eq!(sub.attachment_name.as_deref(), Some("insurance-card.pdf"));
    assert_eq!(
        sub.attachment_url.as_deref(),
        Some("https://storage.example.com/insurance-card.pdf")
    );
    assert_eq!(sub.attachment_type.as_deref(), Some("application/pdf"));
    assert_eq!(sub.created_at, "2026-02-01T12:34:56Z");

    // form_data stays a raw JSON-encoded string...
    assert_eq!(
        sub.form_data,
        "{\"first_name\":\"Jane\",\"last_name\":\"Doe\",\"consent\":true}"
    );
    // ...and the helper parses it into a Value.
    let parsed = sub.form_data_json().unwrap();
    assert_eq!(parsed["first_name"], "Jane");
    assert_eq!(parsed["last_name"], "Doe");
    assert_eq!(parsed["consent"], true);
}

// ---------------------------------------------------------------------------
// list_submissions — query parameters reach the wire
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_submissions_sends_query_params() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/forms/api/forms/{FORM_ID}/submissions")))
        .and(query_param("submission_id", SUBMISSION_ID))
        .and(query_param("order", "asc"))
        .and(query_param("order_by", "submitter_email"))
        .and(query_param("page", "3"))
        .and(query_param("items", "25"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [],
            "total": 0,
            "page": 3,
            "items": 25
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = authed_client(&server);
    let params = SubmissionListParams::default()
        .submission_id(SUBMISSION_ID)
        .order("asc")
        .order_by("submitter_email")
        .page(3)
        .items(25);

    let page = client.list_submissions(FORM_ID, &params).await.unwrap();
    assert!(page.data.is_empty());
    assert_eq!(page.page, 3);
}

// ---------------------------------------------------------------------------
// list_submissions — 401 maps to Auth
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_submissions_401_returns_auth_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/forms/api/forms/{FORM_ID}/submissions")))
        .respond_with(ResponseTemplate::new(401).set_body_string("Invalid API key"))
        .mount(&server)
        .await;

    let client = authed_client(&server);
    let err = client
        .list_submissions(FORM_ID, &SubmissionListParams::default())
        .await
        .unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}

// ---------------------------------------------------------------------------
// list_submissions — 404 maps to Http
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_submissions_404_returns_http_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/forms/api/forms/missing-form/submissions"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Form not found"))
        .mount(&server)
        .await;

    let client = authed_client(&server);
    let err = client
        .list_submissions("missing-form", &SubmissionListParams::default())
        .await
        .unwrap_err();

    match err {
        PauboxError::Http { status, body } => {
            assert_eq!(status, 404);
            assert_eq!(body, "Form not found");
        }
        other => panic!("unexpected: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// list_submissions — malformed JSON maps to Deserialize
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_submissions_malformed_json_returns_deserialize_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/forms/api/forms/{FORM_ID}/submissions")))
        .respond_with(ResponseTemplate::new(200).set_body_string("{not valid json"))
        .mount(&server)
        .await;

    let client = authed_client(&server);
    let err = client
        .list_submissions(FORM_ID, &SubmissionListParams::default())
        .await
        .unwrap_err();

    assert!(matches!(err, PauboxError::Deserialize(_)));
}

// ---------------------------------------------------------------------------
// list_submissions — no API key fails before any network call
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_submissions_without_api_key_fails_before_network() {
    let server = MockServer::start().await;

    // Expect zero requests; MockServer verifies expectations on drop.
    Mock::given(method("GET"))
        .and(path(format!("/forms/api/forms/{FORM_ID}/submissions")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [], "total": 0, "page": 1, "items": 50
        })))
        .expect(0)
        .mount(&server)
        .await;

    let client = anon_client(&server);
    let err = client
        .list_submissions(FORM_ID, &SubmissionListParams::default())
        .await
        .unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}

// ---------------------------------------------------------------------------
// export_submissions_csv — happy path (exact bytes)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn export_submissions_csv_returns_exact_bytes() {
    let server = MockServer::start().await;
    let csv = b"id,first_name,last_name\ns-1,Jane,Doe\ns-2,John,Smith\n";

    Mock::given(method("GET"))
        .and(path(format!(
            "/forms/api/forms/{FORM_ID}/submissions/submission-csv"
        )))
        .and(header("Authorization", format!("Bearer {API_KEY}")))
        .respond_with(ResponseTemplate::new(200).set_body_raw(csv.to_vec(), "text/csv"))
        .expect(1)
        .mount(&server)
        .await;

    let client = authed_client(&server);
    let bytes = client.export_submissions_csv(FORM_ID).await.unwrap();

    assert_eq!(bytes, csv.to_vec());
}

// ---------------------------------------------------------------------------
// export_submissions_csv — 401 maps to Auth
// ---------------------------------------------------------------------------

#[tokio::test]
async fn export_submissions_csv_401_returns_auth_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!(
            "/forms/api/forms/{FORM_ID}/submissions/submission-csv"
        )))
        .respond_with(ResponseTemplate::new(401).set_body_string("Invalid API key"))
        .mount(&server)
        .await;

    let client = authed_client(&server);
    let err = client.export_submissions_csv(FORM_ID).await.unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}

// ---------------------------------------------------------------------------
// export_submissions_csv — 404 maps to Http
// ---------------------------------------------------------------------------

#[tokio::test]
async fn export_submissions_csv_404_returns_http_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/forms/api/forms/missing-form/submissions/submission-csv",
        ))
        .respond_with(ResponseTemplate::new(404).set_body_string("Form not found"))
        .mount(&server)
        .await;

    let client = authed_client(&server);
    let err = client
        .export_submissions_csv("missing-form")
        .await
        .unwrap_err();

    match err {
        PauboxError::Http { status, body } => {
            assert_eq!(status, 404);
            assert_eq!(body, "Form not found");
        }
        other => panic!("unexpected: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// export_submission_csv — happy path (submission id in the path)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn export_submission_csv_hits_submission_path_and_returns_exact_bytes() {
    let server = MockServer::start().await;
    let csv = b"id,first_name\ns-1,Jane\n";

    Mock::given(method("GET"))
        .and(path(format!(
            "/forms/api/forms/{FORM_ID}/submissions/submission-csv/{SUBMISSION_ID}"
        )))
        .and(header("Authorization", format!("Bearer {API_KEY}")))
        .respond_with(ResponseTemplate::new(200).set_body_raw(csv.to_vec(), "text/csv"))
        .expect(1)
        .mount(&server)
        .await;

    let client = authed_client(&server);
    let bytes = client
        .export_submission_csv(FORM_ID, SUBMISSION_ID)
        .await
        .unwrap();

    assert_eq!(bytes, csv.to_vec());
}

// ---------------------------------------------------------------------------
// export_submission_csv — 401 maps to Auth
// ---------------------------------------------------------------------------

#[tokio::test]
async fn export_submission_csv_401_returns_auth_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!(
            "/forms/api/forms/{FORM_ID}/submissions/submission-csv/{SUBMISSION_ID}"
        )))
        .respond_with(ResponseTemplate::new(401).set_body_string("Invalid API key"))
        .mount(&server)
        .await;

    let client = authed_client(&server);
    let err = client
        .export_submission_csv(FORM_ID, SUBMISSION_ID)
        .await
        .unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}

// ---------------------------------------------------------------------------
// export_submission_csv — 404 maps to Http
// ---------------------------------------------------------------------------

#[tokio::test]
async fn export_submission_csv_404_returns_http_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!(
            "/forms/api/forms/{FORM_ID}/submissions/submission-csv/missing"
        )))
        .respond_with(ResponseTemplate::new(404).set_body_string("Submission not found"))
        .mount(&server)
        .await;

    let client = authed_client(&server);
    let err = client
        .export_submission_csv(FORM_ID, "missing")
        .await
        .unwrap_err();

    match err {
        PauboxError::Http { status, body } => {
            assert_eq!(status, 404);
            assert_eq!(body, "Submission not found");
        }
        other => panic!("unexpected: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// export_submission_pdf — happy path (path shape + exact bytes)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn export_submission_pdf_hits_pdf_path_and_returns_exact_bytes() {
    let server = MockServer::start().await;
    // A minimal PDF-ish payload: header magic plus some binary bytes.
    let pdf = b"%PDF-1.7\n\x00\x01\x02binary\xff\xfe\n%%EOF".to_vec();

    Mock::given(method("GET"))
        .and(path(format!(
            "/forms/api/forms/{FORM_ID}/submissions/{SUBMISSION_ID}/submission-pdf"
        )))
        .and(header("Authorization", format!("Bearer {API_KEY}")))
        .respond_with(ResponseTemplate::new(200).set_body_raw(pdf.clone(), "application/pdf"))
        .expect(1)
        .mount(&server)
        .await;

    let client = authed_client(&server);
    let bytes = client
        .export_submission_pdf(FORM_ID, SUBMISSION_ID)
        .await
        .unwrap();

    assert_eq!(bytes, pdf);
}

// ---------------------------------------------------------------------------
// export_submission_pdf — 401 maps to Auth
// ---------------------------------------------------------------------------

#[tokio::test]
async fn export_submission_pdf_401_returns_auth_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!(
            "/forms/api/forms/{FORM_ID}/submissions/{SUBMISSION_ID}/submission-pdf"
        )))
        .respond_with(ResponseTemplate::new(401).set_body_string("Invalid API key"))
        .mount(&server)
        .await;

    let client = authed_client(&server);
    let err = client
        .export_submission_pdf(FORM_ID, SUBMISSION_ID)
        .await
        .unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}

// ---------------------------------------------------------------------------
// export_submission_pdf — 404 maps to Http
// ---------------------------------------------------------------------------

#[tokio::test]
async fn export_submission_pdf_404_returns_http_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!(
            "/forms/api/forms/{FORM_ID}/submissions/missing/submission-pdf"
        )))
        .respond_with(ResponseTemplate::new(404).set_body_string("Submission not found"))
        .mount(&server)
        .await;

    let client = authed_client(&server);
    let err = client
        .export_submission_pdf(FORM_ID, "missing")
        .await
        .unwrap_err();

    match err {
        PauboxError::Http { status, .. } => assert_eq!(status, 404),
        other => panic!("unexpected: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// export_submission_pdf — no API key fails before any network call
// ---------------------------------------------------------------------------

#[tokio::test]
async fn export_submission_pdf_without_api_key_fails_before_network() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!(
            "/forms/api/forms/{FORM_ID}/submissions/{SUBMISSION_ID}/submission-pdf"
        )))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(b"%PDF-1.7".to_vec(), "application/pdf"),
        )
        .expect(0)
        .mount(&server)
        .await;

    let client = anon_client(&server);
    let err = client
        .export_submission_pdf(FORM_ID, SUBMISSION_ID)
        .await
        .unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}

// ---------------------------------------------------------------------------
// Submission::form_data_json — failure on unparseable form_data
// ---------------------------------------------------------------------------

#[tokio::test]
async fn form_data_json_fails_on_unparseable_form_data() {
    let server = MockServer::start().await;

    // `form_data` is a plain string on the wire, so a value that is not
    // itself valid JSON still deserializes into a Submission — only the
    // helper fails.
    Mock::given(method("GET"))
        .and(path(format!("/forms/api/forms/{FORM_ID}/submissions")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{
                "id": SUBMISSION_ID,
                "form_id": FORM_ID,
                "form_data": "this is not json {",
                "storage_type": "database",
                "created_at": "2026-02-01T12:34:56Z"
            }],
            "total": 1,
            "page": 1,
            "items": 50
        })))
        .mount(&server)
        .await;

    let client = authed_client(&server);
    let page = client
        .list_submissions(FORM_ID, &SubmissionListParams::default())
        .await
        .unwrap();

    let err = page.data[0].form_data_json().unwrap_err();
    assert!(matches!(err, PauboxError::Deserialize(_)));
}
