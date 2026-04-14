# langmail

**Email preprocessing for LLMs.** Fast, typed, Rust-powered.

[![crates.io](https://img.shields.io/crates/v/langmail?label=crates.io)](https://crates.io/crates/langmail)
[![npm](https://img.shields.io/npm/v/langmail?label=npm)](https://www.npmjs.com/package/langmail)
[![PyPI](https://img.shields.io/pypi/v/langmail?label=PyPI)](https://pypi.org/project/langmail/)
[![docs](https://img.shields.io/badge/docs-langmail.dev-4c1)](https://langmail.dev)
[![CI](https://github.com/usemarbles/langmail/actions/workflows/ci.yml/badge.svg)](https://github.com/usemarbles/langmail/actions/workflows/ci.yml)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)

Emails are messy — nested MIME parts, quoted reply chains, HTML cruft, signatures, forwarded headers. LLMs don't need any of that. langmail strips it all away and gives you clean, structured **Markdown** optimized for language model consumption.

## Features

- **MIME parsing** — handles nested multipart messages, attachments, and encoded headers
- **HTML to Markdown** — converts HTML email bodies to clean Markdown, preserving links, headings, and structure
- **Quote stripping** — detects and removes quoted replies from Gmail, Outlook, Apple Mail, forwarded messages, and `>` prefixed lines; supports English, German, French, and Spanish
- **Signature removal** — strips signatures (preserved separately in the output); detected via `-- ` delimiter and heuristics
- **CTA extraction** — extracts the primary call-to-action from HTML emails via JSON-LD or heuristic link scoring; filters out unsubscribe/privacy/logo links
- **Thread history** — extracts quoted reply blocks into a structured list (oldest first); available via the thread-history render mode
- **Whitespace cleanup** — normalizes excessive blank lines and trailing spaces

## Install

### AI coding agents

Drop this prompt into Claude Code, Cursor, or any other coding agent to have it integrate langmail into your project:

```text
Integrate the langmail library into this project. langmail is an
email preprocessing library that prepares raw email content for LLM
consumption — parsing, cleaning, and rendering it into context strings.

Before writing any code, fetch the current API reference at:
https://langmail.dev/llms.txt

Use it to determine the correct package name, install method, and API
for this project's language and runtime, then implement accordingly.
```

### Node.js

```bash
npm install langmail
```

Requires **Node.js 18+**.

### Rust

```bash
cargo add langmail
```

Requires **stable Rust**.

### Python

```bash
pip install langmail
```

Requires **Python 3.9+**.

Prebuilt native binaries ship with the Node.js and Python packages — no Rust toolchain needed at install time.

## Quick Start

### TypeScript / Node.js

```typescript
import { preprocess, preprocessString, toLlmContext } from "langmail";
import { readFileSync } from "fs";

// From a raw .eml file
const raw = readFileSync("message.eml");
const email = preprocess(raw);

// Or from a string (e.g. Gmail API response)
const fromString = preprocessString(rawEmailString);

console.log(email.body);
// → Hi Alice! Great to hear from you.

console.log(email.from);
// → { name: "Bob", email: "bob@example.com" }

// Format for an LLM prompt
console.log(toLlmContext(email));
// FROM: Bob <bob@example.com>
// TO: Alice <alice@example.com>
// SUBJECT: Re: Project update
// DATE: 2024-01-15T10:30:00Z
// CONTENT:
// Hi Alice! Great to hear from you.
```

#### Gmail API

Already using the Gmail API with `format: "full"`? Skip the round-trip to raw MIME — feed the parsed response straight into langmail with `preprocessGmail`:

```typescript
import { preprocessGmail, toLlmContext } from "langmail";
import { google } from "googleapis";

const gmail = google.gmail({ version: "v1", auth });
const { data: msg } = await gmail.users.messages.get({
  userId: "me",
  id: messageId,
  format: "full",
});

const email = preprocessGmail(msg);
console.log(toLlmContext(email));
```

No need to switch to `format: "raw"`, double the payload size, or re-parse MIME — `preprocessGmail` walks `payload.parts`, base64url-decodes the HTML/text body, and runs the same cleaning pipeline as `preprocess`. `googleapis` is a separate install; langmail only consumes the response shape.

### Rust

```rust
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw = fs::read("message.eml")?;
    let email = langmail::preprocess(&raw)?;

    println!("{}", email.body);
    // → Hi Alice! Great to hear from you.

    println!("{:?}", email.from);
    // → Some(Address { name: Some("Bob"), email: "bob@example.com" })

    // Format for an LLM prompt
    println!("{}", email.to_llm_context());
    Ok(())
}
```

### Python

```python
import langmail

with open("message.eml", "rb") as f:
    email = langmail.preprocess(f.read())

print(email.body)
# → Hi Alice! Great to hear from you.

print(email.from_address)
# → Address(name='Bob', email='bob@example.com')

# Format for an LLM prompt
print(langmail.to_llm_context(email))
```

> **Full API reference** (all functions, types, and per-language signatures): **[langmail.dev](https://langmail.dev)**

## Performance

langmail uses [mail-parser](https://github.com/stalwartlabs/mail-parser) under the hood — a zero-copy Rust MIME parser. The preprocessing pipeline adds minimal overhead on top of the parse step.

Typical throughput on a modern machine: **10,000+ emails/second** for plain text messages.

## Contributing

Contributions welcome — see [CONTRIBUTING.md](./CONTRIBUTING.md) for the development setup, test/format/clippy workflow, and commit-message conventions.

## License

MIT OR Apache-2.0

---

Built by the team behind [Marbles](https://marbles.dev).
