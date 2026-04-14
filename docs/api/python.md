---
title: Python API
description: Full API surface for the langmail Python package.
---

**Language:** [TypeScript](typescript.md) · Python · [Rust](rust.md) — see also [Concepts](../concepts.md)

```python
from langmail import (
    preprocess,
    preprocess_string,
    preprocess_with_options,
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

`langmail.ParseError` (subclass of `ValueError`) is raised when the input can't be parsed as an RFC 5322 message.

!!! note
    There is no Python equivalent of `preprocessGmail`. Provider adapters currently live only in the Node.js package — open an issue if you need one for Python.
