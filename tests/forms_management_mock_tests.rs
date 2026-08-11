//! Mocked unit tests for the Paubox Forms API form-management endpoints
//! (scoped-API-key protected).

use paubox::{
    forms::{CreateForm, FormListParams, FormsClient, UpdateForm},
    PauboxError,
};
use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, MockServer, ResponseTemplate};

const API_KEY: &str = "test-api-key";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// An authenticated client pointed at the mock server, preserving the
/// `/forms` base path segment (regression guard for the dropped-segment bug).
fn make_client(server: &MockServer) -> FormsClient {
    let base_url = url::Url::parse(&format!("{}/forms", server.uri())).unwrap();
    FormsClient::builder()
        .api_key(API_KEY)
        .base_url(base_url)
        .build()
        .unwrap()
}

/// A client with no API key, for the fail-before-network tests.
fn make_unauthenticated_client(server: &MockServer) -> FormsClient {
    let base_url = url::Url::parse(&format!("{}/forms", server.uri())).unwrap();
    FormsClient::with_base_url(base_url)
}

/// The `Authorization` header the client must send on protected endpoints.
fn bearer() -> String {
    format!("Bearer {API_KEY}")
}

/// A full server-side Form serialization: every field present, matching the
/// pb_rforms wire shape.
fn full_form_json(id: &str, title: &str) -> serde_json::Value {
    json!({
        "id": id,
        "title": title,
        "description": "Initial intake form",
        "form_html": "<form>...</form>",
        "form_json": {"fields": ["first_name", "last_name"]},
        "form_css": "body { font-family: sans-serif; }",
        "vanity_url": "my-intake",
        "version": 2,
        "active": true,
        "customer_id": 42,
        "old_form_id": null,
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-02-01T00:00:00Z",
        "recipient": "admin@example.com",
        "signable": false,
        "signature_confirmation_label": null,
        "submission_count": 7,
        "type": "hosted",
        "subscription_list_id": null,
        "deleted": false,
        "archived": false
    })
}

// ===========================================================================
// list_forms
// ===========================================================================

#[tokio::test]
async fn list_forms_happy_path_parses_page() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/forms/api/forms"))
        .and(header("Authorization", bearer().as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [
                full_form_json("form-1", "Patient Intake"),
                full_form_json("form-2", "Consent"),
            ],
            "page_info": {"count": 2, "pages": 1, "page": 1, "items": 50}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server);
    let page = client.list_forms(&FormListParams::default()).await.unwrap();

    assert_eq!(page.results.len(), 2);
    assert_eq!(page.results[0].id, "form-1");
    assert_eq!(page.results[0].title, "Patient Intake");
    assert_eq!(page.results[0].version, 2);
    assert_eq!(page.results[0].vanity_url.as_deref(), Some("my-intake"));
    assert_eq!(page.results[0].type_.as_deref(), Some("hosted"));
    assert!(!page.results[0].archived);
    assert!(!page.results[0].deleted);
    assert_eq!(page.results[1].id, "form-2");
    assert_eq!(page.page_info.count, 2);
    assert_eq!(page.page_info.pages, 1);
    assert_eq!(page.page_info.page, 1);
    assert_eq!(page.page_info.items, 50);
}

#[tokio::test]
async fn list_forms_sends_pagination_query_params() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/forms/api/forms"))
        .and(query_param("customer_id", "42"))
        .and(query_param("page", "3"))
        .and(query_param("items", "25"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [],
            "page_info": {"count": 60, "pages": 3, "page": 3, "items": 25}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server);
    let params = FormListParams::default().customer_id(42).page(3).items(25);
    let page = client.list_forms(&params).await.unwrap();

    assert!(page.results.is_empty());
    assert_eq!(page.page_info.page, 3);
    assert_eq!(page.page_info.pages, 3);
}

#[tokio::test]
async fn list_forms_sends_filter_query_params() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/forms/api/forms"))
        .and(query_param("search", "intake"))
        .and(query_param("archived", "false"))
        .and(query_param("active", "true"))
        .and(query_param("order_by", "updated_at"))
        .and(query_param("order", "asc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [full_form_json("form-1", "Patient Intake")],
            "page_info": {"count": 1, "pages": 1, "page": 1, "items": 50}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server);
    let params = FormListParams::default()
        .search("intake")
        .archived(false)
        .active(true)
        .order_by("updated_at")
        .order("asc");
    let page = client.list_forms(&params).await.unwrap();

    assert_eq!(page.results.len(), 1);
}

