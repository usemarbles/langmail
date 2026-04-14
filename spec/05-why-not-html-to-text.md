# Spec 05 — Positioning: "Why not just html-to-text?"

**Status:** draft
**Priority:** P2

## Problem

When an evaluator asks "do I need langmail?" the default answer their subconscious reaches for is: *"I'll just `html-to-text` the body. Good enough."* This was literally the fallback I recommended when reviewing langmail. It's the cheap alternative that sits in every evaluator's head as the escape hatch.

If langmail doesn't explicitly demolish this comparison, it loses the evaluation to an already-installed npm package the evaluator has used before. The escape hatch must be closed in the README itself, not left to discovery.

## Goal

A prominent README section (and matching docs page) titled **"Why not just html-to-text?"** that makes the gap visible and undeniable.

## Content outline

### Opening frame

One line: *"html-to-text converts HTML to plain text. langmail preprocesses email for LLMs. Those are different jobs."*

### Concrete example

Pick one real-shape email — a reply to a reply with an HTML signature, a corporate disclaimer footer, and an image-heavy marketing CTA block. Show:

**Input**: the raw email (collapsed by default in a `<details>`).

**`html-to-text` output**: ~40 lines. Contains:
- The actual message (~3 lines)
- The quoted prior reply (~15 lines, "On Mon, Alice wrote:" then "> Hi Bob, ...")
- The quoted reply before *that* (~10 lines, double-indented)
- The signature ("Bob Smith / VP Engineering / ..." — 6 lines)
- The corporate legal footer (5 lines of "this email is confidential...")

**langmail output**: ~3 lines. Just the actual message. `threadMessages` array has the prior replies structured separately, `signature` field has the signature, disclaimer is dropped.

**Token counts** for each, plus the delta.

### Comparison table

| Capability                              | html-to-text | mailparser + html-to-text | langmail |
| --------------------------------------- | :----------: | :-----------------------: | :------: |
| HTML → plain text                       |      ✅      |            ✅             |    ✅    |
| MIME parsing (headers, addresses)       |      ❌      |            ✅             |    ✅    |
| Strip quoted reply chains               |      ❌      |            ❌             |    ✅    |
| Strip signatures                        |      ❌      |            ❌             |    ✅    |
| Strip corporate/legal footers           |      ❌      |            ❌             |    ✅    |
| Structured thread messages              |      ❌      |            ❌             |    ✅    |
| CTA extraction                          |      ❌      |            ❌             |    ✅    |
| LLM-ready output formatter              |      ❌      |            ❌             |    ✅    |
| Native/Rust-speed                       |      ❌      |            ❌             |    ✅    |

### When html-to-text is enough

Be honest. If the user's job is "display email in a terminal client," `html-to-text` is fine. If the job is "feed email to an LLM," it's not. State this directly — credibility beats puffery.

## Scope

**In:**
- README section at the same nesting level as "Features" and "Performance," inserted between "Quick Start" and "API Reference."
- A matching `docs/comparison.md` with the longer version: more examples, more edge cases, links to the playground's compare mode (spec 04).
- The worked example is generated from a real fixture in `crates/langmail/tests/fixtures/` — commit the fixture so readers can reproduce.

**Out:**
- Comparisons with every parser on npm.
- Dunking. The tone is "different tools for different jobs," not "html-to-text bad."

## Success criteria

- An evaluator reading the README reaches the worked example before they close the tab, and the example makes the gap self-evident.
- The comparison table is reachable from the first paragraph of the README (via anchor or visible near the top).
- Docs comparison page links to the playground's compare mode so readers can verify on their own data.

## Risk

- `html-to-text` maintainers or users may object to the framing. Mitigation: keep tone neutral, acknowledge where `html-to-text` is genuinely the right tool, and link to its README.
