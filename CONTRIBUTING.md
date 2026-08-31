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

- **The PR title must be a [conventional commit](https://www.conventionalcommits.org/)** — see below
- All tests pass: `cargo test --all-features`
- Formatting is clean: `cargo fmt --check`
- No clippy warnings: `cargo clippy --all-features -- -D warnings`
- New endpoints require tests covering: happy path, 401, 404/400, malformed JSON, and builder validation
- Public API additions require doc comments (`///` on items, `//!` on modules)
- No `unwrap()` / `expect()` in library code (acceptable in examples and tests)
- No path dependencies in `Cargo.toml`

## PR titles and the changelog

This repo squash-merges, so your PR title becomes the commit subject on `main`,
and that subject is what release-please turns into the next version and the
`CHANGELOG.md` entry. CI enforces the format.

```
fix: retry Forms submissions on 502
feat: add attachment support to FormSubmission
feat!: drop PauboxClientBuilder::api_user
docs: correct the send_email example
```

`fix:` gives a patch bump, `feat:` a minor one, and a `!` (or a `BREAKING CHANGE:`
footer in the body) a major one. `docs:`, `chore:`, `ci:`, `test:`, `style:`,
`refactor:`, and `build:` do not trigger a release.

CI checks the title against the code. `cargo-semver-checks` compares your branch's
public API to the released crate, and fails if you break it without a `!`. If that
check fails, either mark the PR breaking or make the change backwards compatible —
do not merge a breaking change under a `feat:`, because it would publish as a minor
release and break dependents' builds.

A common way to trip this is adding a field to a public struct. That breaks any
caller constructing it with a struct literal, unless the struct is
`#[non_exhaustive]`. Mark new response types — the ones callers only ever receive
— as `#[non_exhaustive]` from the start, so later field additions stay backwards
compatible. Note that adding the attribute to an *existing* type is itself a
breaking change, so it can only land in a major release.

**Do not edit `CHANGELOG.md` or the `version` in `Cargo.toml` in your PR.** Both
are generated — see [RELEASING.md](RELEASING.md).

See [CLAUDE.md](CLAUDE.md) for the full repository conventions.

## Reporting security issues

Do **not** open a public issue for security vulnerabilities. Email security@paubox.com instead. See [SECURITY.md](SECURITY.md).
