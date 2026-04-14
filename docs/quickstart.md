---
title: Quick Start
description: Parse your first email in under two minutes.
---

## TypeScript / Node.js

Call `preprocessString()` with a raw email string, then pass the result to `toLlmContext()` to get a clean, LLM-ready context block. (If you already have the email as a `Buffer`, use `preprocess()` instead.)

```ts
import { preprocessString, toLlmContext } from "langmail"
import { readFileSync } from "fs"

// load a raw .eml file as a string
const raw = readFileSync("email.eml", "utf8")

// parse and clean (synchronous)
const parsed = preprocessString(raw)

// serialize to LLM-ready context
const context = toLlmContext(parsed)

console.log(context)
```

### Example output

Given a typical reply-chain email, the output looks like this:

```text
FROM: Alice <alice@example.com>
SUBJECT: Q4 budget review
DATE: 2024-11-12

CONTENT:
Hi,

Following up on the Q4 numbers. Can you send
the updated forecast by Friday?
```

### Gmail API

If your Node app is already calling `gmail.users.messages.get({ format: "full" })` through `googleapis`, feed the parsed response directly to `preprocessGmail` instead of switching to `format: "raw"`:

```ts
import { preprocessGmail, toLlmContext } from "langmail"
import { google } from "googleapis"

const gmail = google.gmail({ version: "v1", auth })
const { data: msg } = await gmail.users.messages.get({
  userId: "me",
  id: messageId,
  format: "full",
})

const parsed  = preprocessGmail(msg)
const context = toLlmContext(parsed)
```

`preprocessGmail` walks `payload.parts`, base64url-decodes the HTML/text body, normalizes headers, and runs the same cleaning pipeline as `preprocess` — no MIME re-parsing, no extra fetch.

!!! note
    langmail does not bundle or depend on `googleapis` — only the shape of the response is consumed. Install `googleapis` (or any client that returns the same `Schema$Message`) separately.

## Python

`preprocess()` takes raw bytes, so open the file in binary mode (`"rb"`).

```python
from langmail import preprocess, to_llm_context

with open("email.eml", "rb") as f:
    raw = f.read()

parsed  = preprocess(raw)
context = to_llm_context(parsed)

print(context)
```

## Rust

`preprocess` returns a `Result<ProcessedEmail, _>`, and `to_llm_context` is a method on `ProcessedEmail`.

```rust
use langmail::preprocess;

let raw = std::fs::read("email.eml")?;
let parsed  = preprocess(&raw)?;
let context = parsed.to_llm_context();

println!("{}", context);
```

!!! tip
    Don't have a `.eml` file handy? Any raw RFC 5322 message works — including bytes fetched from IMAP, the Gmail API, or any other email source.
