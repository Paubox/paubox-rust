# Paubox Rust SDK — API Reference

## Overview

The SDK exposes three independent clients and shared error/helper types:

| Type | Module | Auth required | API Base URL |
|------|--------|:-------------:|--------------|
| `PauboxClient` | `paubox` | Yes — `Token token=<key>` | `https://api.paubox.com/v1/email/` |
| `FormsClient` | `paubox::forms` | No | `https://api.paubox.com/v1/forms` |
| `WebhooksClient` | `paubox::webhooks` | Yes — `Bearer <scoped key>` | `https://api.paubox.com/v1/webhooks` |

All three are reachable from the top-level `paubox` crate:

```rust
use paubox::{PauboxClient, PauboxError};
use paubox::forms::FormsClient;
use paubox::webhooks::WebhooksClient;
```

Each sits behind a cargo feature, all enabled by default:

```toml
paubox = { version = "2", default-features = false, features = ["webhooks"] }
```

---

## `PauboxClient`

Holds API credentials and a shared HTTP connection pool.  Create once and reuse.

### Constructors

#### `PauboxClient::new(api_key) -> PauboxClient`

| Parameter | Type | Description |
|-----------|------|-------------|
| `api_key` | `impl Into<String>` | Your Paubox API key |

Uses the default Email API base URL (`https://api.paubox.com/v1/email/`).  Use `PauboxClient::builder()` to override the base URL or timeout.

#### `PauboxClient::from_env() -> Result<PauboxClient, PauboxError>`

Reads credentials from environment variables:

| Variable | Description |
|----------|-------------|
| `PAUBOX_API_KEY` | API key |

Returns `PauboxError::EnvVar` if the variable is absent or empty.

#### `PauboxClient::builder() -> PauboxClientBuilder`

Returns a builder supporting optional overrides.

### `PauboxClientBuilder`

| Method | Type | Description |
|--------|------|-------------|
| `.api_key(key)` | `impl Into<String>` | **Required.** API key |
| `.base_url(url)` | `url::Url` | Override the Email API base URL (useful for testing) |
| `.timeout(duration)` | `std::time::Duration` | Per-request timeout |
| `.build()` | — | Returns `Result<PauboxClient, PauboxError>` |

### `PauboxClient::forms() -> FormsClient`

Returns a `FormsClient` that reuses the same underlying HTTP connection pool.

---

## Email API — `PauboxClient` methods

### `send_message(message: &Message) -> Result<SendResponse, PauboxError>`

Send a message through the Paubox Email API.

```
POST https://api.paubox.com/v1/email/messages
Authorization: Token token={api_key}
Content-Type: application/json
```

**Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `message` | `&Message` | The message to send. Use `Message::builder()` to construct. |

**Returns** `SendResponse` on success (HTTP 200).

**Errors**
- `PauboxError::Auth` — invalid credentials (HTTP 401)
- `PauboxError::Http` — other non-2xx responses
- `PauboxError::Request` — network or TLS failure
- `PauboxError::Deserialize` — unexpected response body

---

### `get_email_disposition(source_tracking_id: &str) -> Result<DispositionResponse, PauboxError>`

Retrieve per-recipient delivery status for a sent message.

```
GET https://api.paubox.com/v1/email/message_receipt?sourceTrackingId={id}
Authorization: Token token={api_key}
```

**Parameters**

| Parameter | Type | Description |
|-----------|------|-------------|
| `source_tracking_id` | `&str` | Value from `SendResponse::source_tracking_id` |

**Returns** `DispositionResponse` on success.

---

### `api_status() -> Result<(), PauboxError>`

Check that the API is reachable and credentials are valid.

```
GET https://api.paubox.com/v1/email/
Authorization: Token token={api_key}
```

Returns `Ok(())` on success.

---

## `Message`

Use `Message::builder()` to construct.

### Required fields

| Field | Type | Description |
|-------|------|-------------|
| `from` | `String` | Sender email address |
| `to` | `Vec<String>` | One or more recipient email addresses |
| `subject` | `String` | Email subject line |

