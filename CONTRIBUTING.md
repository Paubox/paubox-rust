# Contributing to paubox

Thank you for your interest in contributing!

By submitting a contribution, you agree that it is licensed under the project's [Apache 2.0 license](LICENSE).

## Getting started

```bash
git clone https://github.com/Paubox/paubox-rust
cd paubox-rust
rustup update stable
cargo build --all-features
```

MSRV is **Rust 1.86**. No external services are needed to run the test suite — all tests use mocked HTTP via `wiremock`.

## Running tests

```bash
# All unit + integration mock tests
cargo test --all-features

# A single test by name
cargo test send_message_returns_tracking_id

# Live integration tests (requires credentials; off by default)
cargo test --all-features -- --ignored
```

There are **no live API calls** in the default test suite.

## Linting and formatting

```bash
# Format check (CI enforces this)
cargo fmt --check

# Lints must pass with zero warnings
cargo clippy --all-features -- -D warnings
```

## Feature-flag combinations to verify

```bash
cargo check                                        # default (email + forms)
cargo check --no-default-features --features email
cargo check --no-default-features --features forms
# At least one feature is required; a bare --no-default-features build is
# intentionally rejected by a compile_error in src/lib.rs.
```

## Pull request expectations

- All tests pass: `cargo test --all-features`
- Formatting is clean: `cargo fmt --check`
- No clippy warnings: `cargo clippy --all-features -- -D warnings`
- New endpoints require tests covering: happy path, 401, 404/400, malformed JSON, and builder validation
- Public API additions require doc comments (`///` on items, `//!` on modules) and an entry in `CHANGELOG.md`
- No `unwrap()` / `expect()` in library code (acceptable in examples and tests)
- No path dependencies in `Cargo.toml`

See [CLAUDE.md](CLAUDE.md) for the full repository conventions.

## Reporting security issues

Do **not** open a public issue for security vulnerabilities. Email security@paubox.com instead. See [SECURITY.md](SECURITY.md).
