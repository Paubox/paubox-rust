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

The Forms API endpoints are **public** — no API key is required. They are intended to be called on behalf of form respondents. `FormsClient` can be created independently of `PauboxClient`.

If you already have a `PauboxClient`, you can reuse its connection pool:

```rust
let forms = client.forms(); // reuses the underlying reqwest::Client
```

## API reference

See [`api.md`](api.md) for full documentation of all types, methods, and fields.

## MSRV

Rust 1.75 or later.

## License

Apache 2.0 — see [LICENSE](LICENSE).