### Optional fields

| Field | Type | Description |
|-------|------|-------------|
| `reply_to` | `Option<String>` | Reply-to address |
| `cc` | `Option<Vec<String>>` | CC recipients |
| `bcc` | `Option<Vec<String>>` | BCC recipients |
| `text_content` | `Option<String>` | Plain-text body |
| `html_content` | `Option<String>` | HTML body (auto-base64-encoded on the wire) |
| `attachments` | `Vec<Attachment>` | File attachments |
| `allow_non_tls` | `Option<bool>` | Allow non-TLS delivery (non-PHI only) |
| `force_secure_notification` | `Option<bool>` | Force secure portal notification |

### `MessageBuilder` methods

All setter methods return `Self` for chaining.  Call `.build()` to validate and produce a `Message`.

| Method | Description |
|--------|-------------|
| `.from(address)` | Set sender |
| `.to(addresses)` | Add recipients (accepts any `IntoIterator`) |
| `.subject(subject)` | Set subject |
| `.reply_to(address)` | Set reply-to |
| `.cc(addresses)` | Set CC list |
| `.bcc(addresses)` | Set BCC list |
| `.text_content(text)` | Set plain-text body |
| `.html_content(html)` | Set HTML body |
| `.attachment(att)` | Add one attachment |
| `.attachments(atts)` | Add multiple attachments |
| `.allow_non_tls(bool)` | Set allowNonTLS flag |
| `.force_secure_notification(bool)` | Set forceSecureNotification flag |
| `.build()` | Returns `Result<Message, PauboxError>` |

---

## `Attachment`

| Field | Type | Description |
|-------|------|-------------|
| `file_name` | `String` | Filename shown to recipient |
| `content_type` | `String` | MIME type (e.g. `"application/pdf"`) |
| `content` | `String` | Base64-encoded file contents |

### Constructors

#### `Attachment::from_bytes(file_name, content_type, data: &[u8]) -> Attachment`

Base64-encodes `data` automatically.

#### `Attachment::from_base64(file_name, content_type, content) -> Attachment`

Accepts an already-encoded string.

---

## `SendResponse`

| Field | Type | Description |
|-------|------|-------------|
| `source_tracking_id` | `String` | Pass to `get_email_disposition()` to check status |
| `message` | `String` | Human-readable confirmation from the API |

---

## `DispositionResponse`

| Field | Type | Description |
|-------|------|-------------|
| `source_tracking_id` | `String` | The tracking ID queried |
| `message_deliveries` | `Vec<MessageDelivery>` | Per-recipient delivery records |
| `errors` | `Vec<ApiError>` | API-level errors, if any |

### `MessageDelivery`

| Field | Type | Description |
|-------|------|-------------|
| `recipient` | `String` | Recipient email address |
| `delivery_status` | `String` | e.g. `"delivered"`, `"bounced"`, `"pending"` |
| `delivery_time` | `Option<String>` | ISO 8601 delivery timestamp; `None` if not yet delivered |
| `opened_status` | `String` | `"opened"` or `"unopened"` |
| `opened_time` | `Option<String>` | ISO 8601 open timestamp; `None` if not yet opened |

### `ApiError`

| Field | Type | Description |
|-------|------|-------------|
| `code` | `Option<serde_json::Value>` | Numeric error code |
| `status` | `Option<String>` | Short status label |
| `title` | `Option<String>` | Human-readable title |
| `details` | `Option<String>` | Additional detail |

---

## `FormsClient`

Client for the Paubox Forms API.  No authentication required.

### Constructors

#### `FormsClient::new() -> FormsClient`

Uses the default Forms API base URL (`https://api.paubox.com/v1/forms`).

#### `FormsClient::with_base_url(url: Url) -> FormsClient`

Override the base URL (useful for testing with a mock server).

### `get_form(form_id: &str) -> Result<Form, PauboxError>`

