---
title: Python API
description: Full API surface for the langmail Python package.
---

```python
from langmail import (
    preprocess,
    preprocess_string,
    preprocess_with_options,
    preprocess_gmail,
    to_llm_context,
    to_llm_context_with_options,
    ProcessedEmail,
    PreprocessOptions,
    LlmContextOptions,
    RenderMode,
    ParseError,
)
```

## preprocess()

Accepts raw RFC 5322 email bytes and returns a structured `ProcessedEmail` object.

```python
def preprocess(raw: bytes) -> ProcessedEmail
def preprocess_string(raw: str) -> ProcessedEmail
def preprocess_with_options(
    raw: bytes,
    options: PreprocessOptions,
) -> ProcessedEmail
```

Open `.eml` files in binary mode (`"rb"`) so you get `bytes`. Use `preprocess_string` if you already have a `str`. Use `preprocess_with_options` to override defaults — see [`PreprocessOptions`](#preprocessoptions).

!!! note
    Prefer `preprocess(bytes)` for legacy non-UTF-8 sources. `preprocess_string` accepts a `str`, so the caller has already made a decoding decision — any encoding loss happens before langmail sees the input.

Raises `langmail.ParseError` if the input can't be parsed as an email.

### ProcessedEmail

| Attribute | Type | Description |
| --- | --- | --- |
| body | str | Cleaned body text, with quotes and signature removed |
| subject | str \| None | Subject line |
| from_address | Address \| None | Sender (renamed from `from` — reserved keyword in Python) |
| to | list[Address] | To recipients |
| cc | list[Address] | Cc recipients |
| date | str \| None | ISO 8601 date string |
| rfc_message_id | str \| None | RFC 2822 Message-ID header value |
| in_reply_to | list[str] \| None | In-Reply-To header values (for threading) |
| references | list[str] \| None | References header values (for threading) |
| signature | str \| None | Extracted signature, if found |
| raw_body_length | int | Length of the original body before cleaning |
| clean_body_length | int | Length of the cleaned body |
| primary_cta | CallToAction \| None | Primary call-to-action link extracted from the HTML body |
| thread_messages | list[ThreadMessage] | Quoted reply messages, oldest first |

`Address` has `name: str | None` and `email: str`. `CallToAction` has `url: str`, `text: str`, `confidence: float`. `ThreadMessage` has `sender: str`, `timestamp: str | None`, `body: str`.

### PreprocessOptions

```python
PreprocessOptions(
    strip_quotes: bool = True,
    strip_signature: bool = True,
    max_body_length: int = 0,
)
```

| Option | Default | Description |
| --- | --- | --- |
| strip_quotes | `True` | Remove quoted reply chains |
| strip_signature | `True` | Remove trailing signature block |
| max_body_length | `0` | Truncate body after N characters. `0` = no limit |

## preprocess_gmail()

Provider adapter for the Gmail API. Accepts a JSON-serialized Gmail
`users.messages.get` response (`format='full'`) and returns the same
`ProcessedEmail` shape as `preprocess()`. The body tree walk, base64url
decoding, and header parsing all happen in Rust — shared with the Node
binding.

```python
def preprocess_gmail(
    msg_json: str,
    options: PreprocessOptions | None = None,
) -> ProcessedEmail
```

Pass the message as a JSON string (typically via `json.dumps(msg)`). Accepts
either the bare `Schema$Message` or the full googleapis-style wrapper
(`{"data": ..., "status": 200}`). The message must have been fetched with
`format='full'` so `payload` is present with headers and base64url-encoded
body parts.

```python
import json
from googleapiclient.discovery import build
from langmail import preprocess_gmail, to_llm_context

gmail = build("gmail", "v1", credentials=creds)
msg = gmail.users().messages().get(
    userId="me", id=message_id, format="full"
).execute()

email   = preprocess_gmail(json.dumps(msg))
context = to_llm_context(email)
```

**Body selection:** walks `payload.parts` depth-first and picks the first
non-attachment leaf of each type. When both `text/html` and `text/plain`
are present, HTML wins. Parts with `Content-Disposition: attachment` or a
`filename` are skipped.

**Raises `ParseError`** if:

- the input is not valid JSON,
- the input is not a JSON object (e.g. `json.dumps(42)`, `json.dumps(None)`),
- `payload` is missing (fetch with `format='full'`), or
- the chosen body part is attachment-backed (Gmail returned
  `body.attachmentId` because the body exceeded the inline size
  threshold — fetch it with `users.messages.attachments.get` and inline
  the decoded content).

!!! note
    langmail does not bundle a Gmail client — only the shape of the
    response is consumed. Install `google-api-python-client` (or any
    client that returns the same `Schema$Message`) separately.

## to_llm_context()

```python
def to_llm_context(email: ProcessedEmail) -> str
def to_llm_context_with_options(
    email: ProcessedEmail,
    options: LlmContextOptions,
) -> str
```

### LlmContextOptions

```python
LlmContextOptions(render_mode: RenderMode = RenderMode.LatestOnly)
```

`RenderMode` is an enum with values `LatestOnly` and `ThreadHistory`. See [Concepts → Rendering modes](../concepts.md#rendering-modes).

## Errors

`langmail.ParseError` (subclass of `ValueError`) is raised when the input can't be parsed as an RFC 5322 message, or when `preprocess_gmail` is given a malformed Gmail message (invalid JSON, missing `payload`, or an attachment-backed body that needs a separate fetch).
