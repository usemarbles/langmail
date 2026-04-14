'use strict'

// Public entry point for the langmail Node package.
//
// This file is hand-written. The NAPI-RS build produces `native.js`
// (the raw native binding) and this wrapper re-exports the subset that
// constitutes langmail's public surface, plus the JS-side provider
// adapters. `preprocessParsed` is intentionally NOT re-exported — it is
// an internal hook that powers the provider adapters and may change
// shape before it is published as `preprocessFromParsed`.
//
// The list below is an allow-list (rather than a spread) so new internal
// bindings cannot accidentally leak through the public API.
//
// `RenderMode` is a runtime alias of `NapiRenderMode` — the unprefixed
// name is canonical; the prefixed name stays for backward compat. The
// other NAPI-generated names (`NapiAddress`, `NapiCallToAction`,
// `NapiLlmContextOptions`, `NapiThreadMessage`) are TS-only interfaces
// so they don't need a runtime re-export here; their clean aliases are
// declared in `index.d.ts`.

const native = require('./native.js')
const { preprocessGmail } = require('./src/adapters/gmail.js')

// TODO(spec-01): once Microsoft Graph, Postmark, and SendGrid adapters
// land and the input shape stabilizes, expose `preprocessParsed` here as
// `preprocessFromParsed` (and mirror the re-export in `index.d.ts`).
module.exports = {
  preprocess: native.preprocess,
  preprocessWithOptions: native.preprocessWithOptions,
  preprocessString: native.preprocessString,
  toLlmContext: native.toLlmContext,
  toLlmContextWithOptions: native.toLlmContextWithOptions,
  NapiRenderMode: native.NapiRenderMode,
  RenderMode: native.NapiRenderMode,
  preprocessGmail,
}
