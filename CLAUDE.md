# Paubox Rust SDK — Developer Guide

## Toolchain

- **MSRV:** Rust 1.86
- **Edition:** 2021
- Install: `rustup update stable`

## Build & check

```sh
# Build with all features
cargo build --all-features

# Typecheck only (faster)
cargo check --all-features

# Lints (must pass with zero warnings)
cargo clippy --all-features -- -D warnings

# Format check
cargo fmt --check

# Generate docs (opens in browser)
cargo doc --all-features --no-deps --open
```

## Testing

```sh
# Run all unit + integration mock tests
cargo test --all-features

# Run a single test by name
cargo test send_message_returns_tracking_id

# Run integration tests that hit the live API (requires credentials)
cargo test --all-features -- --ignored
```

Live integration tests require:

| Variable | Description |
|----------|-------------|
| `PAUBOX_API_KEY` | Your Paubox API key |
| `PAUBOX_API_USER` | Your API user / endpoint name |
| `PAUBOX_FROM` | Sender address (for email integration tests) |
| `PAUBOX_TO` | Recipient address (for email integration tests) |
| `PAUBOX_FORM_ID` | Form UUID (for Forms integration tests) |

## Running examples

All examples read credentials from environment variables:

```sh
export PAUBOX_API_KEY="your-key"
export PAUBOX_API_USER="your-user"

cargo run --example send_email
cargo run --example check_disposition -- <source_tracking_id>
cargo run --example get_form -- <form_uuid>
cargo run --example submit_form -- <form_uuid>
```

## Feature flag combinations to verify

```sh
cargo check                                       # default (email + forms)
cargo check --no-default-features --features email
cargo check --no-default-features --features forms
# Note: at least one feature is required. A bare `--no-default-features`
# build is intentionally rejected by a `compile_error!` in `src/lib.rs`.
```

## Publishing to crates.io

```sh
# Dry-run check (catches missing metadata, path deps, etc.)
cargo publish --dry-run

# Actual publish (requires `cargo login` first)
cargo publish
```

Before publishing:
- All tests pass: `cargo test --all-features`
- No clippy warnings: `cargo clippy --all-features -- -D warnings`
- `Cargo.toml` version bumped appropriately
- `CHANGELOG` or release notes updated

## Repository conventions

- **No `unsafe` code** — `#![forbid(unsafe_code)]` is intentionally _not_ set but the policy is enforced in review
- **No path dependencies** in `Cargo.toml` (would break crates.io publish)
- **Doc comments required** on all `pub` items — use `///` for items, `//!` for modules
- **No `unwrap()` / `expect()` in library code** — return errors via `Result`; `expect()` is acceptable in examples and tests
- **Error variants** — add new `PauboxError` variants rather than using `anyhow` or string errors
- **Tests** — new endpoints need at least: happy path, 401, 404/400, malformed JSON, and builder validation tests
- **Wire format** — the JSON key mapping lives in `Message::to_wire()` in `src/email/message.rs`; keep it in sync with any API changes
