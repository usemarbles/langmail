'use strict'

// Gmail provider adapter (Node wrapper).
//
// All Gmail-specific logic — body tree walk, base64url decoding, address
// parsing, RFC 2822 date conversion, Message-ID list parsing — lives in
// the Rust core (`crates/langmail/src/adapters/gmail.rs`). This file is
// a thin wrapper that serializes the caller's object to JSON and
// delegates to the native binding, so Node and Python share exactly one
// implementation.
//
// The wrapper performs only two things the Rust side can't do cheaply:
// - Guard against non-object input (so callers get a clear `TypeError`
//   instead of a generic JSON parse error).
// - `JSON.stringify` the message — PyO3 and NAPI-RS both find a JSON
//   string the cheapest shared handoff for deeply-nested optional
//   structures.
//
// No runtime dependency on `googleapis` — only the shape of the
// response is consumed.

const { preprocessGmail: preprocessGmailNative } = require('../../native.js')

/**
 * Preprocess a Gmail API message through langmail's cleaning pipeline.
 *
 * Accepts either the raw `Schema$Message` object or the full googleapis
 * response (`{ data, status, ... }`). Requires the message to have been
 * fetched with `format: 'full'` so `payload` is present with headers
 * and base64url-encoded body parts. The googleapis wrapper is unwrapped
 * inside the Rust core.
 *
 * Throws:
 * - `TypeError` if the input is not an object.
 * - `Error` if the message has no `payload` (fetch with `format: 'full'`)
 *   or if the chosen body part is attachment-backed (Gmail returned
 *   `body.attachmentId` because the body exceeded the inline size
 *   threshold — fetch with `users.messages.attachments.get` and inline
 *   the decoded content).
 *
 * @param {object} msg - Gmail message (or full googleapis response)
 * @param {object} [options] - PreprocessOptions
 * @returns {object} ProcessedEmail
 */
function preprocessGmail(msg, options) {
  if (msg === null || typeof msg !== 'object') {
    throw new TypeError(
      `preprocessGmail: expected a Gmail message object, got ${msg === null ? 'null' : typeof msg}`,
    )
  }

  return preprocessGmailNative(JSON.stringify(msg), options)
}

module.exports = { preprocessGmail }
