# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

No public API changes — every exported item, signature, and JSON payload is
unchanged.

### Fixed
- `cargo test` / `cargo check --all-targets` now succeed under
  `--no-default-features --features email` and
  `--no-default-features --features forms`. Examples and integration tests
  declare `required-features`, and the crate-level doctests are feature-gated,
  so targets belonging to a disabled API are skipped rather than failing to
  compile.
- CI's feature-combination job runs `cargo test --all-targets` and a
  per-feature doctest pass instead of a library-only `cargo check`, which had
  been hiding the above.

### Added
- Mock tests for `PauboxClient::api_status` covering 200, 401, and 500.

### Changed
- Internal only: the Email API's non-success response mapping is shared
  between `api_status` and `handle_response`; the Forms base URL is defined
  once instead of through a `pub(crate)` alias.

### Documentation
- Corrected the CI MSRV noted for 0.1.0 (1.86, not 1.75).
- `FormSubmissionBuilder::form_data` and `FormsClient::submit_form` no longer
  claim an empty `form_data` is rejected locally; only an unset value and JSON
  `null` are. An empty object is still sent and the API answers 400.
- api.md no longer claims `FormsClient` is re-exported at the crate root.

## [0.1.0] - 2026-05-28

Initial public release.

### Added
- Async Rust SDK for the Paubox Email API and Forms API
- `PauboxClient` with constructor, `from_env`, and a builder (`api_key`, `api_user`, `timeout`)
- **Email**: `send_message`, `get_email_disposition`
- `Message` builder with `from`, `to`, `subject`, `text_content`, and attachment support
- `Attachment::from_bytes` for base64-encoded attachments
- **Forms**: `FormsClient` with `get_form` and `submit_form` (public endpoints, no API key required)
- `FormSubmission` builder
- `PauboxError` with variants for auth, HTTP, and (de)serialization failures
- Cargo feature flags `email` and `forms` (both enabled by default)
- `wiremock`-based mock test suite — no live API calls required
- Examples: `send_email`, `check_disposition`, `get_form`, `submit_form`
- GitHub Actions CI: fmt, clippy, test, MSRV (1.86), and feature-flag matrix
- `LICENSE` (Apache 2.0), `NOTICE`, `SECURITY.md`, and `CONTRIBUTING.md`

### Requirements
- Requires Rust 1.86 or later.

[Unreleased]: https://github.com/Paubox/paubox-rust/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Paubox/paubox-rust/releases/tag/v0.1.0
