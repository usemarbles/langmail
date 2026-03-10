#!/usr/bin/env npx tsx
/**
 * anonymize-fixture.ts
 *
 * Anonymizes raw email (.eml) files for use as test fixtures.
 * Scrubs PII from headers, URLs, and body content while preserving
 * structure that matters for parsing (MIME, HTML layout, encoding).
 *
 * Handles both LinkedIn and generic emails:
 * - LinkedIn: anonymizes tracking URLs, profile URLs, CDN image URLs, etc.
 * - Generic: extracts sender name from From header, derives recipient name
 *   from To header (display name or email local part).
 * Always manually review the output for any remaining PII in free-text
 * content such as headlines, taglines, or other profile data.
 *
 * Usage:
 *   npx tsx anonymize-fixture.ts input.eml output.eml
 *   npx tsx anonymize-fixture.ts input.eml   # writes input.anon.eml
 */

import * as fs from "fs";
import * as path from "path";

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

function escapeRegex(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/** Build a regex pattern with Unicode-aware word boundaries (unlike \b, works with accented chars). */
function withUnicodeBounds(pattern: string): string {
  return `(?<![\\p{L}\\p{N}])${pattern}(?![\\p{L}\\p{N}])`;
}

/** Remove quoted-printable soft line breaks so regexes can match full URLs. */
function undoQPSoftBreaks(text: string): string {
  return text.replace(/=\r?\n/g, "");
}

/**
 * Build a mapping from unwrapped-text positions to wrapped-text positions.
 * map[i] = position in wrapped text of the i-th character of the unwrapped text.
 * A sentinel at map[unwrapped.length] points past the last character.
 */
function buildPositionMap(wrappedText: string): number[] {
  const map: number[] = [];
  let wi = 0;
  while (wi < wrappedText.length) {
    if (
      wrappedText[wi] === "=" &&
      wi + 1 < wrappedText.length &&
      (wrappedText[wi + 1] === "\n" ||
        (wrappedText[wi + 1] === "\r" && wi + 2 < wrappedText.length && wrappedText[wi + 2] === "\n"))
    ) {
      wi += wrappedText[wi + 1] === "\r" ? 3 : 2;
    } else {
      map.push(wi);
      wi++;
    }
  }
  map.push(wi); // sentinel
  return map;
}

/**
 * Run a regex replacement that works across QP soft line breaks.
 * Matches against the unwrapped text but applies replacements to the wrapped
 * text, so =\n breaks outside matched regions are preserved exactly.
 */
function replaceInQP(text: string, pattern: RegExp, replacement: string): string {
  const unwrapped = undoQPSoftBreaks(text);
  const posMap = buildPositionMap(text);

  const re = new RegExp(pattern.source, pattern.flags);
  const matches: Array<{ wrappedStart: number; wrappedEnd: number; rep: string }> = [];

  let m: RegExpExecArray | null;
  while ((m = re.exec(unwrapped)) !== null) {
    const rep = m[0].replace(
      new RegExp(pattern.source, pattern.flags.replace("g", "")),
      replacement
    );
    matches.push({
      wrappedStart: posMap[m.index],
      wrappedEnd: posMap[m.index + m[0].length],
      rep,
    });
    if (!pattern.global) break;
  }

  if (matches.length === 0) return text;

  // Apply in reverse order so earlier positions remain valid
  let result = text;
  for (let i = matches.length - 1; i >= 0; i--) {
    const { wrappedStart, wrappedEnd, rep } = matches[i];
    result = result.slice(0, wrappedStart) + rep + result.slice(wrappedEnd);
  }
  return result;
}

/** Re-wrap only lines that exceed the QP limit (76 chars). */
function rewrapLongLines(text: string, maxLineLen = 76): string {
  const crlfCount = (text.match(/\r\n/g) ?? []).length;
  const lfOnly = (text.match(/(?<!\r)\n/g) ?? []).length;
  const eol = crlfCount > lfOnly ? "\r\n" : "\n";
  const lines = text.split(eol);
  const result: string[] = [];

  for (const line of lines) {
    if (line.length <= maxLineLen) {
      result.push(line);
      continue;
    }

    let remaining = line;
    while (remaining.length > maxLineLen) {
      let breakAt = maxLineLen - 1;

      if (breakAt >= 1 && remaining[breakAt - 1] === "=") {
        breakAt -= 1;
      } else if (
        breakAt >= 2 &&
        remaining[breakAt - 2] === "=" &&
        /[0-9A-Fa-f]/.test(remaining[breakAt - 1])
      ) {
        breakAt -= 2;
      }

      if (breakAt <= 0) breakAt = 1;
      result.push(remaining.slice(0, breakAt) + "=");
      remaining = remaining.slice(breakAt);
    }
    result.push(remaining);
  }

  return result.join(eol);
}

// ---------------------------------------------------------------------------
// Replacement values
// ---------------------------------------------------------------------------

const REPLACEMENTS = {
  email: "test@example.com",
  emailLocal: "test",
  emailDomain: "example.com",
  firstName: "Max",
  lastName: "Mustermann",
  fullName: "Max Mustermann",
  headline: "Software Engineer at Example Corp",
  recipientHeadline: "Test User (Test Headline)",
  profileImageUrl: "https://example.com/profile.jpg",
  messageThreadUrl: "https://www.linkedin.com/messaging/thread/test-thread-id",
  genericUrl: "https://example.com/",
  trackingPixelUrl: "https://example.com/tracking.gif",
  unsubscribeUrl: "https://example.com/unsubscribe",
  helpUrl: "https://example.com/help",
  messageId: "<test-message-id@example.com>",
  ipAddress: "192.0.2.1", // TEST-NET per RFC 5737
  bounceAddress: "bounce@example.com",
};

// ---------------------------------------------------------------------------
// Header-level transformations
// ---------------------------------------------------------------------------

/**
 * Headers to strip entirely (security / tracking / routing noise).
 * Everything in this list is replaced with nothing.
 */
const STRIP_HEADERS = new Set([
  "received",
  "x-google-smtp-source",
  "x-received",
  "arc-seal",
  "arc-message-signature",
  "arc-authentication-results",
  "authentication-results",
  "received-spf",
  "dkim-signature",
  "x-linkedin-fbl",
  "x-linkedin-id",
  "x-google-dkim-signature",
  "x-gm-message-state",
  "x-gm-gg",
  "x-gm-features",
  "feedback-id",
  "require-recipient-valid-since",
  "return-path",
  "sender",
  "x-google-sender-delegation",
  "x-google-sender-auth",
]);

/** Headers whose values contain PII that needs targeted replacement. */
function anonymizeHeaderValue(
  name: string,
  value: string,
  senderFirst: string,
  senderLast: string
): string {
  const lower = name.toLowerCase();

  if (lower === "delivered-to" || lower === "to" || lower === "cc" || lower === "bcc") {
    return value.replace(/[^\s<>@,]+@[^\s<>@,]+/g, REPLACEMENTS.email)
                .replace(/([^,<]+?)(\s*<)/g, `${REPLACEMENTS.fullName} $2`);
  }

  if (lower === "from") {
    let v = value.replace(/[^\s<>@,]+@[^\s<>@,]+/g, `noreply@example.com`);
    // Replace sender name while preserving "via Service" suffix
    if (senderFirst && senderLast) {
      v = v.replace(new RegExp(escapeRegex(`${senderFirst} ${senderLast}`), "gi"), REPLACEMENTS.fullName);
    } else if (senderFirst) {
      v = v.replace(new RegExp(withUnicodeBounds(escapeRegex(senderFirst)), "giu"), REPLACEMENTS.firstName);
    }
    return v;
  }

  if (lower === "subject") {
    let v = value;
    if (senderFirst) {
      v = v.replace(new RegExp(withUnicodeBounds(escapeRegex(senderFirst)), "giu"), REPLACEMENTS.firstName);
    }
    return v;
  }

  if (lower === "message-id") {
    return REPLACEMENTS.messageId;
  }

  if (lower === "list-unsubscribe") {
    return `<${REPLACEMENTS.unsubscribeUrl}>`;
  }

  return value;
}

// ---------------------------------------------------------------------------
// Body transformations
// ---------------------------------------------------------------------------

/** A text replacer — either plain String.replace or QP-aware replaceInQP. */
type TextReplacer = (text: string, pattern: RegExp, replacement: string) => string;

function defaultReplace(text: string, pattern: RegExp, replacement: string): string {
  return text.replace(pattern, replacement);
}

function anonymizeUrls(text: string, rep: TextReplacer = defaultReplace): string {
  // LinkedIn messaging thread URLs → canonical test URL
  text = rep(
    text,
    /https?:\/\/(?:[\w-]+\.)?linkedin\.com\/comm\/messaging\/thread\/[^\s"'>)]+/g,
    REPLACEMENTS.messageThreadUrl
  );

  // LinkedIn tracking pixel (emimp)
  text = rep(
    text,
    /https?:\/\/(?:[\w-]+\.)?linkedin\.com\/emimp\/[^\s"'>)]+/g,
    REPLACEMENTS.trackingPixelUrl
  );

  // LinkedIn profile image CDN URLs
  text = rep(
    text,
    /https?:\/\/media\.licdn\.com\/dms\/image\/[^\s"'>)]+/g,
    REPLACEMENTS.profileImageUrl
  );

  // LinkedIn static asset URLs (logos etc.) — keep structurally but genericize
  text = rep(
    text,
    /https?:\/\/static\.licdn\.com\/[^\s"'>)]+/g,
    "https://example.com/static/image.png"
  );

  // LinkedIn unsubscribe / psettings URLs
  text = rep(
    text,
    /https?:\/\/(?:[\w-]+\.)?linkedin\.com\/comm\/psettings\/[^\s"'>)]+/g,
    REPLACEMENTS.unsubscribeUrl
  );

  // LinkedIn help URLs
  text = rep(
    text,
    /https?:\/\/(?:www\.)?linkedin\.com\/help\/[^\s"'>)]+/g,
    REPLACEMENTS.helpUrl
  );

  // LinkedIn profile URLs  (in/username pattern)
  text = rep(
    text,
    /https?:\/\/(?:[\w-]+\.)?linkedin\.com\/comm\/in\/[^\s"'>)]+/g,
    "https://www.linkedin.com/in/test-user"
  );

  // LinkedIn feed / generic comm URLs
  text = rep(
    text,
    /https?:\/\/(?:[\w-]+\.)?linkedin\.com\/comm\/[^\s"'>)]+/g,
    REPLACEMENTS.genericUrl
  );

  // Any remaining linkedin.com URLs
  text = rep(
    text,
    /https?:\/\/(?:[\w-]+\.)?linkedin\.com\/[^\s"'>)]+/g,
    REPLACEMENTS.genericUrl
  );

  // Google user content (signature images)
  text = rep(
    text,
    /https?:\/\/lh[0-9]*\.googleusercontent\.com\/[^\s"'>)]+/g,
    "https://example.com/image.png"
  );

  // g.page review URLs
  text = rep(
    text,
    /https?:\/\/g\.page\/[^\s"'>)]+/g,
    REPLACEMENTS.genericUrl
  );

  // meetup.com URLs
  text = rep(
    text,
    /https?:\/\/(?:[\w-]+\.)?meetup\.com\/[^\s"'>)]+/g,
    REPLACEMENTS.genericUrl
  );

  // monday.com form URLs
  text = rep(
    text,
    /https?:\/\/(?:[\w-]+\.)?monday\.com\/[^\s"'>)]+/g,
    REPLACEMENTS.genericUrl
  );

  // Catch-all: any remaining URL not pointing to example* or standard domains
  text = rep(
    text,
    /https?:\/\/(?!(?:www\.)?example[\w-]*\.com\b)(?!(?:www\.)?w3\.org\b)[^\s"'>)]+/g,
    REPLACEMENTS.genericUrl
  );

  return text;
}

function anonymizeNames(text: string, realFirst: string, realLast: string, rep: TextReplacer = defaultReplace): string {
  // Only replace as a full name when both parts are present — avoids replacing a
  // single first name with the two-word "Max Mustermann" placeholder.
  if (realFirst && realLast) {
    const fullName = `${realFirst} ${realLast}`;
    text = rep(text, new RegExp(escapeRegex(fullName), "gi"), REPLACEMENTS.fullName);
  }
  if (realFirst) {
    text = rep(text, new RegExp(withUnicodeBounds(escapeRegex(realFirst)), "giu"), REPLACEMENTS.firstName);
  }
  if (realLast) {
    text = rep(text, new RegExp(withUnicodeBounds(escapeRegex(realLast)), "giu"), REPLACEMENTS.lastName);
  }
  return text;
}

function anonymizeEmails(text: string, rep: TextReplacer = defaultReplace): string {
  return rep(text, /[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}/g, REPLACEMENTS.email);
}

function anonymizeLinkedInIds(text: string, rep: TextReplacer = defaultReplace): string {
  // midToken, midSig, otpToken, eid, loid query params
  // Handle both plain & and HTML &amp; as separators, and =3D (QP-encoded =)
  text = rep(
    text,
    /([?&]|&amp;)(midToken|midSig|otpToken|eid|loid)(=3D|=)([^&\s"'>)]+)/g,
    "$1$2$3REDACTED"
  );
  // lipi URN values
  text = rep(text, /lipi(=3D|=)[^&\s"'>)]+/g, "lipi$1REDACTED");
  // trk / trkEmail params
  text = rep(text, /(?:trkEmail|trk)(=3D|=)[^&\s"'>)]+/g, "trk$1REDACTED");
  return text;
}

// ---------------------------------------------------------------------------
// MIME parser / reconstructor
// ---------------------------------------------------------------------------

function parseHeaders(raw: string): Array<{ name: string; value: string; raw: string }> {
  const headers: Array<{ name: string; value: string; raw: string }> = [];
  // Unfold continuation lines
  const unfolded = raw.replace(/\r?\n([ \t]+)/g, " $1");
  for (const line of unfolded.split(/\r?\n/)) {
    const m = line.match(/^([^:]+):\s*(.*)/);
    if (m) {
      headers.push({ name: m[1], value: m[2], raw: line });
    }
  }
  return headers;
}

function getBoundary(contentType: string): string | undefined {
  const m = contentType.match(/boundary="?([^";\r\n]+)"?/i);
  return m ? m[1] : undefined;
}

function anonymizeBody(
  rawBody: string,
  contentType: string,
  senderFirst: string,
  senderLast: string,
  recipientName: string,
  isQP = false
): string {
  const rep: TextReplacer = isQP ? replaceInQP : defaultReplace;

  let body = rawBody;
  body = anonymizeUrls(body, rep);
  body = anonymizeEmails(body, rep);
  body = anonymizeLinkedInIds(body, rep);
  body = anonymizeNames(body, senderFirst, senderLast, rep);

  // Recipient name scrub
  if (recipientName) {
    const [rFirst, ...rRest] = recipientName.split(" ");
    const rLast = rRest.join(" ");
    body = anonymizeNames(body, rFirst, rLast, rep);
  }

  // Safety: re-wrap any lines that ended up > 76 chars after replacement
  if (isQP) {
    body = rewrapLongLines(body);
  }

  return body;
}

// ---------------------------------------------------------------------------
// Main processing
// ---------------------------------------------------------------------------

function extractSenderInfo(headers: Array<{ name: string; value: string; raw: string }>): {
  firstName: string;
  lastName: string;
} {
  const from = headers.find((h) => h.name.toLowerCase() === "from")?.value ?? "";

  // Try From: "First Last via LinkedIn" — gives full name with service suffix
  const viaMatch = from.match(/^([A-Z][a-z]+)\s+([A-Z][a-z]+)\s+via/);
  if (viaMatch) {
    return { firstName: viaMatch[1], lastName: viaMatch[2] };
  }

  // Fall back to Subject: "Elias just messaged you" — first name only
  const subject = headers.find((h) => h.name.toLowerCase() === "subject")?.value ?? "";
  const subjectMatch = subject.match(/^(\w+)\s+just messaged/i);
  if (subjectMatch) {
    return { firstName: subjectMatch[1], lastName: "" };
  }

  // Generic From: "Display Name <email@example.com>"
  const displayNameMatch = from.match(/^([^<]+?)\s*</);
  if (displayNameMatch) {
    const displayName = displayNameMatch[1].trim();

    // Detect organizational / non-personal senders and skip name replacement
    const ROLE_ADDRESSES = new Set([
      "noreply", "no-reply", "events", "info", "support", "team", "hello",
      "contact", "office", "booking", "sales", "admin", "press", "marketing",
      "newsletter", "notifications", "service",
    ]);
    const emailMatch = from.match(/<([^@]+)@/);
    const localPart = emailMatch?.[1]?.toLowerCase() ?? "";
    if (ROLE_ADDRESSES.has(localPart)) {
      return { firstName: "", lastName: "" };
    }

    const BUSINESS_WORDS = /\b(?:Events|Team|Support|GmbH|Inc|LLC|Ltd|Corp|Foundation|Services?|Newsletter|News|Notifications|Alerts)\b/i;
    if (BUSINESS_WORDS.test(displayName)) {
      return { firstName: "", lastName: "" };
    }

    // All-caps with digits and length > 2 (e.g. "WERK1")
    if (/^[A-Z0-9]{3,}$/.test(displayName) || displayName.split(/\s+/).some(w => /^[A-Z0-9]{3,}$/.test(w) && /\d/.test(w))) {
      return { firstName: "", lastName: "" };
    }

    const parts = displayName.split(/\s+/);
    if (parts.length >= 2) {
      return { firstName: parts[0], lastName: parts.slice(1).join(" ") };
    } else if (parts.length === 1 && parts[0]) {
      return { firstName: parts[0], lastName: "" };
    }
  }

  return { firstName: "", lastName: "" };
}

function extractRecipientName(
  headers: Array<{ name: string; value: string; raw: string }>
): string {
  const to = headers.find((h) => h.name.toLowerCase() === "to")?.value ?? "";

  // "Display Name <email>" format
  const displayMatch = to.match(/^([^<]+)</);
  if (displayMatch) {
    const name = displayMatch[1].trim();
    if (name) return name;
  }

  // Bare email — derive a name from the local part (e.g. "dominik" → "Dominik")
  const emailMatch = to.match(/^([a-zA-Z0-9._%+-]+)@/);
  if (emailMatch) {
    const parts = emailMatch[1]
      .split(/[._+-]+/)
      .filter(Boolean)
      .map((p) => p.charAt(0).toUpperCase() + p.slice(1).toLowerCase());
    if (parts.length > 0) return parts.join(" ");
  }

  return "";
}

function processEmail(raw: string): string {
  // Split top-level headers from body
  const sep = raw.match(/\r?\n\r?\n/);
  if (!sep || sep.index === undefined) return raw;

  const headerSection = raw.slice(0, sep.index);
  const bodySection = raw.slice(sep.index + sep[0].length);

  const parsedHeaders = parseHeaders(headerSection);
  const { firstName: senderFirst, lastName: senderLast } = extractSenderInfo(parsedHeaders);
  const recipientName = extractRecipientName(parsedHeaders);

  // Rebuild headers
  const outputHeaders: string[] = [];
  const ctHeader = parsedHeaders.find((h) => h.name.toLowerCase() === "content-type");

  for (const h of parsedHeaders) {
    if (STRIP_HEADERS.has(h.name.toLowerCase())) continue;
    const anonymizedValue = anonymizeHeaderValue(h.name, h.value, senderFirst, senderLast);
    outputHeaders.push(`${h.name}: ${anonymizedValue}`);
  }

  // Process body
  const contentType = ctHeader?.value ?? "";
  const boundary = getBoundary(contentType);

  let processedBody: string;

  if (boundary) {
    // Split body into segments separated by boundary delimiters.
    // Segments alternate: preamble, delimiter, part, delimiter, part, ..., closing delimiter, epilogue
    const delimiterPattern = new RegExp(`(^--${escapeRegex(boundary)}(?:--)?[\\t ]*\\r?\\n?)`, "m");
    const segments = bodySection.split(delimiterPattern);

    let result = "";
    let afterClosing = false;
    const boundaryPrefix = "--" + boundary;
    const closingPrefix = boundaryPrefix + "--";
    for (let i = 0; i < segments.length; i++) {
      const seg = segments[i];
      if (seg.startsWith(boundaryPrefix)) {
        // This is a boundary delimiter line — emit as-is
        result += seg;
        if (seg.startsWith(closingPrefix)) {
          afterClosing = true;
        }
      } else if (i === 0 || afterClosing) {
        // Preamble (before first boundary) or epilogue (after closing delimiter)
        result += anonymizeBody(seg, "", senderFirst, senderLast, recipientName);
      } else {
        // MIME part content — split into part headers + body
        const partSep = seg.match(/\r?\n\r?\n/);
        if (!partSep || partSep.index === undefined) {
          result += seg;
          continue;
        }
        const partHeaders = seg.slice(0, partSep.index);
        const partBody = seg.slice(partSep.index + partSep[0].length);
        const partCT = partHeaders.match(/content-type:\s*([^\r\n;]+)/i)?.[1] ?? "";
        const isQP = /content-transfer-encoding:\s*quoted-printable/i.test(partHeaders);
        const anonymized = anonymizeBody(partBody, partCT, senderFirst, senderLast, recipientName, isQP);
        result += partHeaders + partSep[0] + anonymized;
      }
    }

    processedBody = result;
  } else {
    processedBody = anonymizeBody(bodySection, contentType, senderFirst, senderLast, recipientName);
  }

  return outputHeaders.join("\n") + "\n\n" + processedBody;
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

const [, , inputPath, outputPath] = process.argv;

if (!inputPath) {
  console.error("Usage: npx tsx anonymize-fixture.ts <input.eml> [output.eml]");
  process.exit(1);
}

const resolvedInput = path.resolve(inputPath);
if (!fs.existsSync(resolvedInput)) {
  console.error(`Error: file not found: ${resolvedInput}`);
  process.exit(1);
}

const resolvedOutput = outputPath
  ? path.resolve(outputPath)
  : resolvedInput.replace(/(\.[^.]+)?$/, ".anon$1");

const raw = fs.readFileSync(resolvedInput, "utf-8");
const anonymized = processEmail(raw);
fs.writeFileSync(resolvedOutput, anonymized, "utf-8");

console.log(`✓ Anonymized: ${path.basename(resolvedInput)} → ${path.basename(resolvedOutput)}`);
