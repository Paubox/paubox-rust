# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- Email and Forms requests dropped the trailing base-path segment, sending
  requests to `…/v1/messages` and `…/public/form_data/{id}` (404) instead of
  `…/v1/{api_user}/messages` and `…/forms/public/form_data/{id}`. Base URLs
  are now normalised with a trailing slash so `Url::join` preserves them.
- `send_message` failed to parse the success response: the confirmation
  string is returned by the API in the `data` field, not `message`.
- `get_email_disposition` failed to parse responses for freshly-sent
  messages because `deliveryTime` / `openedTime` may be absent; these now
  default to `None`.

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
- GitHub Actions CI: fmt, clippy, test, MSRV (1.75), and feature-flag matrix
- `LICENSE` (Apache 2.0), `NOTICE`, `SECURITY.md`, and `CONTRIBUTING.md`

### Requirements
- Requires Rust 1.86 or later.

[Unreleased]: https://github.com/Paubox/paubox-rust/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Paubox/paubox-rust/releases/tag/v0.1.0
