// Public type surface for the langmail Node package.
//
// Hand-written wrapper around the NAPI-RS generated `native.d.ts`. Keep
// this file in sync with `index.js`: exactly the exports listed here are
// the ones re-exported from `index.js`. `preprocessParsed` and its
// `NapiParsedInput` input type are omitted on purpose — they are
// internal implementation details of the provider adapters.
//
// The `Napi`-prefixed names come from the NAPI-RS generator; we re-export
// them under clean, unprefixed aliases (`Address`, `CallToAction`, `LlmContextOptions`,
// `RenderMode`, `ThreadMessage`) which are the canonical public names. The
// prefixed names remain exported for backward compatibility.
//
// Value re-exports (functions, const enums) and type re-exports
// (interfaces) are kept on separate `export` / `export type` lines so
// downstream packages with `verbatimModuleSyntax` / `--isolatedModules`
// compile cleanly against them.

export {
  preprocess,
  preprocessWithOptions,
  preprocessString,
  toLlmContext,
  toLlmContextWithOptions,
  NapiRenderMode,
  NapiRenderMode as RenderMode,
} from './native'

export type {
  NapiAddress,
  NapiAddress as Address,
  NapiCallToAction,
  NapiCallToAction as CallToAction,
  NapiLlmContextOptions,
  NapiLlmContextOptions as LlmContextOptions,
  NapiThreadMessage,
  NapiThreadMessage as ThreadMessage,
  PreprocessOptions,
  ProcessedEmail,
} from './native'

export { preprocessGmail } from './src/adapters/gmail'
export type {
  GmailMessage,
  GmailMessagePart,
  GmailInput,
} from './src/adapters/gmail'
