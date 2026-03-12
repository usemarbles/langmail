# langmail

**Email preprocessing for LLMs.** Fast, typed, Rust-powered.

[![npm](https://img.shields.io/npm/v/langmail)](https://www.npmjs.com/package/langmail)
[![CI](https://github.com/usemarbles/langmail/actions/workflows/ci.yml/badge.svg)](https://github.com/usemarbles/langmail/actions/workflows/ci.yml)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)

Emails are messy — nested MIME parts, quoted reply chains, HTML cruft, signatures, forwarded headers. LLMs don't need any of that. langmail strips it all away and gives you clean, structured text optimized for language model consumption.

## Table of Contents

- [Install](#install)
- [Quick Start](#quick-start)
- [API Reference](#api-reference)
  - [preprocess(raw)](#preprocessraw)
  - [preprocessString(raw)](#preprocessstringraw)
  - [preprocessWithOptions(raw, options)](#preprocesswithoptionsraw-options)
  - [toLlmContext(email)](#tollmcontextemail)
  - [toLlmContextWithOptions(email, options)](#tollmcontextwithoptionsemail-options)
- [Output Structure](#output-structure)
  - [ProcessedEmail](#processedemail)
  - [Address](#address)
  - [CallToAction](#calltoaction)
  - [ThreadMessage](#threadmessage)
  - [PreprocessOptions](#preprocessoptions)
  - [LlmContextOptions / RenderMode](#llmcontextoptions--rendermode)
- [Features](#features)
  - [Processing Pipeline](#processing-pipeline)
  - [Quote Stripping](#quote-stripping)
  - [Signature Removal](#signature-removal)
  - [CTA Extraction](#cta-extraction)
  - [Thread History](#thread-history)
  - [HTML to Text](#html-to-text)
- [Error Handling](#error-handling)
- [Performance](#performance)
- [Supported Platforms](#supported-platforms)
- [License](#license)

## Install

```bash
npm install langmail
```

Requires **Node.js 18 or later**. Prebuilt native binaries are included — no Rust toolchain needed.

## Quick Start

```typescript
import { preprocess, preprocessString, toLlmContext } from "langmail";
import { readFileSync } from "fs";

// From a raw .eml file
const raw = readFileSync("message.eml");
const email = preprocess(raw);

// Or from a string (e.g. Gmail API response)
const email = preprocessString(rawEmailString);

console.log(email.body);
// → "Hi Alice! Great to hear from you."

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

## API Reference

### `preprocess(raw)`

Parse and preprocess a raw email from a `Buffer`.

```typescript
import { preprocess } from "langmail";
import { readFileSync } from "fs";

const raw = readFileSync("message.eml");
const email = preprocess(raw);
```

**Parameters:**

| Name  | Type     | Description                          |
| ----- | -------- | ------------------------------------ |
| `raw` | `Buffer` | Raw email bytes (RFC 5322 / EML)     |

**Returns:** [`ProcessedEmail`](#processedemail)

**Throws:** If the input cannot be parsed as a valid RFC 5322 message.

---

### `preprocessString(raw)`

Convenience wrapper that accepts a `string` instead of a `Buffer`.

```typescript
import { preprocessString } from "langmail";

const email = preprocessString(rawEmailString);
```

**Parameters:**

| Name  | Type     | Description         |
| ----- | -------- | ------------------- |
| `raw` | `string` | Raw email as string |

**Returns:** [`ProcessedEmail`](#processedemail)

**Throws:** If the input cannot be parsed as a valid RFC 5322 message.

---

### `preprocessWithOptions(raw, options)`

Preprocess with custom options to control quote stripping, signature removal, and body length.

```typescript
import { preprocessWithOptions } from "langmail";

const email = preprocessWithOptions(raw, {
  stripQuotes: true,      // Remove quoted replies (default: true)
  stripSignature: true,   // Remove email signatures (default: true)
  maxBodyLength: 4000,    // Truncate body to N chars (default: 0 = no limit)
});
```

**Parameters:**

| Name      | Type                                      | Description            |
| --------- | ----------------------------------------- | ---------------------- |
| `raw`     | `Buffer`                                  | Raw email bytes        |
| `options` | [`PreprocessOptions`](#preprocessoptions) | Preprocessing options  |

**Returns:** [`ProcessedEmail`](#processedemail)

**Throws:** If the input cannot be parsed as a valid RFC 5322 message.

---

### `toLlmContext(email)`

Format a `ProcessedEmail` as a deterministic plain-text block for LLM prompts. Missing fields are omitted; the `CONTENT:` line is always present.

```typescript
import { preprocess, toLlmContext } from "langmail";

const email = preprocess(raw);
console.log(toLlmContext(email));
// FROM: Bob <bob@example.com>
// TO: Alice <alice@example.com>
// SUBJECT: Re: Project update
// DATE: 2024-01-15T10:30:00Z
// CONTENT:
// Hi Alice! Great to hear from you.
```

**Parameters:**

| Name    | Type                                    | Description            |
| ------- | --------------------------------------- | ---------------------- |
| `email` | [`ProcessedEmail`](#processedemail)     | A preprocessed email   |

**Returns:** `string`

**Never throws.**

---

### `toLlmContextWithOptions(email, options)`

Same as `toLlmContext` but accepts options to control rendering. Use `renderMode: "ThreadHistory"` to include quoted reply history as a chronological transcript.

```typescript
import { preprocess, toLlmContextWithOptions } from "langmail";

const email = preprocess(raw);

// Default: only the latest message
console.log(toLlmContextWithOptions(email, { renderMode: "LatestOnly" }));

// Include thread history
console.log(toLlmContextWithOptions(email, { renderMode: "ThreadHistory" }));
// FROM: Bob <bob@example.com>
// SUBJECT: Re: Project update
// CONTENT:
// Hi Alice! Great to hear from you.
//
// THREAD HISTORY (oldest first):
// ---
// FROM: Alice <alice@example.com>
// DATE: 2024-01-14T09:00:00Z
// Alice's original message here...
// ---
```

**Parameters:**

| Name      | Type                                                    | Description          |
| --------- | ------------------------------------------------------- | -------------------- |
| `email`   | [`ProcessedEmail`](#processedemail)                     | A preprocessed email |
| `options` | [`LlmContextOptions`](#llmcontextoptions--rendermode)   | Rendering options    |

**Returns:** `string`

**Never throws.**

## Output Structure

### `ProcessedEmail`

```typescript
interface ProcessedEmail {
  body: string;                    // Clean text, ready for your LLM
  subject?: string;
  from?: Address;
  to: Address[];
  cc: Address[];
  date?: string;                   // ISO 8601
  rfcMessageId?: string;           // RFC 2822 Message-ID header
  inReplyTo?: string[];            // In-Reply-To header (threading)
  references?: string[];           // References header (threading)
  signature?: string;              // Extracted signature, if found
  rawBodyLength: number;           // Body length before cleaning
  cleanBodyLength: number;         // Body length after cleaning
  primaryCta?: CallToAction;       // Primary call-to-action from HTML body
  threadMessages: ThreadMessage[]; // Quoted replies, oldest first
}
```

### `Address`

```typescript
interface Address {
  name?: string;  // Display name (e.g. "Alice")
  email: string;  // Email address (e.g. "alice@example.com")
}
```

### `CallToAction`

```typescript
interface CallToAction {
  url: string;        // The URL the action points to
  text: string;       // Human-readable label
  confidence: number; // Score between 0.0 and 1.0
}
```

### `ThreadMessage`

```typescript
interface ThreadMessage {
  sender: string;     // Sender attribution (e.g. "Max <max@example.com>")
  timestamp?: string; // ISO 8601, if parseable from the attribution
  body: string;       // Message body (cleaned, no nested quotes)
}
```

### `PreprocessOptions`

```typescript
interface PreprocessOptions {
  stripQuotes?: boolean;    // Remove quoted replies (default: true)
  stripSignature?: boolean; // Remove email signatures (default: true)
  maxBodyLength?: number;   // Max body chars, 0 = no limit (default: 0)
}
```

### `LlmContextOptions` / `RenderMode`

```typescript
interface LlmContextOptions {
  renderMode?: RenderMode; // Default: "LatestOnly"
}

const enum RenderMode {
  /** Only the latest message — all quoted content stripped. */
  LatestOnly = "LatestOnly",
  /** Chronological transcript of quoted replies below the main content. */
  ThreadHistory = "ThreadHistory",
}
```

## Features

### Processing Pipeline

| Step               | Before                                    | After                          |
| ------------------ | ----------------------------------------- | ------------------------------ |
| MIME parsing        | Raw RFC 5322 bytes                        | Structured parts               |
| HTML to text        | `<p>Hello <b>world</b></p>`               | `Hello world`                  |
| Quote stripping     | Gmail/Outlook/Apple Mail quoted replies    | Just the new message           |
| Signature removal   | `-- \nJohn Doe\nCEO, Acme Corp\n555-0123` | Body without signature         |
| CTA extraction      | `<a class="btn" href="...">Confirm</a>`   | `{ url, text, confidence }`    |
| Thread extraction   | Nested quoted reply blocks                | `ThreadMessage[]` (oldest first) |
| Whitespace cleanup  | Excessive blank lines, trailing spaces    | Clean, normalized text         |

### Quote Stripping

langmail detects and strips quoted reply patterns across multiple email clients and languages:

| Pattern           | Example                                        |
| ----------------- | ---------------------------------------------- |
| Gmail             | `On Jan 15, 2024, Bob <bob@ex.com> wrote:`     |
| Outlook           | `-----Original Message-----`                   |
| Outlook (header)  | `From: ... Sent: ...`                          |
| Apple Mail        | `On Jan 15, 2024, at 10:30, Bob wrote:`        |
| Forwarded         | `-------- Forwarded Message --------`          |
| German            | `Am 15.01.2024 schrieb Bob:`                   |
| French            | `Le 15/01/2024, Bob a écrit :`                 |
| Spanish           | `El 15/01/2024, Bob escribió:`                 |
| Generic           | `> ` prefixed quote lines                      |

### Signature Removal

Signatures are detected by the standard `-- ` delimiter and common heuristics. The removed signature is preserved in the `signature` field so you can use it if needed:

```typescript
const email = preprocess(raw);
console.log(email.body);       // Body without signature
console.log(email.signature);  // "John Doe\nCEO, Acme Corp\n555-0123"
```

### CTA Extraction

langmail extracts the primary call-to-action from HTML emails using two strategies:

1. **JSON-LD fast path** — parses `<script type="application/ld+json">` blocks for `potentialAction.target` URLs (confidence: 1.0)
2. **Heuristic scoring** — scores each link based on button styling, action keywords, prominence, and ARIA labels, then picks the highest-scoring link above the threshold

Common non-CTA links (unsubscribe, privacy policy, bare homepages, logo images) are filtered out automatically.

```typescript
const email = preprocess(raw);

if (email.primaryCta) {
  console.log(email.primaryCta.url);        // "https://example.com/confirm"
  console.log(email.primaryCta.text);       // "Confirm your email"
  console.log(email.primaryCta.confidence); // 0.85
}
```

### Thread History

When an email contains quoted replies, langmail extracts them into structured `ThreadMessage` objects (oldest first). Use `toLlmContextWithOptions` with `renderMode: "ThreadHistory"` to include the full conversation:

```typescript
import { preprocess, toLlmContextWithOptions } from "langmail";

const email = preprocess(raw);

// Access thread messages directly
for (const msg of email.threadMessages) {
  console.log(`${msg.sender} (${msg.timestamp}): ${msg.body}`);
}

// Or render as a transcript for your LLM
const context = toLlmContextWithOptions(email, {
  renderMode: "ThreadHistory",
});
```

### HTML to Text

HTML email bodies are converted to clean plain text. Structural elements like links, lists, and headings are preserved in a readable format while all styling, scripts, and layout markup is removed.

## Error Handling

`preprocess`, `preprocessWithOptions`, and `preprocessString` throw if the input cannot be parsed as a valid RFC 5322 message:

```typescript
try {
  const email = preprocess(raw);
} catch (err) {
  // err.message === "Failed to parse email message"
}
```

`toLlmContext` and `toLlmContextWithOptions` never throw.

## Performance

langmail uses [mail-parser](https://github.com/stalwartlabs/mail-parser) under the hood — a zero-copy Rust MIME parser. The preprocessing pipeline adds minimal overhead on top of the parse step.

Typical throughput on a modern machine: **10,000+ emails/second** for plain text messages.

## Supported Platforms

Prebuilt native binaries are published for the following platforms:

| Platform               | Architecture | Package                      |
| ---------------------- | ------------ | ---------------------------- |
| macOS                  | arm64        | `langmail-darwin-arm64`      |
| macOS                  | x64          | `langmail-darwin-x64`        |
| Linux (glibc)          | x64          | `langmail-linux-x64-gnu`     |
| Linux (glibc)          | arm64        | `langmail-linux-arm64-gnu`   |
| Windows                | x64          | `langmail-win32-x64-msvc`    |

These are installed automatically as optional dependencies — `npm install langmail` picks the right binary for your platform.

## License

MIT OR Apache-2.0

---

Built by the team behind [Marbles](https://marbles.dev).
