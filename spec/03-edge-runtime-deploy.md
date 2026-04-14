# Spec 03 — Edge-runtime deploy story (WASM)

**Status:** draft
**Priority:** P1

## Problem

langmail ships prebuilt native binaries via NAPI-RS. This works for Node, Bun, and traditional server deploys. It does **not** work for:

- Cloudflare Workers (no filesystem, no native addons)
- Vercel Edge Runtime (V8 isolates, no native addons)
- Deno Deploy (native binding support is incomplete)
- Browser / client-side usage

A large slice of AI app developers deploy to exactly those environments. For them, langmail is a hard "no" regardless of quality.

## Goal

Ship a WASM build that runs everywhere JavaScript runs, with the same public API.

## Surface

A second distribution under the same npm package:

```js
// Auto-selects native on Node/Bun, WASM on edge runtimes
import { preprocess } from "langmail";

// Or pin explicitly
import { preprocess } from "langmail/wasm";
import { preprocess } from "langmail/native";
```

Export map in `package.json`:

```json
{
  "exports": {
    ".": {
      "workerd": "./wasm/index.js",
      "edge-light": "./wasm/index.js",
      "node": "./native/index.js",
      "bun": "./native/index.js",
      "default": "./wasm/index.js"
    },
    "./wasm": "./wasm/index.js",
    "./native": "./native/index.js"
  }
}
```

## Implementation notes

- Build the core `langmail` crate for `wasm32-unknown-unknown` via `wasm-bindgen` in a new `crates/langmail-wasm/` crate. Reuse the same Rust pipeline — no logic fork.
- Target size: <500KB gzipped for the initial release (tight for Workers' 1 MB limit with room for user code).
- Ship a `.wasm` binary plus a JS loader that uses `WebAssembly.instantiate` with the inlined module for Workers' module-syntax deploys.
- No filesystem access, no network — the core is already pure, so this should be achievable.
- Keep native binding as default on Node/Bun for perf; WASM is ~3–5× slower but still sub-millisecond per email.

## Test matrix

CI runs the full test suite against each target:

| Runtime              | Build       | CI job                   |
| -------------------- | ----------- | ------------------------ |
| Node 18/20/22        | native      | existing                 |
| Bun latest           | native      | new                      |
| Cloudflare Workers   | WASM        | `wrangler dev` + Vitest  |
| Vercel Edge          | WASM        | `@edge-runtime/vm`       |
| AWS Lambda (Node 20) | native      | zip + invoke             |
| Deno                 | WASM        | `deno test`              |

One integration test per runtime: `preprocessString` on a canonical fixture, assert `ProcessedEmail` shape.

## Documentation

New `docs/deploy.md` with per-runtime snippets:

- Cloudflare Workers `wrangler.toml` + `import` example
- Vercel Edge Function example
- AWS Lambda (native) example
- Bun example
- Deno example

Each snippet is copy-pasteable, ≤15 lines.

## Scope

**In:** WASM build targeting the `wasm32-unknown-unknown` triple, auto-selecting loader, documented deploy recipes, CI coverage for each runtime.

**Out:**
- Browser/DOM-specific features (no HTML sandboxing concerns).
- Streaming APIs.
- Shrinking the WASM bundle below 500KB in v1 — follow-up if size becomes a real complaint.

## Success criteria

- `import { preprocess } from "langmail"` works on Cloudflare Workers with zero extra config.
- README Features list adds: *"Runs on Cloudflare Workers, Vercel Edge, Lambda, Bun, Deno, Node."*
- CI blocks releases that break any of the supported runtimes.

## Open questions

- Does `wasm-bindgen` + NAPI-RS coexist cleanly in one workspace, or does the WASM build need a separate crate with a shared `langmail-core`? Leaning toward the latter.
- Tree-shake thresholds: can we ship a "core-only" WASM without the CTA extractor to fit in <300KB for extreme-size-constrained users? Defer unless requested.
