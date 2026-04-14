# Spec 04 — Live playground

**Status:** draft
**Priority:** P1

## Problem

Developer libraries live or die on "show, don't tell." A skeptic reading the README sees claims ("we strip quoted replies and signatures") but can't verify them on their own data without installing. The friction between "looks interesting" and "works on my emails" is too high.

## Goal

On `langmail.dev`, a web playground where a visitor pastes a raw email (or provider JSON) and sees:

1. The cleaned Markdown body (langmail output).
2. The structured metadata (from/to/cc/subject/date/thread).
3. A **token-delta counter**: "1,842 tokens → 412 tokens (−77%)".
4. A **compare tab**: raw / html-to-text / mailparser / langmail side-by-side on the same input.

No signup. No API key. Runs entirely in the browser via the WASM build (see spec 03).

## Surface

New route under the Zensical docs site (or a standalone static page if Zensical makes embedding a React/Solid playground awkward): `langmail.dev/playground`.

Input:
- Textarea for raw `.eml` content.
- Tab for "Gmail JSON" that accepts pasted `messages.get` response.
- "Load example" dropdown with 6–10 curated real-shape emails: marketing, long reply chain, HTML newsletter with CTAs, multi-part, forwarded, non-English, signature-heavy, iPhone-sent.

Output panels:
- **Cleaned body** (Markdown rendered + raw toggle).
- **Structured metadata** (JSON tree).
- **LLM context** — rendered output of `toLlmContext()` / `toLlmContextWithOptions({ renderMode: "ThreadHistory" })`.
- **Token count** — live, using `@anthropic-ai/tokenizer` in-browser. Shows raw vs. cleaned for each.

Compare mode:
- Three-column view: raw input, `html-to-text` output, langmail output.
- Each column shows its own token count.
- Makes the value gap obvious at a glance.

## Implementation notes

- WASM build (spec 03) is a hard dependency. The playground runs langmail entirely client-side.
- `html-to-text` and a lightweight `mailparser`-equivalent are also bundled client-side for the compare view.
- Stack: minimal — vanilla TS + a small framework (Preact/Solid) to keep bundle ≤200KB excluding WASM. No Next.js.
- Share button: encodes input as URL hash (`#input=<base64-deflate>`) so users can share reproducers. Size-limit with a "too large to share" warning over ~30KB.
- Privacy: all processing is client-side; nothing leaves the browser. State this prominently on the page so security-conscious users feel safe pasting real emails.

## Scope

**In:**
- Input box + provider-payload tab.
- Curated examples.
- All three output panels.
- Token delta.
- Compare mode.
- Shareable URLs.
- "Privacy: runs in your browser" banner.

**Out:**
- Authentication, saved history, user accounts.
- Server-side rendering or stored submissions (intentionally — avoids handling PII).
- Mobile optimization beyond "doesn't break below 375px."

## Success criteria

- Time-to-first-demo from landing page click is <10 seconds (curated example loaded by default).
- A user can paste their own email, see the cleaned output and token delta, and reach "I get it" in under 30 seconds without installing anything.
- The playground URL is the primary call-to-action on `langmail.dev` (above `npm install`).

## Stretch

- "Try the Gmail adapter" mode: paste a Gmail `messages.get` payload and see `preprocessGmail` output alongside `preprocess` output on the same logical email.
- LLM classify button: "Run Haiku classification on this (bring your own key)" — shows the actual downstream effect. Optional, post-v1.
