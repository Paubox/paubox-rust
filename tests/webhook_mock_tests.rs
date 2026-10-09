use paubox::webhooks::{ListWebhookEndpointsParams, UpdateWebhookEndpoint, WebhooksClient};
use paubox::PauboxError;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const ENDPOINT_ID: &str = "2ec66c21-bf48-48eb-8d28-f80b2d6b77c7";

/// Appends the `/v1/webhooks` base segment, so these also regression-test that
/// `Url::join` preserves it rather than replacing the path.
fn make_client(server: &MockServer) -> WebhooksClient {
    let base_url = url::Url::parse(&format!("{}/v1/webhooks", server.uri())).unwrap();
    WebhooksClient::with_base_url("test-key", base_url)
}

fn endpoint_json() -> serde_json::Value {
    serde_json::json!({
        "id": ENDPOINT_ID,
        "target_url": "https://hooks.example.com/paubox",
        "status": "active",
        "events": ["forms.submission.created"],
        "created_at": "2026-10-09T01:43:41.639968+00:00",
        "updated_at": "2026-10-09T01:43:41.639968+00:00"
    })
}

// ---------------------------------------------------------------------------
// list_webhook_endpoints
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_sends_bearer_auth_to_the_endpoints_resource() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/webhooks/endpoints"))
        // Bearer, not the Email API's "Token token=" — the whole reason this
        // is a separate client.
        .and(header("Authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [endpoint_json()],
            "page_info": { "count": 1, "items": 50 }
        })))
        .mount(&server)
        .await;

    let list = make_client(&server)
        .list_webhook_endpoints(&ListWebhookEndpointsParams::default())
        .await
        .unwrap();

    assert_eq!(list.data.len(), 1);
    assert_eq!(list.data[0].id, ENDPOINT_ID);
    assert_eq!(list.data[0].status, "active");
}

#[tokio::test]
async fn list_count_is_the_total_not_the_page_length() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/webhooks/endpoints"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [endpoint_json()],
            "page_info": { "count": 2, "items": 1 }
        })))
        .mount(&server)
        .await;

    let list = make_client(&server)
        .list_webhook_endpoints(&ListWebhookEndpointsParams::default())
        .await
        .unwrap();

    assert_eq!(list.page_info.count, 2);
    assert_eq!(list.data.len(), 1);
}

#[tokio::test]
async fn list_sends_pagination() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/webhooks/endpoints"))
        .and(query_param("page", "2"))
        .and(query_param("items", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [],
            "page_info": { "count": 0, "items": 1 }
        })))
        .mount(&server)
        .await;

    make_client(&server)
        .list_webhook_endpoints(&ListWebhookEndpointsParams {
            page: Some(2),
            items: Some(1),
        })
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// create_webhook_endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_returns_the_signing_secret() {
    let server = MockServer::start().await;

    let mut created = endpoint_json();
    created["signing_secret"] = serde_json::json!("whsec_abc123");

    Mock::given(method("POST"))
        .and(path("/v1/webhooks/endpoints"))
        .and(body_json(serde_json::json!({
            "target_url": "https://hooks.example.com/paubox",
            "events": ["forms.submission.created"]
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": created,
            "message": "Store this signing_secret now — it is not shown again."
        })))
        .mount(&server)
        .await;

    let created = make_client(&server)
        .create_webhook_endpoint(
            "https://hooks.example.com/paubox",
            &["forms.submission.created"],
        )
        .await
        .unwrap();

    assert_eq!(created.signing_secret, "whsec_abc123");
    assert_eq!(created.endpoint.id, ENDPOINT_ID);
}

#[tokio::test]
async fn create_validates_locally_before_sending() {
    let server = MockServer::start().await;
    let client = make_client(&server);

    // No mocks are mounted, so any request would 404 the test.
    assert!(matches!(
        client
            .create_webhook_endpoint("", &["forms.submission.created"])
            .await,
        Err(PauboxError::Validation(_))
    ));
    assert!(matches!(
        client
            .create_webhook_endpoint("https://e.test/h", &[])
            .await,
        Err(PauboxError::Validation(_))
    ));
}

/// The catalog belongs to the service and grows without a crate release, so an
/// unrecognised name must reach the server rather than be rejected here.
#[tokio::test]
async fn create_does_not_validate_event_names() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/webhooks/endpoints"))
        .and(body_json(serde_json::json!({
            "target_url": "https://e.test/h",
            "events": ["some.future.event"]
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {
                "id": ENDPOINT_ID,
                "target_url": "https://e.test/h",
                "status": "active",
                "events": ["some.future.event"],
                "created_at": "x",
                "updated_at": "y",
                "signing_secret": "whsec_x"
            }
        })))
        .mount(&server)
        .await;

    make_client(&server)
        .create_webhook_endpoint("https://e.test/h", &["some.future.event"])
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// get_webhook_endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_unwraps_the_data_envelope() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/v1/webhooks/endpoints/{ENDPOINT_ID}")))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "data": endpoint_json() })),
        )
        .mount(&server)
        .await;

    let endpoint = make_client(&server)
        .get_webhook_endpoint(ENDPOINT_ID)
        .await
        .unwrap();

    assert_eq!(endpoint.id, ENDPOINT_ID);
    assert_eq!(endpoint.target_url, "https://hooks.example.com/paubox");
}

