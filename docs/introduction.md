---
title: Introduction
description: langmail is an open-source library for converting raw emails into clean, structured input for language models.
---

langmail is an open-source library for converting raw emails into clean, structured input for language models.

## What it does

Raw emails are noisy. They contain quoted reply chains, signatures, tracking pixels, malformed HTML, zero-width characters, and platform-specific formatting artifacts. Feeding this noise to an LLM wastes tokens and degrades output quality.

langmail solves this with a two-step pipeline. `preprocess()` parses the raw email and strips everything irrelevant. `toLLMContext()` serializes the result into clean Markdown — a format LLMs process more reliably than plain text.

## Why Markdown

Markdown preserves semantic structure while remaining far cleaner than raw HTML. A bold subject line, a bullet list of action items, a link with meaningful anchor text — these carry meaning that plain text loses. LLMs trained on large corpora have strong priors for Markdown, which improves downstream task performance on summarisation, CTA extraction, and classification.

## What it is not

langmail does not send emails, receive emails, or integrate with any email provider. It operates entirely on raw RFC 5322 strings. Provider integration is out of scope by design — keeping the library agnostic means it works with any source: IMAP, Gmail API, SES, Postmark, or a local `.eml` file.

## Implementation

The core library is written in Rust for performance and cross-platform compilation. Bindings are available for:

- **Node.js** — via napi-rs, published to npm as `langmail`
- **Python** — via PyO3, published to PyPI as `langmail`
- **Rust** — published to crates.io as `langmail`

!!! tip "Open Source"
    langmail is MIT licensed. Source code is on [GitHub](https://github.com/usemarbles/langmail). Contributions welcome.