#[tokio::test]
async fn list_forms_401_returns_auth_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/forms/api/forms"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let err = client
        .list_forms(&FormListParams::default())
        .await
        .unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}

#[tokio::test]
async fn list_forms_400_returns_http_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/forms/api/forms"))
        .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let err = client
        .list_forms(&FormListParams::default())
        .await
        .unwrap_err();

    match err {
        PauboxError::Http { status, body } => {
            assert_eq!(status, 400);
            assert_eq!(body, "Bad Request");
        }
        other => panic!("unexpected: {other:?}"),
    }
}

#[tokio::test]
async fn list_forms_malformed_json_returns_deserialize_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/forms/api/forms"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{invalid json"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let err = client
        .list_forms(&FormListParams::default())
        .await
        .unwrap_err();

    assert!(matches!(err, PauboxError::Deserialize(_)));
}

#[tokio::test]
async fn list_forms_without_api_key_fails_before_network() {
    // No mocks mounted: any request that reached the server would be
    // recorded, and we assert none was.
    let server = MockServer::start().await;

    let client = make_unauthenticated_client(&server);
    let err = client
        .list_forms(&FormListParams::default())
        .await
        .unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
    let received = server.received_requests().await.unwrap();
    assert!(
        received.is_empty(),
        "expected no network call, saw {} request(s)",
        received.len()
    );
}

// ===========================================================================
// create_form
// ===========================================================================

#[tokio::test]
async fn create_form_sends_required_body_and_returns_id() {
    let server = MockServer::start().await;

    // Exact body match: required keys present (version defaulted to 1), and
    // unset optional fields must be absent entirely.
    Mock::given(method("POST"))
        .and(path("/forms/api/forms"))
        .and(header("Authorization", bearer().as_str()))
        .and(body_json(json!({
            "title": "Patient Intake",
            "form_json": {"fields": ["first_name"]},
            "customer_id": 42,
            "version": 1
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"id": "3c9d9e8a-1111-2222-3333-444455556666"})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server);
    let form = CreateForm::builder()
        .title("Patient Intake")
        .customer_id(42)
        .form_json(json!({"fields": ["first_name"]}))
        .build()
        .unwrap();
    let id = client.create_form(&form).await.unwrap();

    assert_eq!(id, "3c9d9e8a-1111-2222-3333-444455556666");
}

#[tokio::test]
async fn create_form_sends_optional_fields_on_the_wire() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/forms/api/forms"))
        .and(body_json(json!({
            "title": "Consent",
            "form_json": {},
            "customer_id": 7,
            "version": 3,
            "description": "A consent form",
            "recipient": "a@x.com,b@x.com",
            "signable": true,
            "type": "marketing_form",
            "active": false
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "new-id"})))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server);
    let form = CreateForm::builder()
        .title("Consent")
        .customer_id(7)
        .form_json(json!({}))
        .version(3)
        .description("A consent form")
        .recipient("a@x.com,b@x.com")
        .signable(true)
        .type_("marketing_form")
        .active(false)
        .build()
        .unwrap();
    let id = client.create_form(&form).await.unwrap();

    assert_eq!(id, "new-id");
}

#[test]
fn create_form_builder_missing_title_fails_without_network() {
    let err = CreateForm::builder()
        .customer_id(42)
        .form_json(json!({}))
        .build()
        .unwrap_err();
    assert!(matches!(err, PauboxError::Validation(_)));
}

#[test]
fn create_form_builder_missing_customer_id_fails_without_network() {
    let err = CreateForm::builder()
        .title("Intake")
        .form_json(json!({}))
        .build()
        .unwrap_err();
    assert!(matches!(err, PauboxError::Validation(_)));
}

#[test]
fn create_form_builder_missing_form_json_fails_without_network() {
    let err = CreateForm::builder()
        .title("Intake")
        .customer_id(42)
        .build()
        .unwrap_err();
    assert!(matches!(err, PauboxError::Validation(_)));
}

#[tokio::test]
async fn create_form_401_returns_auth_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/forms/api/forms"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let form = CreateForm::builder()
        .title("Intake")
        .customer_id(42)
        .form_json(json!({}))
        .build()
        .unwrap();
    let err = client.create_form(&form).await.unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}

#[tokio::test]
async fn create_form_400_returns_http_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/forms/api/forms"))
        .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let form = CreateForm::builder()
        .title("Intake")
        .customer_id(42)
        .form_json(json!({}))
        .build()
        .unwrap();
    let err = client.create_form(&form).await.unwrap_err();

    match err {
        PauboxError::Http { status, .. } => assert_eq!(status, 400),
        other => panic!("unexpected: {other:?}"),
    }
}

