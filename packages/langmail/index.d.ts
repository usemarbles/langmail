// Public type surface for the langmail Node package.
//
// Hand-written wrapper around the NAPI-RS generated `native.d.ts`. Keep
// this file in sync with `index.js`: exactly the exports listed here are
// the ones re-exported from `index.js`. `preprocessParsed` and its
// `NapiParsedInput` input type are omitted on purpose — they are
// internal implementation details of the provider adapters.

export {
  preprocess,
  preprocessWithOptions,
  preprocessString,
  toLlmContext,
  toLlmContextWithOptions,
  NapiRenderMode,
  NapiAddress,
  NapiCallToAction,
  NapiLlmContextOptions,
  NapiThreadMessage,
  PreprocessOptions,
  ProcessedEmail,
} from './native'

export { preprocessGmail } from './src/adapters/gmail'
export type { GmailMessage, GmailMessagePart } from './src/adapters/gmail'
