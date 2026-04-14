'use strict'

// Gmail provider adapter.
//
// Normalizes a gmail_v1.Schema$Message (the object returned by
// `gmail.users.messages.get({ id, format: 'full' })` from googleapis)
// into langmail's ParsedInput shape, then runs the shared cleaning
// pipeline via the internal `preprocessParsed` native binding.
//
// No runtime dependency on `googleapis` — only the shape of the
// response is consumed.

const { preprocessParsed } = require('../../native.js')

/**
 * Preprocess a Gmail API message through langmail's cleaning pipeline.
 *
 * Accepts either the raw `Schema$Message` object or the full response
 * from googleapis (`{ data: Schema$Message, ... }`). Requires the
 * message to have been fetched with `format: 'full'` so `payload` is
 * present with headers and (base64url-encoded) body parts.
 *
 * Body selection: walks `payload.parts` depth-first and picks the first
 * non-attachment leaf of each type. When both `text/html` and `text/plain`
 * are present, HTML wins — it's converted to Markdown before quote and
 * signature stripping. Parts with `Content-Disposition: attachment` or a
 * `filename` are skipped.
 *
 * Known limitations:
 * - Bodies are decoded as UTF-8; the per-part `Content-Type: charset=…`
 *   parameter is not consulted. Gmail normalizes most modern mail to
 *   UTF-8, but legacy encodings (ISO-8859-1, windows-1252, GB2312, …)
 *   will produce mojibake.
 * - Quoted-pair escapes inside address display names (`"foo\"bar"
 *   <x@y>`) are not fully handled; in practice Gmail emits simple,
 *   well-formed address headers.
 *
 * Throws:
 * - `TypeError` if the input is not an object or has no `payload` (i.e.
 *   the message wasn't fetched with `format: 'full'`).
 * - `Error` if the chosen body part is attachment-backed (Gmail returned
 *   `body.attachmentId` instead of `body.data` because the body exceeded
 *   the inline size threshold — fetch with
 *   `users.messages.attachments.get` and inline the decoded content).
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

  // Accept both `response.data` (full googleapis response) and a bare message.
  const message = msg.data != null && typeof msg.data === 'object' && msg.data.payload
    ? msg.data
    : msg

  const payload = message.payload
  if (payload == null || typeof payload !== 'object') {
    throw new TypeError(
      'preprocessGmail: message.payload is missing. Did you fetch with format: "full"?',
    )
  }

  const { html, text } = extractBodies(payload)
  const headers = normalizeHeaders(payload.headers)

  const input = {
    html,
    text,
    subject: headers.get('subject'),
    from: parseAddress(headers.get('from')),
    to: parseAddressList(headers.get('to')),
    cc: parseAddressList(headers.get('cc')),
    date: parseDate(headers.get('date')),
    rfcMessageId: stripAngleBrackets(headers.get('message-id')),
    inReplyTo: parseIdListHeader(headers.get('in-reply-to')),
    references: parseIdListHeader(headers.get('references')),
  }

  return preprocessParsed(input, options)
}

/**
 * Normalize an `In-Reply-To` / `References` header into an array of bare
 * Message-IDs. Returns `undefined` when the header is missing, whitespace-
 * only, or contains no `@`-bearing tokens after parsing — empty-but-present
 * is semantically different from absent, and downstream LLM context
 * rendering relies on the distinction.
 */
function parseIdListHeader(raw) {
  if (typeof raw !== 'string') return undefined
  const ids = splitIdList(raw)
    .map(stripAngleBrackets)
    // Drop trailing `(comment)` fragments and other non-ID tokens — a bare
    // Message-ID always contains an `@`.
    .filter((s) => s.includes('@'))
  return ids.length > 0 ? ids : undefined
}

// ---------------------------------------------------------------------------
// Body extraction
// ---------------------------------------------------------------------------

function extractBodies(payload) {
  // Walk the part tree depth-first. Pick the first text/html leaf and the
  // first text/plain leaf; everything else (attachments, inline images, …)
  // is ignored — attachments are not supported today.
  //
  // Assumption: for real Gmail responses the main body lives in the first
  // text/html (or text/plain) leaf in document order — `multipart/related`
  // inline-image trees and `multipart/alternative` bodies both respect this.
  // Exotic `multipart/mixed` layouts that put a preamble text part before
  // the real body would land on the preamble; we've not seen that in
  // practice on Gmail but it's a known limitation of the simple heuristic.
  let html
  let text

  const visit = (part) => {
    if (part == null || typeof part !== 'object') return

    const mime = typeof part.mimeType === 'string'
      ? part.mimeType.toLowerCase()
      : ''

    const body = part.body && typeof part.body === 'object' ? part.body : undefined
    const data = body && typeof body.data === 'string' ? body.data : undefined

    if (!isAttachmentPart(part)) {
      const isHtmlCandidate = mime === 'text/html' && html === undefined
      const isTextCandidate = mime === 'text/plain' && text === undefined

      if (data) {
        if (isHtmlCandidate) {
          html = decodeBase64Url(data)
        } else if (isTextCandidate) {
          text = decodeBase64Url(data)
        }
      } else if (
        (isHtmlCandidate || isTextCandidate) &&
        body &&
        typeof body.attachmentId === 'string' &&
        body.attachmentId !== ''
      ) {
        // Gmail returns `body.attachmentId` (instead of inline `data`) when
        // a body part exceeds the `get` response's size threshold. Silent
        // fallthrough would leave the email body empty; surface it so the
        // caller can fetch with `users.messages.attachments.get` and inline
        // the decoded content into the part before retrying.
        throw new Error(
          `preprocessGmail: ${mime} body part has no inline data (body.attachmentId=${JSON.stringify(body.attachmentId)}). ` +
            'Fetch the part via gmail.users.messages.attachments.get and set body.data before calling preprocessGmail.',
        )
      }
    }

    if (Array.isArray(part.parts)) {
      for (const child of part.parts) visit(child)
    }
  }

  visit(payload)
  return { html, text }
}

