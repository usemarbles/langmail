# Spec 01 — Provider adapters

**Status:** draft
**Priority:** P0 (highest-leverage single change)

## Problem

`preprocess(raw: Buffer)` only accepts RFC 5322 bytes. Most Node.js email code in production never touches `.eml`; it consumes already-parsed JSON from a provider (Gmail API, Microsoft Graph, Postmark, SendGrid, Resend, Nylas, Front, IMAP libraries).

Adopting langmail today forces a user to:

1. Change their Gmail call from `format: "full"` to `format: "raw"` (roughly doubles payload size).
2. Re-parse MIME that Google already parsed.
3. Re-derive headers/labels they already had as structured fields.

All three are negative-value trades on axes unrelated to langmail's real value-add (quote stripping, signature removal, HTML→Markdown, thread extraction). This friction is the single biggest blocker to "yes" in adoption evaluations.

## Goal

Let users run langmail's cleaning pipeline on the **output of whatever email API they already use**, without re-downloading or re-parsing MIME.

## Surface

New top-level entry points in `packages/langmail/index.js` (and mirrored in the Python binding):

```ts
// Gmail API (googleapis) — gmail_v1.Schema$Message
preprocessGmail(msg, options?): ProcessedEmail

// Microsoft Graph — Message resource
preprocessGraph(message, options?): ProcessedEmail

// Postmark inbound webhook payload
preprocessPostmark(payload, options?): ProcessedEmail

// SendGrid Inbound Parse payload
preprocessSendgrid(payload, options?): ProcessedEmail

// Generic escape hatch for anything pre-parsed
preprocessFromParsed(input: ParsedEmailInput, options?): ProcessedEmail

interface ParsedEmailInput {
  html?: string
  text?: string
  headers?: Record<string, string | string[]>
  subject?: string
  from?: string | Address
  to?: Array<string | Address>
  cc?: Array<string | Address>
  date?: string | Date
  messageId?: string
  inReplyTo?: string | string[]
  references?: string | string[]
}
```

All adapters return the same `ProcessedEmail` shape as `preprocess()`. Options are the existing `PreprocessOptions`.

## Scope

**In:**
- Gmail, Microsoft Graph, Postmark, SendGrid adapters (the top 4 by Node.js usage).
- Generic `preprocessFromParsed` for Resend, Nylas, Front, IMAP libs, custom pipelines.
- All adapters skip MIME parsing entirely; they feed already-decoded HTML/text into the existing cleaning pipeline.
- Each adapter is a thin TypeScript/JS wrapper around a single Rust entry point that accepts `ParsedEmailInput`-equivalent.

**Out:**
- No new fetchers. Users bring their own API client.
- No provider-specific auth.
- No support for attachments (unchanged from today).
- No runtime dependency on `googleapis`, `@microsoft/microsoft-graph-client`, etc. — only types (peer-optional).

## Implementation notes

- Add a new Rust function `preprocess_parsed(input: ParsedInput, options: Options) -> ProcessedEmail` in `crates/langmail` that bypasses the MIME parser and runs `clean_body`, `strip_quotes`, `strip_signature`, `html_to_markdown`, `extract_thread_messages` directly.
- NAPI binding exposes it as `preprocessParsed`.
- TypeScript adapters live in `packages/langmail/src/adapters/{gmail,graph,postmark,sendgrid}.ts` and call `preprocessParsed` after normalising the provider's shape. Pure JS, no extra native cost.
- Gmail adapter must recursively walk `payload.parts` and base64url-decode `text/plain` / `text/html` (same logic as today's callers), but that code now lives **inside** langmail, not in every user's codebase.
- Each adapter has a dedicated fixture folder under `crates/langmail/tests/fixtures/adapters/<provider>/` with real-shape sample payloads and a golden `ProcessedEmail` output.

## Success criteria

- A Gmail user replaces ~40 lines of `extractBody` + header-parsing boilerplate with `preprocessGmail(msg.data)` in ≤3 lines.
- No change to `format: "full"` fetch — langmail works on the existing Gmail payload.
- README shows one-line drop-in examples for all four adapters.
- Adapter paths are covered by snapshot tests against captured real-world payloads (redacted).

## Non-goals

- Streaming/async parsing.
- Anything that requires hitting the provider's API.
- Attachment handling.
