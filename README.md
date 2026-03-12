# langmail

**Email preprocessing for LLMs.** Fast, typed, Rust-powered.

[![npm](https://img.shields.io/npm/v/langmail)](https://www.npmjs.com/package/langmail)
[![CI](https://github.com/usemarbles/langmail/actions/workflows/ci.yml/badge.svg)](https://github.com/usemarbles/langmail/actions/workflows/ci.yml)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)

Emails are messy — nested MIME parts, quoted reply chains, HTML cruft, signatures, forwarded headers. LLMs don't need any of that. langmail strips it all away and gives you clean, structured **Markdown** optimized for language model consumption.

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
- [Performance](#performance)
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
const fromString = preprocessString(rawEmailString);

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
  body: string;                    // Clean Markdown, ready for your LLM
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

// TypeScript enum — JS users pass the string literals directly ("LatestOnly" or "ThreadHistory")
const enum RenderMode {
  /** Only the latest message — all quoted content stripped. */
  LatestOnly = "LatestOnly",
  /** Chronological transcript of quoted replies below the main content. */
  ThreadHistory = "ThreadHistory",
}
```

## Features

- **MIME parsing** — handles nested multipart messages, attachments, and encoded headers
- **HTML to Markdown** — converts HTML email bodies to clean Markdown, preserving links, headings, and structure
- **Quote stripping** — detects and removes quoted replies from Gmail, Outlook, Apple Mail, forwarded messages, and `>` prefixed lines; supports English, German, French, and Spanish
- **Signature removal** — strips signatures (preserved in the `signature` field); detected via `-- ` delimiter and heuristics
- **CTA extraction** — extracts the primary call-to-action from HTML emails via JSON-LD (`potentialAction`) or heuristic link scoring; filters out unsubscribe/privacy/logo links
- **Thread history** — extracts quoted reply blocks into structured `ThreadMessage[]` (oldest first); render with `toLlmContextWithOptions({ renderMode: "ThreadHistory" })`
- **Whitespace cleanup** — normalizes excessive blank lines and trailing spaces

## Performance

langmail uses [mail-parser](https://github.com/stalwartlabs/mail-parser) under the hood — a zero-copy Rust MIME parser. The preprocessing pipeline adds minimal overhead on top of the parse step.

Typical throughput on a modern machine: **10,000+ emails/second** for plain text messages.

## License

MIT OR Apache-2.0

---

Built by the team behind [Marbles](https://marbles.dev).
