# langmail

**Email preprocessing for LLMs.** Fast, typed, Rust-powered.

[![PyPI](https://img.shields.io/pypi/v/langmail)](https://pypi.org/project/langmail/)
[![CI](https://github.com/usemarbles/langmail/actions/workflows/ci.yml/badge.svg)](https://github.com/usemarbles/langmail/actions/workflows/ci.yml)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)

Emails are messy — nested MIME parts, quoted reply chains, HTML cruft, signatures, forwarded headers. LLMs don't need any of that. langmail strips it all away and gives you clean, structured text optimized for language model consumption.

```python
from langmail import preprocess_string

result = preprocess_string(raw_email)

print(result.body)
# → "Hi Alice! Great to hear from you."
# (no quoted replies, no signature, no HTML noise)

print(result.from_address)
# → Address(name='Bob', email='bob@example.com')
```

## Why langmail?

- **Built for LLMs** — minimizes token waste by stripping quoted replies, signatures, and HTML noise
- **Fast** — Rust core with zero-copy parsing via [mail-parser](https://github.com/stalwartlabs/mail-parser)
- **Typed** — full type stubs, every field documented
- **Multilingual** — detects quote patterns in English, German, French, and Spanish
- **One function** — `preprocess()` does everything; options available when you need them

## Install

```bash
pip install langmail
```

Requires **Python 3.9 or later**.

Prebuilt wheels for Linux (x64, arm64), macOS (x64, arm64), and Windows (x64). No Rust toolchain needed.

## Usage

### Basic

```python
from langmail import preprocess, preprocess_string

# From raw .eml bytes
with open("message.eml", "rb") as f:
    result = preprocess(f.read())

# Or from a string
result = preprocess_string(raw_email_string)
```

### With options

```python
from langmail import preprocess_with_options, PreprocessOptions

options = PreprocessOptions(
    strip_quotes=True,       # Remove quoted replies (default: True)
    strip_signature=True,    # Remove email signatures (default: True)
    max_body_length=4000,    # Truncate body to N chars (default: 0 = no limit)
)

result = preprocess_with_options(raw, options)
```

### Format for LLM prompts

`to_llm_context` converts a `ProcessedEmail` into a compact, deterministic plain-text
block ready to paste into an LLM prompt:

```python
from langmail import preprocess, to_llm_context

result = preprocess(raw)
print(to_llm_context(result))
# FROM: Bob <bob@example.com>
# TO: Alice <alice@example.com>
# SUBJECT: Re: Project update
# DATE: 2024-01-15T10:30:00Z
# CONTENT:
# Hi Alice! Great to hear from you.
```

Missing fields (no `from`, empty `to`, etc.) are simply omitted. The `CONTENT:` line is always present.

### Output structure

```python
class ProcessedEmail:
    body: str                           # Clean text, ready for your LLM
    subject: str | None
    from_address: Address | None        # Address(name='Bob', email='bob@example.com')
    to: list[Address]
    cc: list[Address]
    date: str | None                    # ISO 8601
    rfc_message_id: str | None          # RFC 2822 Message-ID header
    in_reply_to: list[str] | None       # Threading
    references: list[str] | None        # Threading
    signature: str | None               # Extracted signature (if found)
    raw_body_length: int                # Before cleaning
    clean_body_length: int              # After cleaning
    primary_cta: CallToAction | None    # Primary CTA from HTML emails

class CallToAction:
    url: str                            # Action URL
    text: str                           # Button/link label
    confidence: float                   # 0.0–1.0
```

### Error handling

`preprocess`, `preprocess_with_options`, and `preprocess_string` raise `ParseError`
(a `ValueError` subclass) if the input cannot be parsed as a valid RFC 5322 message:

```python
from langmail import preprocess, ParseError

try:
    result = preprocess(raw)
except ParseError as e:
    print(e)  # "Failed to parse email message"
```

`to_llm_context` never raises.

## What it does

| Step                  | Before                                       | After                              |
| --------------------- | -------------------------------------------- | ---------------------------------- |
| MIME parsing           | Raw RFC 5322 bytes                           | Structured parts                   |
| HTML to text           | `<p>Hello <b>world</b></p>`                  | `Hello world`                      |
| Quote stripping        | Gmail/Outlook/Apple Mail quoted replies       | Just the new message               |
| Signature removal      | `-- \nJohn Doe\nCEO, Acme Corp\n555-0123`    | Body without signature             |
| Whitespace cleanup     | Excessive blank lines, trailing spaces        | Clean, normalized text             |

## Supported quote patterns

- **Gmail**: `On <date>, <name> <email> wrote:`
- **Outlook**: `-----Original Message-----` and `From: ... Sent: ...`
- **Apple Mail**: `On <date>, at <time>, <name> wrote:`
- **Forwarded**: `-------- Forwarded Message --------`
- **German**: `Am <date> schrieb <name>:`
- **French**: `Le <date>, <name> a ecrit :`
- **Spanish**: `El <date>, <name> escribio:`
- **Generic**: `> ` prefixed quote lines

## License

MIT OR Apache-2.0

---

Built by the team behind [Marbles](https://marbles.dev).
