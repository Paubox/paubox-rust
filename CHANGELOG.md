# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Forms**: scoped API key authentication — `FormsClient::with_api_key` and
  `FormsClient::builder()` (`api_key`, `base_url`, `timeout`). Keys are sent as
  `Authorization: Bearer` and must carry the `forms` scope
- **Forms**: authenticated form management — `list_forms` (pagination, search,
  archived/active filters, sorting), `create_form`, `get_form_by_id`,
  `update_form` (PATCH-style partial update), `archive_form`, `unarchive_form`,
  `copy_form`, and `form_stats`
- **Forms**: authenticated submission access — `list_submissions`,
  `export_submissions_csv`, `export_submission_csv`, and `export_submission_pdf`
- New `forms::admin` types: `CreateForm` (+ builder), `UpdateForm`,
  `FormListParams`, `SubmissionListParams`, `FormPage`, `PageInfo`, `FormStats`,
  `Submission` (with `form_data_json()` helper), and `SubmissionPage`
- Examples: `list_forms`, `create_form`, `export_submissions`

### Changed
- `forms::Form` gained the newer server fields (`version`, `vanity_url`,
  `recipient`, `signature_confirmation_label`, `type_`, `subscription_list_id`,
  `archived`, `deleted`, `old_form_id`) — all serde-defaulted, so existing
  deserialization keeps working
- Forms endpoints now map HTTP 401 to `PauboxError::Auth` (previously
  `PauboxError::Http { status: 401, .. }`); protected methods fail fast with
  `PauboxError::Auth` before any network call when no API key is configured

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
