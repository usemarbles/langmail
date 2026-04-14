# Contributing to langmail

Thanks for your interest in contributing! This guide covers the dev setup and the checks you should run before opening a PR.

## Repository layout

- `crates/langmail/` — the core Rust library (MIME parsing, quote/signature stripping, HTML → Markdown, CTA extraction)
- `crates/langmail-node/` — Node.js bindings via [napi-rs](https://napi.rs/)
- `crates/langmail-python/` — Python bindings via [PyO3](https://pyo3.rs/) / [maturin](https://www.maturin.rs/)
- `packages/langmail/` — the published npm package (contains the committed, generated `index.js` + `index.d.ts`)
- `docs/` — [Zensical](https://zensical.org/) documentation source, deployed to [langmail.dev](https://langmail.dev)

## Dev setup

```bash
git clone https://github.com/usemarbles/langmail.git
cd langmail
cargo build --workspace
```

To work on the **Node bindings**, also:

```bash
cd packages/langmail
npm install
npm run build:debug
```

To work on the **Python bindings**, also:

```bash
cd crates/langmail-python
python -m venv .venv && source .venv/bin/activate
pip install maturin pytest
maturin develop
```

## Before committing

Run the same checks CI runs:

```bash
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
```

If you changed the Node bindings, also regenerate and stage the committed JS/TS artifacts:

```bash
cd packages/langmail && npm run build:debug
git add index.js index.d.ts
```

If you changed the Python bindings, re-run the Python tests:

```bash
pytest crates/langmail-python/tests/ -v
```

## Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/) — the changelog is grouped by prefix automatically:

- `feat:` — new feature
- `fix:` — bug fix
- `refactor:` — internal change, no behavior change
- `docs:` — documentation only
- `test:` — tests only
- `chore:` — tooling, release, dependencies

## Documentation

```bash
make docs        # Build the site into ./site/
make serve-docs  # Serve locally with live reload
```

Docs deploy from `main` via [`.github/workflows/docs.yml`](./.github/workflows/docs.yml).
