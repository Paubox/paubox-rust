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
| `forms` | ✓ | Manage forms, list/export submissions, submit responses |

## Quick start

### Send an email

```rust
use paubox::{PauboxClient, email::Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PauboxClient::new("YOUR_API_KEY", "YOUR_API_USER");

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

### Manage forms with a scoped API key

Administrative Forms endpoints require an API key carrying the `"forms"` scope, sent as a Bearer token. Create an authenticated client with `FormsClient::with_api_key` (or `FormsClient::builder()` for custom options):

```rust
use paubox::forms::{CreateForm, FormListParams, FormsClient, SubmissionListParams, UpdateForm};
use serde_json::json;

let client = FormsClient::with_api_key("key-with-forms-scope");

// List forms with filters and pagination.
// `customer_id` is required for API-key callers — pass the customer the key belongs to.
let page = client
    .list_forms(&FormListParams::default().customer_id(42).active(true).items(25))
    .await?;
println!("{} of {} forms", page.results.len(), page.page_info.count);

// Create, update, archive
let form = CreateForm::builder()
    .title("Patient intake")
    .customer_id(42)
    .form_json(json!({"fields": []}))
    .build()?;
let id = client.create_form(&form).await?;
client.update_form(&id, &UpdateForm::default().active(true)).await?;
client.archive_form(&id).await?; // and unarchive_form / copy_form

// Stats, submissions, exports
let stats = client.form_stats(None).await?;
let subs = client.list_submissions(&id, &SubmissionListParams::default()).await?;
let csv = client.export_submissions_csv(&id).await?;
let pdf = client.export_submission_pdf(&id, &subs.data[0].id).await?;
```

Available management methods: `list_forms`, `create_form`, `get_form_by_id`, `update_form`, `archive_form`, `unarchive_form`, `copy_form`, `form_stats`, `list_submissions`, `export_submissions_csv`, `export_submission_csv`, `export_submission_pdf`.

## Credentials

### Constructor

```rust
let client = PauboxClient::new("api-key", "api-user");
```

### Environment variables

```rust
// Reads PAUBOX_API_KEY and PAUBOX_API_USER
let client = PauboxClient::from_env()?;
```

| Variable | Description |
|----------|-------------|
| `PAUBOX_API_KEY` | Your Paubox API key |
| `PAUBOX_API_USER` | Your API user / endpoint name |

### Builder (with custom options)

```rust
use std::time::Duration;

let client = PauboxClient::builder()
    .api_key("my-key")
    .api_user("my-user")
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

The Forms API uses **scoped API keys**:

- **Public endpoints** — `get_form` and `submit_form` require no API key. They are intended to be called on behalf of form respondents.
- **Administrative endpoints** — everything else (managing forms, listing and exporting submissions) requires an API key with the `"forms"` scope, sent as `Authorization: Bearer <api_key>`. Calling a protected method on a client without an API key fails with `PauboxError::Auth` before any network request; an invalid or unscoped key is rejected by the server with 401 (also surfaced as `PauboxError::Auth`), and cross-customer access with 403 (`PauboxError::Http`).

`FormsClient` can be created independently of `PauboxClient`:

```rust
use std::time::Duration;
use paubox::forms::FormsClient;

// Unauthenticated (public endpoints only)
let client = FormsClient::new();

// Authenticated (all endpoints)
let client = FormsClient::with_api_key("key-with-forms-scope");

// Builder with custom options
let client = FormsClient::builder()
    .api_key("key-with-forms-scope")
    .timeout(Duration::from_secs(30))
    .build()?;
```

If you already have a `PauboxClient`, you can reuse its connection pool:

```rust
let forms = client.forms(); // reuses the underlying reqwest::Client
```

## API reference

See [`api.md`](api.md) for full documentation of all types, methods, and fields.

## MSRV

Rust 1.86 or later.

## License

Apache 2.0 — see [LICENSE](LICENSE).
## 💬 Community & support

Questions, ideas, or want to share what you built? Join the **[Paubox Community](https://github.com/Paubox/community/discussions)** — the single home for discussions across every Paubox SDK and API.

🔐 Found a security issue? Email **devops@paubox.com** — please don't post it publicly.