Retrieve a form definition by UUID.

```
GET https://api.paubox.com/v1/forms/public/form_data/{form_id}
```

**Returns** `Form` on success (HTTP 200).  
**Errors** `PauboxError::Http { status: 404, .. }` if the form is not found.

---

### `submit_form(form_id: &str, submission: &FormSubmission) -> Result<(), PauboxError>`

Submit a respondent's answers.  Maximum request size: **250 MB**.

```
POST https://api.paubox.com/v1/forms/api/forms/{form_id}/submissions
Content-Type: application/json
```

**Returns** `Ok(())` on success (HTTP 201).  
**Errors** `PauboxError::Http { status: 400, .. }` for missing/invalid `form_data`,
`PauboxError::Http { status: 404, .. }` if the form is not found.

---

## `Form`

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | UUID of the form |
| `title` | `String` | Form title |
| `description` | `Option<String>` | Optional description |
| `form_html` | `Option<String>` | Rendered HTML for embedding |
| `form_json` | `Option<serde_json::Value>` | JSON schema describing form fields |
| `form_css` | `Option<String>` | CSS styles |
| `active` | `bool` | Whether the form is accepting submissions |
| `signable` | `bool` | Whether the form supports electronic signatures |
| `submission_count` | `u64` | Total submissions received |
| `customer_id` | `u64` | Owning Paubox customer ID |
| `created_at` | `String` | ISO 8601 creation timestamp |
| `updated_at` | `String` | ISO 8601 last-updated timestamp |

---

## `FormSubmission`

| Field | Type | Description |
|-------|------|-------------|
| `form_data` | `serde_json::Value` | Key-value pairs matching the form's `form_json` schema |
| `attachments` | `Vec<FormAttachment>` | Optional file attachments |

### `FormSubmissionBuilder` methods

| Method | Description |
|--------|-------------|
| `.form_data(value)` | **Required.** Set form field data |
| `.attachment(att)` | Add one attachment |
| `.attachments(atts)` | Add multiple attachments |
| `.build()` | Returns `Result<FormSubmission, PauboxError>` |

---

## `FormAttachment`

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Filename (e.g. `"consent.pdf"`) |
| `content` | `String` | Base64-encoded file contents |

### Constructors

#### `FormAttachment::from_bytes(name, data: &[u8]) -> FormAttachment`

#### `FormAttachment::from_base64(name, content) -> FormAttachment`

---

## `PauboxError`

| Variant | Description |
|---------|-------------|
| `Http { status: u16, body: String }` | Non-2xx HTTP response |
| `Auth(String)` | HTTP 401 — invalid or missing credentials |
| `Request(reqwest::Error)` | Network or TLS failure |
| `Deserialize(serde_json::Error)` | Response body could not be parsed |
| `Url(url::ParseError)` | URL construction failed |
| `EnvVar(String)` | Required environment variable is absent or empty |
| `Validation(String)` | Argument validation failed before any network call |

### Error handling pattern

```rust
use paubox::PauboxError;

match client.send_message(&msg).await {
    Ok(resp) => println!("Sent: {}", resp.source_tracking_id),
    Err(PauboxError::Auth(detail)) => {
        eprintln!("Authentication failed: {detail}");
    }
    Err(PauboxError::Http { status, body }) => {
        eprintln!("API error {status}: {body}");
    }
    Err(PauboxError::Validation(msg)) => {
        eprintln!("Invalid request: {msg}");
    }
    Err(e) => eprintln!("Unexpected error: {e}"),
}
```

---

## `WebhooksClient`

Manages webhook subscriptions: which URL Paubox notifies, for which events.

Behind the `webhooks` cargo feature (on by default).

Authenticates with a **scoped API key** sent as `Authorization: Bearer <key>` — the same scheme as Forms, and a different one from the Email API's `Token token=`. Every endpoint is authenticated, so unlike `FormsClient` there is no keyless mode.

