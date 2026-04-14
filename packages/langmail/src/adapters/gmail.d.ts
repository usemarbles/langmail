// Hand-written type declarations for the Gmail adapter.
//
// Mirrors packages/langmail/src/adapters/gmail.js. Structural typing is
// used so callers can pass the actual `gmail_v1.Schema$Message` object
// from `googleapis` without needing to import or cast to our alias
// types — the fields we depend on are a strict subset.

import type { ProcessedEmail, PreprocessOptions } from '../../native'

/** Minimal structural type matching the subset of `gmail_v1.Schema$Message` the adapter reads. */
export interface GmailMessage {
  /** Parsed payload (only present when the message was fetched with `format: 'full'`). */
  payload?: GmailMessagePart
  [key: string]: unknown
}

/** Minimal structural type for a Gmail message part (`gmail_v1.Schema$MessagePart`). */
export interface GmailMessagePart {
  /** e.g. "text/plain", "text/html", "multipart/alternative". */
  mimeType?: string
  /** Header name/value pairs. */
  headers?: Array<{ name?: string; value?: string }>
  /** Body payload (base64url-encoded in `data`). */
  body?: {
    data?: string
    size?: number
  }
  /** Child parts for multipart containers. */
  parts?: GmailMessagePart[]
}

/** Either a bare Gmail message or the full googleapis response wrapping one. */
export type GmailInput = GmailMessage | { data: GmailMessage }

/**
 * Preprocess a Gmail API message through langmail's cleaning pipeline.
 *
 * Works on the response of `gmail.users.messages.get({ id, format: 'full' })`
 * — pass either the bare `Schema$Message` or the full `{ data, status, ... }`
 * response. Skips MIME parsing entirely and feeds the already-decoded
 * bodies into langmail's HTML→Markdown, quote-stripping, signature-stripping
 * pipeline.
 */
export declare function preprocessGmail(
  msg: GmailInput,
  options?: PreprocessOptions,
): ProcessedEmail
