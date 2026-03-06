# Releasing

## How It Works

Releases are fully automated via CI. When commits are pushed to `main`, the [release workflow](.github/workflows/release.yml) runs:

1. **Check** — Scans commits since the last `v*` tag for conventional commit types
2. **Prepare** — Bumps versions in `Cargo.toml` and `package.json`, generates changelog, commits, and tags
3. **Release** — Creates a GitHub Release with generated notes
4. **Build** — Builds native `.node` binaries for all platforms (Linux x64, macOS x64/arm64, Windows x64, Linux arm64)
5. **Publish** — Publishes platform packages and the main `langmail` package to npm with provenance

No manual steps are needed. Just merge to `main`.

## Conventional Commits

Commit messages determine whether a release happens and what version bump to apply:

| Prefix | Bump | Example |
|--------|------|---------|
| `feat:` | minor | `feat: add attachment extraction` |
| `fix:` | patch | `fix: handle empty subject header` |
| `perf:` | patch | `perf: reduce HTML parsing allocations` |
| `refactor:` | patch | `refactor: simplify quote detection` |
| `BREAKING CHANGE` or `!:` | major | `feat!: redesign ProcessedEmail output` |

Other types (`chore:`, `docs:`, `test:`, `ci:`, `build:`) do **not** trigger a release.

## Changelog

Only `feat`, `fix`, `perf`, and `refactor` commits appear in `CHANGELOG.md`. Breaking changes from any commit type are always included (`protect_breaking_commits = true` in `cliff.toml`).

## Required Secrets

| Secret | Purpose |
|--------|---------|
| `NPM_TOKEN` | npm publish token with publish access to `langmail` and platform packages |
| `GITHUB_TOKEN` | Automatically provided; used for GitHub Release creation |
| `RELEASE_TOKEN` | (Optional) PAT with push access — only needed if branch protection rules block the bot from pushing to `main` |

## Manual Release Fallback

If CI fails partway through, you can recover manually:

```bash
# 1. Bump versions
sed -i 's/version = ".*"/version = "X.Y.Z"/' Cargo.toml
# Also update packages/langmail/package.json version + optionalDependencies

# 2. Generate changelog
git-cliff --tag vX.Y.Z -o CHANGELOG.md

# 3. Commit and tag
git add -A
git commit -m "chore: release vX.Y.Z"
git tag vX.Y.Z
git push origin main --follow-tags
```

The `publish.yml` workflow will **not** run (it has been removed). If you need to publish manually, push the tag and trigger the release workflow, or publish npm packages locally with `npx napi` commands.

## Pre-release / Alpha Versions

Automated releases don't support pre-release versions. For alpha/beta releases, bump versions manually:

```bash
# Edit Cargo.toml: version = "0.4.0-alpha.1"
# Edit packages/langmail/package.json similarly
git add -A
git commit -m "chore: release v0.4.0-alpha.1"
git tag v0.4.0-alpha.1
git push origin main --follow-tags
```

Then publish npm packages with the appropriate dist-tag:

```bash
npm publish --tag alpha
```