#[tokio::test]
async fn get_rejects_a_non_uuid_before_any_request() {
    let server = MockServer::start().await;
    let client = make_client(&server);

    for bad in ["not-a-uuid", "../endpoints", ""] {
        assert!(
            matches!(
                client.get_webhook_endpoint(bad).await,
                Err(PauboxError::Validation(_))
            ),
            "{bad:?} should be refused before a request"
        );
    }
}

// ---------------------------------------------------------------------------
// update_webhook_endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn update_sends_only_the_fields_set() {
    let server = MockServer::start().await;

    let mut disabled = endpoint_json();
    disabled["status"] = serde_json::json!("disabled");

    Mock::given(method("PATCH"))
        .and(path(format!("/v1/webhooks/endpoints/{ENDPOINT_ID}")))
        // target_url and events are omitted entirely, not sent as null.
        .and(body_json(serde_json::json!({ "status": "disabled" })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "data": disabled })),
        )
        .mount(&server)
        .await;

    let updated = make_client(&server)
        .update_webhook_endpoint(
            ENDPOINT_ID,
            UpdateWebhookEndpoint {
                status: Some("disabled".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.status, "disabled");
}

// ---------------------------------------------------------------------------
// delete_webhook_endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn delete_handles_an_empty_204() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path(format!("/v1/webhooks/endpoints/{ENDPOINT_ID}")))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    make_client(&server)
        .delete_webhook_endpoint(ENDPOINT_ID)
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// errors — the service answers {"message": "..."}
// ---------------------------------------------------------------------------

#[tokio::test]
async fn promotes_the_service_message_onto_the_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/webhooks/endpoints"))
        .respond_with(
            ResponseTemplate::new(422).set_body_json(
                serde_json::json!({ "message": "target_url: must be an https URL" }),
            ),
        )
        .mount(&server)
        .await;

    let err = make_client(&server)
        .create_webhook_endpoint("http://insecure.test", &["forms.submission.created"])
        .await
        .unwrap_err();

    match err {
        PauboxError::Http { status, body } => {
            assert_eq!(status, 422);
            assert_eq!(body, "target_url: must be an https URL");
        }
        other => panic!("expected Http, got {other:?}"),
    }
}

#[tokio::test]
async fn maps_401_to_auth() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/webhooks/endpoints"))
        .respond_with(
            ResponseTemplate::new(401).set_body_json(serde_json::json!({ "message": "nope" })),
        )
        .mount(&server)
        .await;

    let err = make_client(&server)
        .list_webhook_endpoints(&ListWebhookEndpointsParams::default())
        .await
        .unwrap_err();

    assert!(matches!(err, PauboxError::Auth(_)), "got {err:?}");
}

#[tokio::test]
async fn names_the_event_on_a_403() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/webhooks/endpoints"))
        .respond_with(ResponseTemplate::new(403).set_body_json(
            serde_json::json!({ "message": "events: not entitled to 'api_mail_log_delivered'" }),
        ))
        .mount(&server)
        .await;

    let err = make_client(&server)
        .list_webhook_endpoints(&ListWebhookEndpointsParams::default())
        .await
        .unwrap_err();

    match err {
        PauboxError::Http { status, body } => {
            assert_eq!(status, 403);
            assert!(body.contains("not entitled"), "{body}");
        }
        other => panic!("expected Http, got {other:?}"),
    }
}

/// A duplicate target_url is 422, not 409 — mapping keyed on 409 would miss it.
#[tokio::test]
async fn duplicate_target_url_is_422() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/webhooks/endpoints"))
        .respond_with(ResponseTemplate::new(422).set_body_json(serde_json::json!({
            "message": "target_url: an endpoint already exists for this URL"
        })))
        .mount(&server)
        .await;

    let err = make_client(&server)
        .create_webhook_endpoint("https://e.test/h", &["forms.submission.created"])
        .await
        .unwrap_err();

    assert!(
        matches!(err, PauboxError::Http { status: 422, .. }),
        "{err:?}"
    );
}

#[tokio::test]
async fn falls_back_to_the_raw_body_when_not_the_usual_shape() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/webhooks/endpoints"))
        .respond_with(ResponseTemplate::new(500).set_body_string("plain text failure"))
        .mount(&server)
        .await;

    let err = make_client(&server)
        .list_webhook_endpoints(&ListWebhookEndpointsParams::default())
        .await
        .unwrap_err();

    match err {
        PauboxError::Http { status, body } => {
            assert_eq!(status, 500);
            assert_eq!(body, "plain text failure");
        }
        other => panic!("expected Http, got {other:?}"),
    }
}
