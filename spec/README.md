# langmail — adoption specs

Seven small specs for changes that increase the probability an evaluator answers **"yes, langmail helps here"** rather than **"interesting, but not worth the integration cost."**

Grounded in an actual evaluation of langmail against a real Gmail-driven LLM pipeline. Each spec names the friction it removes.

## Ordering

Priority is stack-rank, not a strict dependency graph — but specs with shared prerequisites are noted.

| #   | Spec                                                          | Priority | Why it matters                                                                                |
| --- | ------------------------------------------------------------- | -------- | --------------------------------------------------------------------------------------------- |
| 01  | [Provider adapters](./01-provider-adapters.md)                | **P0**   | Removes the single biggest "no" in evaluations — forcing users to switch to `format: "raw"`.  |
| 02  | [Token-savings benchmarks](./02-token-savings-benchmarks.md)  | **P0**   | Replaces the unresonant "Rust-fast" pitch with a quantified quality/cost claim.               |
| 03  | [Edge-runtime deploy (WASM)](./03-edge-runtime-deploy.md)     | P1       | Unlocks Cloudflare Workers / Vercel Edge / Deno — a big slice of the AI-app audience.         |
| 04  | [Live playground](./04-playground.md)                         | P1       | Closes the gap from "looks interesting" to "verified on my data" in <30 seconds. Needs 03.    |
| 05  | [Why not html-to-text](./05-why-not-html-to-text.md)          | P2       | Closes the escape hatch every evaluator's brain reaches for as the "good enough" alternative. |
| 06  | [Thread reconstruction](./06-thread-reconstruction.md)        | P2       | Solves the real pain of "N messages → one clean LLM prompt" that everyone reimplements.       |
| 07  | [MCP server](./07-mcp-server.md)                              | P2       | Distribution channel into Claude Code / Desktop / Cursor with zero install friction.          |

## If you do only one thing

**Spec 01.** Every other improvement is diminished by the current MIME-only entry point. With Gmail + Graph adapters, a "not worth the churn" evaluation becomes "drop-in, clear token savings, low risk."

## Dependencies

- Spec 04 depends on Spec 03 (playground runs the WASM build client-side).
- Spec 07 benefits from Spec 01 (`preprocess_gmail` tool wraps the Gmail adapter).
- Spec 02 can proceed independently but the benchmark harness is easier to scale once Spec 01 exists (run benchmarks directly on Gmail JSON, not only on `.eml`).

## Non-goals across the set

Stated once here so each spec doesn't have to repeat it:

- **No hosted SaaS / classification API.** langmail stays a library. A hosted endpoint would be a different company.
- **No LLM wrapper.** Users bring their own model and prompt. langmail preprocesses; it does not classify or summarise.
- **No attachment handling.** Out of scope for v1 of any of these specs.
- **No streaming parser.** Email bodies fit in memory; streaming adds complexity without a concrete user need.
