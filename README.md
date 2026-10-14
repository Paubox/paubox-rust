# paubox

Async Rust SDK for the [Paubox](https://www.paubox.com) Email API and Forms API.

Paubox is a HITRUST-certified platform for sending HIPAA-compliant email and collecting patient data through secure forms.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
paubox = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### Feature flags

Both features are enabled by default. Disable one if you don't need it:

```toml
# Email API only
paubox = { version = "0.1", default-features = false, features = ["email"] }

# Forms API only
paubox = { version = "0.1", default-features = false, features = ["forms"] }
```

| Feature | Default | Description |
|---------|:-------:|-------------|
| `email` | ✓ | Send HIPAA-compliant email, track delivery status |
| `forms` | ✓ | Retrieve form definitions, submit form responses |

## Quick start

### Send an email

```rust
use paubox::{PauboxClient, email::Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PauboxClient::new("YOUR_API_KEY");

    let msg = Message::builder()
        .from("you@yourdomain.com")
        .to(["patient@example.com"])
        .subject("Your lab results are ready")
        .text_content("Please log in to the patient portal to view your results.")
        .build()?;

    let resp = client.send_message(&msg).await?;
    println!("Tracking ID: {}", resp.source_tracking_id);
    Ok(())
}
```

### Check email delivery status

```rust
let disposition = client.get_email_disposition(&resp.source_tracking_id).await?;
for d in &disposition.message_deliveries {
    println!("{}: {} (opened: {})", d.recipient, d.delivery_status, d.opened_status);
}
```

### Send a message with attachments

```rust
use paubox::email::{Attachment, Message};

let pdf_bytes = std::fs::read("report.pdf")?;
let attachment = Attachment::from_bytes("report.pdf", "application/pdf", &pdf_bytes);

let msg = Message::builder()
    .from("you@yourdomain.com")
    .to(["patient@example.com"])
    .subject("Your report")
    .text_content("Please find your report attached.")
    .attachment(attachment)
    .build()?;
```

### Retrieve a form definition

```rust
use paubox::forms::FormsClient;

let client = FormsClient::new();
let form = client.get_form("YOUR-FORM-UUID").await?;
println!("Form: {} (active: {})", form.title, form.active);
```

### Submit a form response

```rust
use paubox::forms::{FormsClient, FormSubmission};
use serde_json::json;

let client = FormsClient::new();
let submission = FormSubmission::builder()
    .form_data(json!({
        "first_name": "Jane",
        "last_name": "Doe",
        "email": "jane@example.com"
    }))
    .build()?;
client.submit_form("YOUR-FORM-UUID", &submission).await?;
```

## Credentials

### Constructor

```rust
let client = PauboxClient::new("api-key");
```

### Environment variables

```rust
// Reads PAUBOX_API_KEY
let client = PauboxClient::from_env()?;
```

| Variable | Description |
|----------|-------------|
| `PAUBOX_API_KEY` | Your Paubox API key |

### Builder (with custom options)

```rust
use std::time::Duration;

let client = PauboxClient::builder()
    .api_key("my-key")
    .timeout(Duration::from_secs(30))
    .build()?;
```

## Error handling

All methods return `Result<T, PauboxError>`:

```rust
use paubox::PauboxError;

match client.send_message(&msg).await {
    Ok(resp) => println!("Sent: {}", resp.source_tracking_id),
    Err(PauboxError::Auth(msg)) => eprintln!("Auth failed: {msg}"),
    Err(PauboxError::Http { status, body }) => eprintln!("HTTP {status}: {body}"),
    Err(e) => eprintln!("Error: {e}"),
}
```

## Forms API note

The Forms API endpoints are **public** — no API key is required. They are intended to be called on behalf of form respondents. `FormsClient` can be created independently of `PauboxClient`.

If you already have a `PauboxClient`, you can reuse its connection pool:

```rust
let forms = client.forms(); // reuses the underlying reqwest::Client
```

## Webhooks

A webhook endpoint is a URL you own that Paubox notifies when something happens. `WebhooksClient` manages those subscriptions: which URL to notify, and for which events.

Unlike `FormsClient`, every method here needs a **scoped API key**, sent as `Authorization: Bearer <key>`:

```rust
use paubox::webhooks::WebhooksClient;

let client = WebhooksClient::new(std::env::var("PAUBOX_WEBHOOKS_API_KEY")?);
```

Which events a key may subscribe to follows from its scopes. The SDK does not inspect them — the service refuses an event the key isn't scoped for with 403, and an unrecognised event with 422.

**Available events**

| Event | Fires when |
| --- | --- |
| `forms.submission.created` | A form submission is received |
| `api_mail_log_delivered` | An Email API message is delivered |
| `api_mail_log_opened` | A delivered message is opened |
| `api_mail_log_temporary_failure` | Delivery fails temporarily |
| `api_mail_log_permanent_failure` | Delivery fails permanently |
| `email.inbound.received` | Inbound mail is received |

The four Email API events keep their legacy `api_mail_log_` names, so a receiver written
against the old Email API webhooks keeps working unchanged. Note these are the names on
the wire: `delivered` or `failed` on their own are rejected as unknown.

The SDK does not check these names. The catalog lives in the service, so an event added
later works here without upgrading the SDK.

### Subscribe a URL to an event

```rust
let created = client
    .create_webhook_endpoint("https://example.com/paubox-webhook", &["forms.submission.created"])
    .await?;

println!("{}", created.endpoint.id);    // UUID
println!("{}", created.signing_secret); // whsec_... — store this now
```

The endpoint itself is a flattened field, so its properties are reached through `created.endpoint`; `signing_secret` sits beside it.

`signing_secret` is on `CreatedWebhookEndpoint` and no other type, because the service returns it once and never again. There is no way to read it back: recovering from a lost secret means deleting the endpoint and creating another. Use it to verify that deliveries really came from Paubox.

### List, fetch, update and delete

```rust
use paubox::webhooks::{ListWebhookEndpointsParams, UpdateWebhookEndpoint};

let list = client
    .list_webhook_endpoints(&ListWebhookEndpointsParams::default())
    .await?;

for endpoint in &list.data {
    println!("{} {} {}", endpoint.id, endpoint.target_url, endpoint.status);
}
println!("{}", list.page_info.count); // total matching, not the length of data

let endpoint = client.get_webhook_endpoint(&created.endpoint.id).await?;

let updated = client
    .update_webhook_endpoint(
        &created.endpoint.id,
        UpdateWebhookEndpoint { status: Some("disabled".into()), ..Default::default() },
    )
    .await?;

client.delete_webhook_endpoint(&created.endpoint.id).await?;
```

Only the fields set on `UpdateWebhookEndpoint` are sent, so changing the status leaves the URL and events alone. Pausing deliveries without losing the subscription is what `"disabled"` is for; set `"active"` to resume. Setting `events` replaces the list rather than adding to it. Delete returns `()` — the service answers 204 with no body — and takes the signing secret with it.

Set `page` and `items` on `ListWebhookEndpointsParams` to paginate.

### Webhook errors

Failures are the same `PauboxError` the rest of the SDK returns, with the service's `{"message": "..."}` promoted onto it rather than a bare status. Subscribing a `target_url` that already has an endpoint is a 422, and an id belonging to another account is a 404 rather than a 403 — so one account cannot probe another's for existence. An id that is not UUID-shaped is rejected as `PauboxError::Validation` before any request is sent.

## API reference

See [`api.md`](api.md) for full documentation of all types, methods, and fields.

## MSRV

Rust 1.86 or later.

## License

Apache 2.0 — see [LICENSE](LICENSE).
## 💬 Community & support

Questions, ideas, or want to share what you built? Join the **[Paubox Community](https://github.com/Paubox/community/discussions)** — the single home for discussions across every Paubox SDK and API.

🔐 Found a security issue? Email **devops@paubox.com** — please don't post it publicly.
