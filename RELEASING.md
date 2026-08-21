# Releasing `paubox`

Releases are automated. [release-please](https://github.com/googleapis/release-please)
decides the version, writes the changelog, and tags; the tag is published to
[crates.io](https://crates.io/crates/paubox) by the same workflow via
[Trusted Publishing](https://crates.io/docs/trusted-publishing) (GitHub OIDC),
so no registry token is stored in the repo.

Nobody edits `version` in `Cargo.toml` by hand.

## How a release happens

1. **Merge PRs to `main` with conventional-commit titles.** The repo squash-merges,
   so the PR title becomes the commit subject and is the only thing release-please
   reads. `.github/workflows/pr-title.yml` rejects titles it cannot parse.

   | Title prefix | Effect on the next version |
   |---|---|
   | `fix:` | patch |
   | `feat:` | minor |
   | `feat!:` / any type with `!`, or a `BREAKING CHANGE:` footer | major |
   | `docs:`, `chore:`, `ci:`, `refactor:`, `test:`, `style:`, `build:` | none |

2. **release-please keeps a release PR open** titled `chore(main): release X.Y.Z`.
   It bumps `Cargo.toml` and `Cargo.lock` and adds the `CHANGELOG.md` section.
   It rewrites itself on every push to `main`, so leave it alone until you want
   to ship.

3. **Merge the release PR.** That creates the `vX.Y.Z` tag and the GitHub
   Release, which in turn runs the `publish` job: `cargo fmt --check`,
   `cargo test --all-features`, `cargo publish --dry-run`, then `cargo publish`
   against an OIDC-minted token.

## Forcing a specific version

To release a version release-please would not have chosen, put a `Release-As`
footer in a commit on `main`:

```sh
git commit --allow-empty -m "chore: release 2.0.0" -m "Release-As: 2.0.0"
```

The next release PR is pinned to that version.

## Configuration

| File | Purpose |
|---|---|
| `release-please-config.json` | `release-type: rust`, bare `vX.Y.Z` tags |
| `.release-please-manifest.json` | the last released version — **seeded from crates.io, not from `Cargo.toml`** |
| `.github/workflows/release-please.yml` | the release PR and the publish job |

The publish job lives in `release-please.yml` rather than in a workflow
triggered by `push: tags`, because release-please creates the tag with
`GITHUB_TOKEN` and GitHub does not start workflow runs from `GITHUB_TOKEN`
events — a tag-triggered publish would never fire.

### Trusted publishing

Registered on crates.io → `paubox` → Settings → Trusted Publishing:

- Repository owner: `Paubox`
- Repository name: `paubox-rust`
- Workflow filename: `release-please.yml`
- Environment: `release`

The `release` environment must allow `main` as a deployment branch, since the
workflow run's ref is `refs/heads/main` even though the publish job checks out
the tag. Add required reviewers there if you want a human gate on publishes.

## Caveats

crates.io versions are **immutable** — you cannot overwrite or re-upload one. To
retract a broken release use `cargo yank --version X.Y.Z`; that stops new
dependents from selecting it but does not delete it. The fix for a bad release
is always a new version.
