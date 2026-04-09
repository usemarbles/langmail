---
title: Quick Start
description: Parse your first email in under two minutes.
---

## TypeScript / Node.js

Call `preprocess()` with a raw email string, then pass the result to `toLLMContext()` to get clean Markdown output.

```ts
import { preprocess, toLLMContext } from "langmail"
import { readFileSync } from "fs"

// load a raw .eml file
const raw = readFileSync("email.eml", "utf8")

// parse and clean
const parsed = await preprocess(raw)

// serialize to LLM-ready Markdown
const context = toLLMContext(parsed)

console.log(context)
```

### Example output

Given a typical reply-chain email, the output looks like this:

```text
From: Alice <alice@example.com>
Subject: Q4 budget review
Date: 2024-11-12

Hi,

Following up on the Q4 numbers. Can you send
the updated forecast by Friday?

— [quoted reply removed] —
— [signature removed] —
```

## Python

```python
from langmail import preprocess, to_llm_context

with open("email.eml") as f:
    raw = f.read()

parsed  = preprocess(raw)
context = to_llm_context(parsed)

print(context)
```

## Rust

```rust
use langmail_core::{ preprocess, to_llm_context };

let raw = std::fs::read_to_string("email.eml")?;
let parsed  = preprocess(&raw)?;
let context = to_llm_context(&parsed);

println!("{}", context);
```

!!! tip
    Don't have a `.eml` file handy? Any raw RFC 5322 string works — including strings fetched from IMAP, the Gmail API, or any other email source.
