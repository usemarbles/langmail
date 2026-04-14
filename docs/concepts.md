---
title: Concepts
description: How langmail transforms raw email into LLM-ready context.
---

langmail exposes the same pipeline and the same output shape across every language binding. Types and function names differ per language — see the API reference pages for [TypeScript](api/typescript.md), [Python](api/python.md), or [Rust](api/rust.md) — but the semantics here apply universally.

## Pipeline

Raw email bytes in, a structured `ProcessedEmail` out. The pipeline runs these stages in order:

1. **MIME parse** — RFC 5322 structure and headers are extracted. Malformed bodies are recoverable; a completely unparseable input returns an error.
2. **Body selection** — if the message has both `text/html` and `text/plain` parts, HTML wins. Attachments are skipped.
3. **Invisible-character normalisation** — zero-width joiners, BOMs, and soft hyphens are stripped from the raw HTML before it is converted.
4. **HTML → Markdown** — HTML is converted to Markdown, preserving structure (headings, lists, links with anchor text) while dropping presentational noise.
5. **CTA extraction** — the primary call-to-action link is extracted from the raw HTML, via JSON-LD fast path when present, falling back to heuristic scoring otherwise. Runs against the pre-stripping HTML so the result is independent of quote/signature boundaries.
6. **Quote stripping** — reply chains from Gmail, Outlook, Apple Mail, and common ad-hoc markers (`On <date>, <sender> wrote:` etc.) are removed from the active body.
7. **Signature stripping** — trailing signatures (detected heuristically) are removed and surfaced as a separate `signature` field.
8. **Thread extraction** — quoted replies are extracted from HTML `<blockquote>` blocks into `threadMessages`, ordered oldest-first.

## ProcessedEmail

The structured output. Field names follow each language's conventions — `from` in TypeScript and Rust, `from_address` in Python (where `from` is a reserved keyword) — but the meaning is identical everywhere. Field names below use TypeScript/Rust camelCase; Python readers should mentally translate to snake_case (`rfcMessageId` → `rfc_message_id`, `primaryCta` → `primary_cta`, `threadMessages` → `thread_messages`).

| Field | Meaning | Empty when |
| --- | --- | --- |
| body | Cleaned body text, quotes and signature removed | source had no body |
| subject | Subject line | absent on the source message |
| from | Sender address | absent on the source message |
| to | Recipient addresses | no `To:` header present |
| cc | Carbon-copy addresses | no `Cc:` header present |
| date | ISO 8601 string | absent or unparseable |
| rfcMessageId | `Message-ID` header value | absent on the source message |
| inReplyTo | `In-Reply-To` header values (for threading) | absent on the source message |
| references | `References` header values (for threading) | absent on the source message |
| signature | Extracted signature block | no signature detected |
| rawBodyLength | Length of the body before cleaning | always present |
| cleanBodyLength | Length of the cleaned body | always present |
| primaryCta | Primary call-to-action link from HTML | no CTA scored above the threshold |
| threadMessages | Quoted reply messages, oldest first | no quotes detected |

An `Address` carries an optional display name and an email. A `CallToAction` has a URL, anchor text, and a confidence score in `[0.0, 1.0]`. A `ThreadMessage` has a sender attribution, optional ISO 8601 timestamp, and a cleaned body with no nested quotes.

## Rendering modes

`toLlmContext` (camelCase here; `to_llm_context` in Python and Rust) takes a `ProcessedEmail` and produces a deterministic plain-text prompt. Its rendering mode controls what happens to quoted reply history:

| Mode | Behaviour |
| --- | --- |
| `LatestOnly` (default) | Only the latest message is rendered. `threadMessages` is dropped from the output. |
| `ThreadHistory` | Latest message first, followed by `---` and a chronological transcript of `threadMessages`. |

## Caveats

- **Quote detection is heuristic.** Accuracy is high for Gmail, Outlook, and Apple Mail, and degrades on non-standard clients. If precision matters, inspect the output before passing to production models.
- **Bodies are decoded as UTF-8.** Per-part `charset` parameters are not consulted. Modern email is UTF-8 in practice, but legacy encodings will produce mojibake.
- **Attachments are skipped.** langmail is a text pipeline; binary parts are not extracted.
- **URLs are stripped from the body; anchor text is preserved.** The goal is LLM readability, not link fidelity. The one exception is `primaryCta`, which keeps its URL.