#[tokio::test]
async fn create_form_malformed_json_returns_deserialize_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/forms/api/forms"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{invalid json"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let form = CreateForm::builder()
        .title("Intake")
        .customer_id(42)
        .form_json(json!({}))
        .build()
        .unwrap();
    let err = client.create_form(&form).await.unwrap_err();

    assert!(matches!(err, PauboxError::Deserialize(_)));
}

// ===========================================================================
// get_form_by_id
// ===========================================================================

#[tokio::test]
async fn get_form_by_id_unwraps_data_wrapper() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/forms/api/forms/form-1"))
        .and(header("Authorization", bearer().as_str()))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"data": full_form_json("form-1", "Patient Intake")})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server);
    let form = client.get_form_by_id("form-1").await.unwrap();

    assert_eq!(form.id, "form-1");
    assert_eq!(form.title, "Patient Intake");
    assert_eq!(form.version, 2);
    assert_eq!(form.customer_id, 42);
    assert_eq!(form.recipient.as_deref(), Some("admin@example.com"));
    assert_eq!(form.submission_count, 7);
    assert!(form.active);
    assert!(!form.archived);
}

#[tokio::test]
async fn get_form_by_id_401_returns_auth_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/forms/api/forms/form-1"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let err = client.get_form_by_id("form-1").await.unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}

#[tokio::test]
async fn get_form_by_id_404_returns_http_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/forms/api/forms/missing"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let err = client.get_form_by_id("missing").await.unwrap_err();

    match err {
        PauboxError::Http { status, .. } => assert_eq!(status, 404),
        other => panic!("unexpected: {other:?}"),
    }
}

#[tokio::test]
async fn get_form_by_id_missing_data_wrapper_returns_deserialize_error() {
    let server = MockServer::start().await;

    // A bare Form (no {"data": ...} wrapper) must fail to parse: the
    // authenticated endpoint always nests the form under "data".
    Mock::given(method("GET"))
        .and(path("/forms/api/forms/form-1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(full_form_json("form-1", "Patient Intake")),
        )
        .mount(&server)
        .await;

    let client = make_client(&server);
    let err = client.get_form_by_id("form-1").await.unwrap_err();

    assert!(matches!(err, PauboxError::Deserialize(_)));
}

// ===========================================================================
// update_form
// ===========================================================================

#[tokio::test]
async fn update_form_sends_only_set_fields() {
    let server = MockServer::start().await;

    // Exact body match: unset fields must be ABSENT from the JSON body, not
    // serialized as null (PATCH semantics — omitted fields stay unchanged).
    Mock::given(method("PUT"))
        .and(path("/forms/api/forms/form-1"))
        .and(header("Authorization", bearer().as_str()))
        .and(body_json(json!({"title": "Renamed", "active": false})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "detail": "Form updated successfully",
            "form_id": "form-1"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server);
    let update = UpdateForm::default().title("Renamed").active(false);
    client.update_form("form-1", &update).await.unwrap();
}

#[tokio::test]
async fn update_form_empty_update_sends_empty_object() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/forms/api/forms/form-1"))
        .and(body_json(json!({})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "detail": "Form updated successfully",
            "form_id": "form-1"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server);
    client
        .update_form("form-1", &UpdateForm::default())
        .await
        .unwrap();
}

#[tokio::test]
async fn update_form_401_returns_auth_error() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/forms/api/forms/form-1"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let err = client
        .update_form("form-1", &UpdateForm::default().title("x"))
        .await
        .unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}

#[tokio::test]
async fn update_form_404_returns_http_error() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/forms/api/forms/missing"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let err = client
        .update_form("missing", &UpdateForm::default().title("x"))
        .await
        .unwrap_err();

    match err {
        PauboxError::Http { status, .. } => assert_eq!(status, 404),
        other => panic!("unexpected: {other:?}"),
    }
}

// ===========================================================================
// archive_form / unarchive_form
// ===========================================================================

#[tokio::test]
async fn archive_form_posts_to_archive_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/forms/api/forms/form-1/archive"))
        .and(header("Authorization", bearer().as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"detail": "Form archived."})))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server);
    client.archive_form("form-1").await.unwrap();
}

#[tokio::test]
async fn archive_form_401_returns_auth_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/forms/api/forms/form-1/archive"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let err = client.archive_form("form-1").await.unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}

