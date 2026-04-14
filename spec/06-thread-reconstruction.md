# Spec 06 — Thread reconstruction from N messages

**Status:** draft
**Priority:** P2

## Problem

Real email threads arrive as **N separate messages** from the provider (Gmail `threads.get` returns `messages[]`, Graph returns `/me/messages?$filter=conversationId`). LLMs want a single chronological transcript, deduped, with quoted overlap removed.

Today langmail's `threadMessages` field extracts quoted-reply blocks from *one* message. That's useful but doesn't solve the common case: "I have five messages from the same thread, build me one clean prompt."

Every user implements this themselves, badly. Common failures:
- Including overlapping quoted chunks that duplicate content across messages.
- Wrong ordering (reply-date vs. received-date drift on bcc).
- Losing attribution when one message's cleaned body duplicates text from an earlier message's raw body.
- Inconsistent handling of "forwarded" vs. "replied" messages in the same thread.

## Goal

A helper that takes an array of `ProcessedEmail`s (already preprocessed individually) and produces a single LLM-ready thread transcript.

## Surface

```ts
// Pure builder: returns a structured thread
export function buildThread(
  messages: ProcessedEmail[],
  options?: BuildThreadOptions,
): Thread

// Renderer: turns a Thread into an LLM context string
export function toThreadContext(
  thread: Thread,
  options?: ThreadContextOptions,
): string

// Convenience: builder + renderer in one call
export function toThreadContextFrom(
  messages: ProcessedEmail[],
  options?: BuildThreadOptions & ThreadContextOptions,
): string

interface Thread {
  subject?: string                  // Canonical subject, "Re:" prefixes collapsed
  participants: Address[]           // Deduped union of from/to/cc across messages
  messages: ThreadTurn[]            // Ordered oldest → newest, deduped
}

interface ThreadTurn {
  from?: Address
  to: Address[]
  cc: Address[]
  date?: string                     // ISO 8601
  body: string                      // Cleaned, with overlap from prior turns removed
  messageId?: string
  isForwarded: boolean
}

interface BuildThreadOptions {
  /** How to order turns. Default: "date". */
  order?: "date" | "references"
  /** Drop near-duplicate turns (e.g. bcc copy of the same message). Default: true. */
  dedupe?: boolean
  /** Strip content from a turn that already appeared in an earlier turn. Default: true. */
  stripOverlap?: boolean
}

interface ThreadContextOptions {
  /** Separator between turns. Default: "\n---\n". */
  separator?: string
  /** Max total output length in chars. 0 = unlimited. Default: 0. */
  maxLength?: number
  /** If truncating, which end to keep. Default: "newest". */
  keep?: "newest" | "oldest"
}
```

## Behavior

**Ordering** (`order: "date"`, default):
- Sort by `date` ascending.
- Ties broken by `References` chain depth, then `messageId` lexicographic order.

**Ordering** (`order: "references"`):
- Build a tree from `inReplyTo` / `references`; flatten depth-first.
- Falls back to `date` for orphans.

**Dedupe**:
- Two turns are duplicates if `(from.email, date, body-hash)` match, or if they share `messageId`.
- Keep the turn with the most complete headers (tie-breaker: earliest receive order).

**Overlap stripping**:
- For each turn after the first, compute the cleaned-body hash of every *prior* turn.
- If the current turn's body contains the full cleaned body of a prior turn, remove that range.
- Goal: if A sends, B replies quoting A, langmail on B already stripped A — but if `stripQuotes: false` was used, this layer catches it.
- Algorithm: longest-common-substring against prior bodies, threshold ≥80% of the shorter string, case-insensitive whitespace-normalized comparison.

**Canonical subject**:
- Take `subject` of newest turn.
- Strip leading `Re:`, `Fwd:`, `RE:`, `AW:`, `WG:`, `[EXTERNAL]`, etc. (locale-aware list).

## Rendering

`toThreadContext` default output:

```
SUBJECT: <canonical subject>
PARTICIPANTS: Alice <a@ex.com>, Bob <b@ex.com>, Carol <c@ex.com>

--- TURN 1 ---
FROM: Alice <a@ex.com>
DATE: 2024-01-14T09:00:00Z
<body>

--- TURN 2 ---
FROM: Bob <b@ex.com>
DATE: 2024-01-14T10:15:00Z
<body>

--- TURN 3 ---
FROM: Alice <a@ex.com>
DATE: 2024-01-14T11:02:00Z
<body>
```

Deterministic. No trailing whitespace. Missing fields omitted.

## Scope

**In:**
- Pure functions on `ProcessedEmail[]` — no I/O, no provider coupling.
- Dedupe + overlap stripping + canonical subject.
- Both ordering strategies.
- Length truncation.

**Out:**
- Fetching thread members from a provider (that's the caller's job; pairs naturally with spec 01 adapters).
- Per-turn summarization (that's an LLM job, outside langmail's scope).
- HTML rendering of threads.

## Success criteria

- One call produces a clean, chronologically-ordered, dedup'd thread transcript from a Gmail `threads.get` response.
- Documented side-by-side: "without `toThreadContextFrom` (400 lines of duplicated quoted text) vs. with (80 lines of actual content)."
- Covered by fixture tests with 3-, 5-, and 7-message threads, including one with bcc duplicates and one with out-of-order receipts.

## Non-goals

- Cross-thread stitching ("these two threads are actually the same conversation").
- Participant role inference ("Alice is the customer, Bob is the agent").
