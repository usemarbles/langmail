---
title: Core Concepts
description: langmail exposes two functions. Everything else is implementation detail.
---

## preprocess()

Accepts raw RFC 5322 email bytes and returns a structured `ProcessedEmail` object. Handles MIME multipart messages, HTML and plain-text body variants, and normalises character encodings.

```ts
function preprocess(raw: Buffer): ProcessedEmail
function preprocessString(raw: string): ProcessedEmail
```

`preprocess` is **synchronous** and takes a `Buffer`. Use `preprocessString` as a convenience wrapper if you already have the email as a string.

What it does internally:

- Parses the MIME structure and selects the most relevant body variant (HTML preferred over plain text)
- Converts HTML to Markdown, preserving semantic structure
- Detects and removes quoted reply chains across Gmail, Outlook, Apple Mail, and non-standard clients
- Detects and removes email signatures
- Normalises zero-width characters and HTML entities
- Strips URLs while preserving anchor text

### ProcessedEmail type

| Field | Type | Description |
| --- | --- | --- |
| body | string | Cleaned body text, with quotes and signature removed |
| subject | string \| null | Subject line |
| from | Address \| null | Sender |
| to | Address[] | To recipients |
| cc | Address[] | Cc recipients |
| date | string \| null | ISO 8601 date string |
| rfcMessageId | string \| null | RFC 2822 Message-ID header value |
| inReplyTo | string[] \| null | In-Reply-To header values (for threading) |
| references | string[] \| null | References header values (for threading) |
| signature | string \| null | Extracted signature, if found |
| rawBodyLength | number | Length of the original body before cleaning |
| cleanBodyLength | number | Length of the cleaned body |
| primaryCta | CallToAction \| null | Primary call-to-action link extracted from the HTML body |
| threadMessages | ThreadMessage[] | Quoted reply messages, oldest first |

`Address` is `{ name?: string, email: string }`. `CallToAction` is `{ url: string, text: string, confidence: number }`. `ThreadMessage` is `{ sender: string, timestamp?: string, body: string }`.

## toLlmContext()

Accepts a `ProcessedEmail` and returns a deterministic plain-text string formatted for direct inclusion in a prompt. The output includes a header block (FROM / TO / SUBJECT / DATE) followed by a `CONTENT:` section.

```ts
function toLlmContext(email: ProcessedEmail): string
function toLlmContextWithOptions(
  email: ProcessedEmail,
  options: LlmContextOptions
): string
```

Use `toLlmContextWithOptions` when you need to control rendering — for example, to include quoted reply history.

### LlmContextOptions

| Option | Type | Default | Description |
| --- | --- | --- | --- |
| renderMode | "LatestOnly" \| "ThreadHistory" | "LatestOnly" | `LatestOnly` strips quoted content; `ThreadHistory` appends quoted replies as a chronological transcript below the main content |

!!! warning
    Quoted reply detection is heuristic-based. Accuracy varies across non-standard email clients. If precision matters for your use case, review the output before passing to production models.