#[tokio::test]
async fn archive_form_404_returns_http_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/forms/api/forms/missing/archive"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let err = client.archive_form("missing").await.unwrap_err();

    match err {
        PauboxError::Http { status, .. } => assert_eq!(status, 404),
        other => panic!("unexpected: {other:?}"),
    }
}

#[tokio::test]
async fn unarchive_form_posts_to_unarchive_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/forms/api/forms/form-1/unarchive"))
        .and(header("Authorization", bearer().as_str()))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"detail": "Form unarchived."})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server);
    client.unarchive_form("form-1").await.unwrap();
}

#[tokio::test]
async fn unarchive_form_401_returns_auth_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/forms/api/forms/form-1/unarchive"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let err = client.unarchive_form("form-1").await.unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}

#[tokio::test]
async fn unarchive_form_404_returns_http_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/forms/api/forms/missing/unarchive"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let err = client.unarchive_form("missing").await.unwrap_err();

    match err {
        PauboxError::Http { status, .. } => assert_eq!(status, 404),
        other => panic!("unexpected: {other:?}"),
    }
}

// ===========================================================================
// copy_form
// ===========================================================================

#[tokio::test]
async fn copy_form_sends_body_and_parses_bare_form_response() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/forms/api/forms/copy"))
        .and(header("Authorization", bearer().as_str()))
        .and(body_json(json!({
            "form_id": "form-1",
            "title": "Intake (copy)"
        })))
        // The copy endpoint returns the new Form bare — no {"data": ...}
        // wrapper.
        .respond_with(
            ResponseTemplate::new(200).set_body_json(full_form_json("form-9", "Intake (copy)")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server);
    let copy = client.copy_form("form-1", "Intake (copy)").await.unwrap();

    assert_eq!(copy.id, "form-9");
    assert_eq!(copy.title, "Intake (copy)");
    assert_eq!(copy.customer_id, 42);
    assert_eq!(copy.version, 2);
}

#[tokio::test]
async fn copy_form_401_returns_auth_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/forms/api/forms/copy"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let err = client.copy_form("form-1", "Copy").await.unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}

#[tokio::test]
async fn copy_form_404_returns_http_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/forms/api/forms/copy"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let err = client.copy_form("missing", "Copy").await.unwrap_err();

    match err {
        PauboxError::Http { status, .. } => assert_eq!(status, 404),
        other => panic!("unexpected: {other:?}"),
    }
}

#[tokio::test]
async fn copy_form_malformed_json_returns_deserialize_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/forms/api/forms/copy"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{invalid json"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let err = client.copy_form("form-1", "Copy").await.unwrap_err();

    assert!(matches!(err, PauboxError::Deserialize(_)));
}

// ===========================================================================
// form_stats
// ===========================================================================

#[tokio::test]
async fn form_stats_without_customer_id_omits_query_param() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/forms/api/forms/stats"))
        .and(header("Authorization", bearer().as_str()))
        .and(query_param_is_missing("customer_id"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "active_form_count": 5,
            "total_submission_count": 120,
            "submissions_last_7_days": 9
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server);
    let stats = client.form_stats(None).await.unwrap();

    assert_eq!(stats.active_form_count, 5);
    assert_eq!(stats.total_submission_count, 120);
    assert_eq!(stats.submissions_last_7_days, 9);
}

#[tokio::test]
async fn form_stats_with_customer_id_sends_query_param() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/forms/api/forms/stats"))
        .and(query_param("customer_id", "42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "active_form_count": 1,
            "total_submission_count": 3,
            "submissions_last_7_days": 0
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server);
    let stats = client.form_stats(Some(42)).await.unwrap();

    assert_eq!(stats.active_form_count, 1);
    assert_eq!(stats.total_submission_count, 3);
    assert_eq!(stats.submissions_last_7_days, 0);
}

#[tokio::test]
async fn form_stats_401_returns_auth_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/forms/api/forms/stats"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let err = client.form_stats(None).await.unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)));
}

#[tokio::test]
async fn form_stats_400_returns_http_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/forms/api/forms/stats"))
        .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let err = client.form_stats(None).await.unwrap_err();

    match err {
        PauboxError::Http { status, .. } => assert_eq!(status, 400),
        other => panic!("unexpected: {other:?}"),
    }
}

#[tokio::test]
async fn form_stats_malformed_json_returns_deserialize_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/forms/api/forms/stats"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{invalid json"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let err = client.form_stats(None).await.unwrap_err();

    assert!(matches!(err, PauboxError::Deserialize(_)));
}
