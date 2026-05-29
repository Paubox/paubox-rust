# Releasing `paubox`

Releases are published to [crates.io](https://crates.io/crates/paubox). The crate
is owned by the `Paubox/engineering` GitHub team, and every release **after the
first** is published automatically by CI via crates.io
[Trusted Publishing](https://crates.io/docs/trusted-publishing) (GitHub OIDC) —
no API tokens are stored.

## One-time setup

crates.io requires the **first** version to be published manually with a token;
trusted publishing can only be configured once the crate exists.

1. On crates.io (signed in with a GitHub account that is a member of
   `Paubox/engineering`): verify your email, then create an API token with the
   `publish-new` scope (Account Settings → API Tokens).
2. From a clean `main` checkout:
   ```sh
   cargo login <token>
   cargo publish            # reserves the name and publishes the first version
   ```
3. Transfer ownership to the org and drop the personal owner:
   ```sh
   cargo owner --add github:Paubox:engineering
   cargo owner --remove <your-crates-io-username>
   ```
4. crates.io → `paubox` crate → **Settings → Trusted Publishing → Add**:
   - Repository owner: `Paubox`
   - Repository name: `paubox-rust`
   - Workflow filename: `release.yml`
   - Environment: `release`
5. In GitHub: repo **Settings → Environments → New environment → `release`**
   (optionally add required reviewers to gate publishes).
6. Revoke the one-time token created in step 1 — it is no longer needed.

## Cutting a release (every time after setup)

1. Update `version` in `Cargo.toml` (SemVer; `0.x` → breaking = minor bump,
   fix = patch bump).
2. Move `CHANGELOG.md`'s `[Unreleased]` items into a new dated section and
   update the compare links.
3. Verify locally:
   ```sh
   cargo fmt --all --check
   cargo clippy --all-features --all-targets -- -D warnings
   cargo test --all-features
   cargo publish --dry-run
   ```
4. Commit, open a PR, and merge to `main`.
5. Tag and push:
   ```sh
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```
   The `release.yml` workflow verifies and publishes to crates.io via OIDC.
6. Create a GitHub Release from the tag (paste the CHANGELOG section).

crates.io versions are **immutable** — you cannot overwrite or re-upload a
version. To retract a broken release use `cargo yank --version X.Y.Z` (this
prevents new dependents from selecting it but does not delete it).
