---
title: Core Concepts
description: langmail exposes two functions. Everything else is implementation detail.
---

## preprocess()

Accepts a raw RFC 5322 email string and returns a structured `ParsedEmail` object. Handles MIME multipart messages, HTML and plain-text body variants, and normalises all character encodings including zero-width characters and HTML entities.

```ts
function preprocess(
  raw: string
): Promise<ParsedEmail>
```

What it does internally:

- Parses the MIME structure and selects the most relevant body variant (HTML preferred over plain text)
- Converts HTML to Markdown, preserving semantic structure
- Detects and removes quoted reply chains across Gmail, Outlook, Apple Mail, and non-standard clients
- Detects and removes email signatures
- Normalises zero-width characters and HTML entities
- Strips URLs while preserving anchor text

### ParsedEmail type

| Field | Type | Description |
| --- | --- | --- |
| subject | string \| null | Decoded subject line |
| sender | Address | From field, parsed into name and email |
| recipients | Address[] | To, Cc recipients |
| date | string \| null | ISO 8601 date string |
| body | string | Cleaned Markdown body |
| ctas | CTA[] | Extracted calls-to-action |
| rfc_message_id | string \| null | RFC 5322 Message-ID header |

## toLLMContext()

Accepts a `ParsedEmail` and returns a Markdown string formatted for direct inclusion in a prompt. The output includes a structured header block followed by the cleaned body.

```ts
function toLLMContext(
  parsed: ParsedEmail,
  options?: LLMContextOptions
): string
```

### LLMContextOptions

| Option | Type | Default | Description |
| --- | --- | --- | --- |
| format | "markdown" \| "plain" | "markdown" | Output format |
| includeHeaders | boolean | true | Prepend From / Subject / Date header block |
| includeCTAs | boolean | true | Append extracted CTAs at the end |

!!! warning
    Quoted reply detection is heuristic-based. Accuracy varies across non-standard email clients. If precision matters for your use case, review the output before passing to production models.
