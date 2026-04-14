## Project Overview

**langmail** is a Rust library with Node.js and Python bindings for preprocessing email content for LLM consumption. It strips HTML noise, quoted replies, signatures, and excessive whitespace while preserving structured metadata (headers, threading info, etc.).

## Architecture
- As much as possible all business logic should be implemented in Rust. Node and Python are only thin adapter layers.

## Documentation
- Always make sure that the `/docs` and examples in `README.md` are in sync with the code.

## Implementation
- After code changes always
  1. Run the `test`, `format` and `clippy` scripts.
  2. Fix any issues that arise.
  3. Spawn a team of expert subagents to review this work for multiple angles.
  4. The main agent then analyze the review findings for validity and decides which ones to fix.
  5. If fixes were made in step 4, repeat steps 1–2 once to verify — do not repeat the review.
  6. Once all tests pass, commit the changes to git using conventional commits. Do not commit prompt changes automatically.
  7. Then push and create a PR if none exists yet.

## Documentation
- Lives in `/docs`, built with [Zensical](https://zensical.org) (config at `zensical.toml`).
- Build: `make docs` (output → `site/`). Serve locally: `make serve-docs`.
- Deployed to `langmail.dev` via `.github/workflows/docs.yml` on push to `main`.
