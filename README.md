# langmail

**Email preprocessing for LLMs.** Fast, typed, Rust-powered.

Emails are messy — nested MIME parts, quoted reply chains, HTML cruft, signatures, forwarded headers. LLMs don't need any of that. langmail strips it all away and gives you clean, structured text optimized for language model consumption.

```typescript
import { preprocessString } from "langmail";

const result = preprocessString(rawEmail);

console.log(result.body);
// → "Hi Alice! Great to hear from you."
// (no quoted replies, no signature, no HTML noise)

console.log(result.from);
// → { name: "Bob", email: "bob@example.com" }
```

## Why langmail?

- **Built for LLMs** — minimizes token waste by stripping quoted replies, signatures, and HTML noise
- **Fast** — Rust core with zero-copy parsing via [mail-parser](https://github.com/stalwartlabs/mail-parser)
- **Typed** — full TypeScript definitions, every field documented
- **Multilingual** — detects quote patterns in English, German, French, and Spanish
- **One function** — `preprocess()` does everything; options available when you need them

## Install

```bash
npm install langmail
```

Prebuilt native binaries for Linux (x64, arm64), macOS (x64, arm64), and Windows (x64). No Rust toolchain needed.

## Usage

### Basic

```typescript
import { preprocess } from "langmail";
import { readFileSync } from "fs";

// From raw .eml file
const raw = readFileSync("message.eml");
const result = preprocess(raw);

// Or from a string (e.g. Gmail API response)
import { preprocessString } from "langmail";
const result = preprocessString(rawEmailString);
```

### With options

```typescript
import { preprocessWithOptions } from "langmail";

const result = preprocessWithOptions(raw, {
  stripQuotes: true, // Remove quoted replies (default: true)
  stripSignature: true, // Remove email signatures (default: true)
  maxBodyLength: 4000, // Truncate body to N chars (default: 0 = no limit)
});
```

### Output structure

```typescript
interface ProcessedEmail {
  body: string; // Clean text, ready for your LLM
  subject?: string;
  from?: { name?: string; email: string };
  to: { name?: string; email: string }[];
  cc: { name?: string; email: string }[];
  date?: string; // ISO 8601
  messageId?: string;
  inReplyTo?: string[]; // Threading
  references?: string[]; // Threading
  signature?: string; // Extracted signature (if found)
  rawBodyLength: number; // Before cleaning
  cleanBodyLength: number; // After cleaning
}
```

## What it does

| Step                  | Before                                       | After                              |
| --------------------- | -------------------------------------------- | ---------------------------------- |
| MIME parsing           | Raw RFC 5322 bytes                           | Structured parts                   |
| HTML → text           | `<p>Hello <b>world</b></p>`                  | `Hello world`                      |
| Quote stripping        | Gmail/Outlook/Apple Mail quoted replies       | Just the new message               |
| Signature removal      | `-- \nJohn Doe\nCEO, Acme Corp\n555-0123`    | Body without signature             |
| Whitespace cleanup     | Excessive blank lines, trailing spaces        | Clean, normalized text             |

## Supported quote patterns

- **Gmail**: `On <date>, <name> <email> wrote:`
- **Outlook**: `-----Original Message-----` and `From: ... Sent: ...`
- **Apple Mail**: `On <date>, at <time>, <name> wrote:`
- **Forwarded**: `-------- Forwarded Message --------`
- **German**: `Am <date> schrieb <name>:`
- **French**: `Le <date>, <name> a écrit :`
- **Spanish**: `El <date>, <name> escribió:`
- **Generic**: `> ` prefixed quote lines

## Performance

langmail uses [mail-parser](https://github.com/stalwartlabs/mail-parser) under the hood — a zero-copy Rust MIME parser with no external dependencies. The preprocessing pipeline adds minimal overhead on top of the parse step.

Typical throughput on a modern machine: **10,000+ emails/second** for plain text messages.

## License

MIT OR Apache-2.0

---

Built by the team behind [Marbles](https://marbles.dev). If you need the full pipeline — email ingestion, AI classification, routing, and response generation — check us out.