function decodeBase64Url(data) {
  // Gmail returns standard base64url (RFC 4648 §5). Node's Buffer has
  // native base64url support since Node 16 — comfortably covered by
  // this package's `engines.node: ">= 18"`.
  return Buffer.from(data, 'base64url').toString('utf8')
}

function isAttachmentPart(part) {
  // Skip parts explicitly marked as attachments. `filename` on the part
  // itself is a strong signal (Gmail sets it for attached files) and
  // the `Content-Disposition: attachment` header is authoritative.
  if (typeof part.filename === 'string' && part.filename !== '') return true
  if (!Array.isArray(part.headers)) return false
  for (const h of part.headers) {
    if (
      h &&
      typeof h.name === 'string' &&
      typeof h.value === 'string' &&
      h.name.toLowerCase() === 'content-disposition' &&
      h.value.trim().toLowerCase().startsWith('attachment')
    ) {
      return true
    }
  }
  return false
}

// ---------------------------------------------------------------------------
// Header normalization
// ---------------------------------------------------------------------------

function normalizeHeaders(headers) {
  // payload.headers is `Array<{ name: string, value: string }>`. MIME
  // header names are case-insensitive, so we lowercase the key. Multiple
  // occurrences of the same header are rare on inbound mail — fall back
  // to last-wins, which matches mail-parser behavior for the headers we
  // care about (Subject, From, Date, Message-ID).
  const map = new Map()
  if (!Array.isArray(headers)) return map
  for (const h of headers) {
    if (h && typeof h.name === 'string' && typeof h.value === 'string') {
      map.set(h.name.toLowerCase(), h.value)
    }
  }
  return map
}

// ---------------------------------------------------------------------------
// Address parsing
// ---------------------------------------------------------------------------

/** Parse a single "Name <email>", legacy "email (Name)", or bare "email" into { name?, email }. */
function parseAddress(raw) {
  if (typeof raw !== 'string') return undefined
  const trimmed = raw.trim()
  if (trimmed === '') return undefined

  // "Display Name" <user@example.com>  or  Display Name <user@example.com>
  const angleMatch = trimmed.match(/^(?:"([^"]*)"|([^<]*?))\s*<([^>]+)>\s*$/)
  if (angleMatch) {
    const name = (angleMatch[1] ?? angleMatch[2] ?? '').trim()
    const email = angleMatch[3].trim()
    return name === '' ? { email } : { name, email }
  }

  // Legacy RFC 5322 comment form: `user@example.com (Display Name)`.
  // Matched *before* the bare-email fallback so the `(Name)` suffix does
  // not leak into the email field.
  const commentMatch = trimmed.match(/^(\S+@\S+?)\s*\(([^)]*)\)\s*$/)
  if (commentMatch) {
    const email = commentMatch[1].trim()
    const name = commentMatch[2].trim()
    return name === '' ? { email } : { name, email }
  }

  // Bare email — no angle brackets, no display name.
  if (trimmed.includes('@')) {
    return { email: trimmed }
  }

  return undefined
}

/** Parse a comma-separated address list, respecting quotes and angle brackets. */
function parseAddressList(raw) {
  if (typeof raw !== 'string') return undefined
  const parts = splitAddressList(raw)
  const addresses = parts
    .map(parseAddress)
    .filter((a) => a !== undefined)
  return addresses.length > 0 ? addresses : undefined
}

function splitAddressList(raw) {
  // Split on commas that are NOT inside quotes or angle brackets. This
  // keeps `"Lastname, Firstname" <x@y>` intact.
  const out = []
  let buf = ''
  let inQuotes = false
  let angleDepth = 0

  for (let i = 0; i < raw.length; i++) {
    const ch = raw[i]
    if (ch === '"' && raw[i - 1] !== '\\') {
      inQuotes = !inQuotes
      buf += ch
    } else if (!inQuotes && ch === '<') {
      angleDepth++
      buf += ch
    } else if (!inQuotes && ch === '>') {
      if (angleDepth > 0) angleDepth--
      buf += ch
    } else if (!inQuotes && angleDepth === 0 && ch === ',') {
      if (buf.trim() !== '') out.push(buf)
      buf = ''
    } else {
      buf += ch
    }
  }
  if (buf.trim() !== '') out.push(buf)
  return out
}

// ---------------------------------------------------------------------------
// Other header helpers
// ---------------------------------------------------------------------------

function parseDate(raw) {
  if (typeof raw !== 'string' || raw.trim() === '') return undefined
  const t = Date.parse(raw)
  if (Number.isNaN(t)) return undefined
  // Match the Rust path's ISO-8601 format (no fractional seconds) so the
  // `date` field is byte-identical across `preprocess` and `preprocessGmail`.
  return new Date(t).toISOString().replace(/\.000Z$/, 'Z')
}

function stripAngleBrackets(s) {
  if (typeof s !== 'string') return s
  const trimmed = s.trim()
  // Match a single `<...>` with no embedded `>` — safe even when a caller
  // hands us a string containing multiple angle-bracketed tokens.
  const m = trimmed.match(/^<([^>]+)>$/)
  return m ? m[1] : trimmed
}

function splitIdList(raw) {
  // References (and, less commonly, In-Reply-To) may contain multiple IDs
  // separated by whitespace. Split on any run of whitespace.
  return raw.split(/\s+/).filter((s) => s.length > 0)
}

module.exports = { preprocessGmail }
