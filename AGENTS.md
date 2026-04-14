## Project Overview

**langmail** is a Rust library with Node.js and Python bindings for preprocessing email content for LLM consumption. It strips HTML noise, quoted replies, signatures, and excessive whitespace while preserving structured metadata (headers, threading info, etc.).

## Implementation
- After code changes always run the `test`, `format` and `clippy` scripts. Fix any issues that arise.
- Use conventional commits (`feat:`, `fix:`, `refactor:`, etc.) so the changelog groups entries automatically.
- Always make sure that the `/docs` and examples in `README.md` are in sync with the code.

## Documentation
- Lives in `/docs`, built with [Zensical](https://zensical.org) (config at `zensical.toml`).
- Build: `make docs` (output → `site/`). Serve locally: `make serve-docs`.
- Deployed to `langmail.dev` via `.github/workflows/docs.yml` on push to `main`.
