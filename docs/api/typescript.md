---
title: TypeScript / Node.js API
description: Full API surface for the Node.js langmail package.
---

```ts
import {
  preprocess,
  preprocessString,
  preprocessWithOptions,
  preprocessGmail,
  toLlmContext,
  toLlmContextWithOptions,
  RenderMode,
} from "langmail"
import type {
  ProcessedEmail,
  PreprocessOptions,
  LlmContextOptions,
  Address,
  CallToAction,
  ThreadMessage,
} from "langmail"
```

!!! note
    The public TypeScript surface also re-exports the underlying NAPI-RS
    generated names (`NapiAddress`, `NapiCallToAction`, `NapiLlmContextOptions`,
    `NapiRenderMode`, `NapiThreadMessage`) as backward-compatible aliases.
    Prefer the unprefixed names — they are the canonical public API.

## preprocess()

Accepts raw RFC 5322 email bytes and returns a structured `ProcessedEmail` object. Handles MIME multipart messages, HTML and plain-text body variants, and normalises character encodings.

```ts
function preprocess(raw: Buffer): ProcessedEmail
function preprocessString(raw: string): ProcessedEmail
function preprocessWithOptions(
  raw: Buffer,
  options: PreprocessOptions
): ProcessedEmail
```

`preprocess` is **synchronous** and takes a `Buffer`. Use `preprocessString` as a convenience wrapper if you already have the email as a string. Use `preprocessWithOptions` to override defaults — see [`PreprocessOptions`](#preprocessoptions).

### ProcessedEmail

Optional fields are declared with `?:` in the generated `.d.ts` — their type
is `T | undefined`, not `T | null`.

| Field | Type | Description |
| --- | --- | --- |
| body | string | Cleaned body text, with quotes and signature removed |
| subject | string \| undefined | Subject line |
| from | Address \| undefined | Sender |
| to | Address[] | To recipients |
| cc | Address[] | Cc recipients |
| date | string \| undefined | ISO 8601 date string |
| rfcMessageId | string \| undefined | RFC 2822 Message-ID header value |
| inReplyTo | string[] \| undefined | In-Reply-To header values (for threading) |
| references | string[] \| undefined | References header values (for threading) |
| signature | string \| undefined | Extracted signature, if found |
| rawBodyLength | number | Length of the original body before cleaning |
| cleanBodyLength | number | Length of the cleaned body |
| primaryCta | CallToAction \| undefined | Primary call-to-action link extracted from the HTML body |
| threadMessages | ThreadMessage[] | Quoted reply messages, oldest first |
| isNewsletter | boolean | `true` if the email was identified as a newsletter or bulk marketing message (header-based; platform notifications are excluded) |

`Address` is `{ name?: string, email: string }`. `CallToAction` is `{ url: string, text: string, confidence: number }`. `ThreadMessage` is `{ sender: string, timestamp?: string, body: string }`.

### PreprocessOptions

| Option | Type | Default | Description |
| --- | --- | --- | --- |
| stripQuotes | boolean | `true` | Remove quoted reply chains |
| stripSignature | boolean | `true` | Remove trailing signature block |
| maxBodyLength | number | `0` | Truncate body after N characters. `0` = no limit |

## preprocessGmail()

Provider adapter for the Gmail API. Accepts the response of `gmail.users.messages.get({ id, format: "full" })` from `googleapis` and returns the same `ProcessedEmail` shape as `preprocess()`. Skips MIME re-parsing — the Gmail API has already decomposed the message into typed parts, so the adapter walks `payload.parts`, base64url-decodes the bodies, and feeds them into the shared cleaning pipeline. The body tree walk, header parsing, and base64url decoding all happen in Rust — this wrapper only serializes the caller's object to JSON and delegates to the native binding, so the output is byte-identical to the Python and Rust entry points.

```ts
function preprocessGmail(
  msg: GmailInput,
  options?: PreprocessOptions
): ProcessedEmail
```

Accepts either the bare `Schema$Message` or the full googleapis response (`{ data: Schema$Message, ... }`). The message must have been fetched with `format: "full"` so `payload` is present with headers and base64url-encoded body parts.

**Body selection:** walks `payload.parts` depth-first and picks the first non-attachment leaf of each type. When both `text/html` and `text/plain` are present, HTML wins. Parts with `Content-Disposition: attachment` or a `filename` are skipped.

**Throws:**

- `TypeError` if the input is not an object or has no `payload` (i.e. the message wasn't fetched with `format: "full"`).
- `Error` if the chosen body part is attachment-backed (Gmail returned `body.attachmentId` instead of `body.data` because the body exceeded the inline size threshold — fetch with `users.messages.attachments.get` and inline the decoded content).

!!! note
    langmail does not bundle or depend on `googleapis` — only the shape of the response is consumed. Bodies are decoded as UTF-8; per-part `charset` parameters are not consulted, so legacy 8-bit encodings may produce mojibake.

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
| renderMode | `RenderMode.LatestOnly` \| `RenderMode.ThreadHistory` | `LatestOnly` | `LatestOnly` strips quoted content; `ThreadHistory` appends quoted replies as a chronological transcript below the main content |

!!! warning
    Quote detection is heuristic. See [Concepts → Caveats](../concepts.md#caveats) for where accuracy degrades.