Which events a key may subscribe to is decided by its scopes: an event it is not scoped for is refused with `403`, an unrecognised one with `422`.

> **Breaking change in 2.0.** These five methods previously hung off `PauboxClient`, targeted the legacy Email API at `/v1/email/webhook_endpoints`, and took **`i64`** ids. They now live on `WebhooksClient`, target the webhooks service, and take **UUID `&str`** ids. `WebhookEndpoint` changed shape with the service: `active: Option<bool>` became `status: String`, and `signing_key`/`api_key` are gone. The module also moved from the `email` feature to its own `webhooks` feature, because the service carries Forms events too and is no longer email-specific.

### Constructors

```rust
WebhooksClient::new(api_key: impl Into<String>) -> Self
WebhooksClient::with_base_url(api_key: impl Into<String>, base_url: Url) -> Self
WebhooksClient::with_http(api_key: impl Into<String>, http: reqwest::Client) -> Self
```

`with_base_url` is for tests pointing at a mock server; `with_http` reuses an existing connection pool.

### `list_webhook_endpoints(&ListWebhookEndpointsParams) -> Result<WebhookEndpointList, PauboxError>`

Lists the endpoints this key can act on. Endpoints carrying an event the key is not scoped for are filtered out by the service.

`WebhookEndpointList { data: Vec<WebhookEndpoint>, page_info: PageInfo }`.

**`page_info.count` is the total number of matching endpoints, not the length of the returned page.**

### `create_webhook_endpoint(target_url: &str, events: &[&str]) -> Result<CreatedWebhookEndpoint, PauboxError>`

Subscribes a URL to one or more events. `target_url` must be an `https` URL resolving to a publicly routable address.

```rust
let created = client
    .create_webhook_endpoint("https://hooks.example.com/paubox", &["forms.submission.created"])
    .await?;
println!("{}", created.signing_secret); // persist this now
```

`signing_secret` is returned **only here** — it is absent from `get_webhook_endpoint` and `list_webhook_endpoints`. A lost secret means replacing the endpoint.

Event names are **not** validated client-side: the catalog is owned by the service and grows without a crate release.

### `get_webhook_endpoint(id: &str) -> Result<WebhookEndpoint, PauboxError>`

A malformed UUID and another tenant's id both return `404` with the same message — the service gives no existence oracle. A non-UUID `id` returns `PauboxError::Validation` before any request.

### `update_webhook_endpoint(id: &str, changes: UpdateWebhookEndpoint) -> Result<WebhookEndpoint, PauboxError>`

Partial update — only the fields set on `changes` are sent, so an update that moves `target_url` leaves `events` and `status` untouched.

```rust
client.update_webhook_endpoint(&id, UpdateWebhookEndpoint {
    status: Some("disabled".to_string()),
    ..Default::default()
}).await?;
```

### `delete_webhook_endpoint(id: &str) -> Result<(), PauboxError>`

Deletes the endpoint, stopping every event on it. The service answers `204` with no body.

### Webhook types

| Type | Notes |
|---|---|
| `WebhookEndpoint` | `id` (UUID `String`), `target_url`, `status` (`"active"`/`"disabled"`), `events`, `created_at`, `updated_at`. Timestamps are RFC 3339 with microsecond precision and a `+00:00` offset, kept as `String`. |
| `CreatedWebhookEndpoint` | Flattens `WebhookEndpoint` and adds `signing_secret`. |
| `UpdateWebhookEndpoint` | `target_url`, `status`, `events` — all `Option`, all skipped when `None`. |
| `WebhookEndpointList` | `data` plus `page_info`. |
| `ListWebhookEndpointsParams` | `page`, `items` — both `Option<u32>`. |

### Webhook errors

Failures carry the service's own `{"message": ...}` rather than an opaque status: `401` maps to `PauboxError::Auth`, everything else to `PauboxError::Http { status, body }` with `body` holding that message. Local id validation returns `PauboxError::Validation`.

Note a duplicate `target_url` is **`422`, not `409`**.
