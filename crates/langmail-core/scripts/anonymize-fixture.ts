#!/usr/bin/env npx tsx
/**
 * anonymize-fixture.ts
 *
 * Anonymizes raw email (.eml) files for use as test fixtures.
 * Scrubs PII from headers, URLs, and body content while preserving
 * structure that matters for parsing (MIME, HTML layout, encoding).
 *
 * Note: URL anonymization is primarily targeted at LinkedIn emails.
 * For non-LinkedIn emails, only email addresses and sender/recipient
 * names (extracted from headers) are anonymized automatically.
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

/** Remove quoted-printable soft line breaks so regexes can match full URLs. */
function undoQPSoftBreaks(text: string): string {
  return text.replace(/=\r?\n/g, "");
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
  "feedback-id",
  "require-recipient-valid-since",
  "return-path",
]);

/** Headers whose values contain PII that needs targeted replacement. */
function anonymizeHeaderValue(
  name: string,
  value: string,
  senderFirst: string,
  senderLast: string
): string {
  const lower = name.toLowerCase();

  if (lower === "delivered-to" || lower === "to") {
    return value.replace(/[^\s<>@,]+@[^\s<>@,]+/g, REPLACEMENTS.email)
                .replace(/([^,<]+?)(\s*<)/g, `${REPLACEMENTS.fullName} $2`);
  }

  if (lower === "from") {
    let v = value.replace(/[^\s<>@,]+@[^\s<>@,]+/g, `noreply@example.com`);
    // Replace sender name while preserving "via Service" suffix
    if (senderFirst && senderLast) {
      v = v.replace(new RegExp(escapeRegex(`${senderFirst} ${senderLast}`), "gi"), REPLACEMENTS.fullName);
    } else if (senderFirst) {
      v = v.replace(new RegExp(`\\b${escapeRegex(senderFirst)}\\b`, "g"), REPLACEMENTS.firstName);
    }
    return v;
  }

  if (lower === "subject") {
    let v = value;
    if (senderFirst) {
      v = v.replace(new RegExp(`\\b${escapeRegex(senderFirst)}\\b`, "g"), REPLACEMENTS.firstName);
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

function anonymizeUrls(text: string): string {
  // LinkedIn messaging thread URLs → canonical test URL
  text = text.replace(
    /https?:\/\/(?:[\w-]+\.)?linkedin\.com\/comm\/messaging\/thread\/[^\s"'>)]+/g,
    REPLACEMENTS.messageThreadUrl
  );

  // LinkedIn tracking pixel (emimp)
  text = text.replace(
    /https?:\/\/(?:[\w-]+\.)?linkedin\.com\/emimp\/[^\s"'>)]+/g,
    REPLACEMENTS.trackingPixelUrl
  );

  // LinkedIn profile image CDN URLs
  text = text.replace(
    /https?:\/\/media\.licdn\.com\/dms\/image\/[^\s"'>)]+/g,
    REPLACEMENTS.profileImageUrl
  );

  // LinkedIn static asset URLs (logos etc.) — keep structurally but genericize
  text = text.replace(
    /https?:\/\/static\.licdn\.com\/[^\s"'>)]+/g,
    "https://example.com/static/image.png"
  );

  // LinkedIn unsubscribe / psettings URLs
  text = text.replace(
    /https?:\/\/(?:[\w-]+\.)?linkedin\.com\/comm\/psettings\/[^\s"'>)]+/g,
    REPLACEMENTS.unsubscribeUrl
  );

  // LinkedIn help URLs
  text = text.replace(
    /https?:\/\/(?:www\.)?linkedin\.com\/help\/[^\s"'>)]+/g,
    REPLACEMENTS.helpUrl
  );

  // LinkedIn profile URLs  (in/username pattern)
  text = text.replace(
    /https?:\/\/(?:[\w-]+\.)?linkedin\.com\/comm\/in\/[^\s"'>)]+/g,
    "https://www.linkedin.com/in/test-user"
  );

  // LinkedIn feed / generic comm URLs
  text = text.replace(
    /https?:\/\/(?:[\w-]+\.)?linkedin\.com\/comm\/[^\s"'>)]+/g,
    REPLACEMENTS.genericUrl
  );

  // Any remaining linkedin.com URLs
  text = text.replace(
    /https?:\/\/(?:[\w-]+\.)?linkedin\.com\/[^\s"'>)]+/g,
    REPLACEMENTS.genericUrl
  );

  return text;
}

function anonymizeNames(text: string, realFirst: string, realLast: string): string {
  const fullName = `${realFirst} ${realLast}`.trim();
  if (fullName) {
    text = text.replace(new RegExp(escapeRegex(fullName), "gi"), REPLACEMENTS.fullName);
  }
  if (realFirst) {
    text = text.replace(new RegExp(`\\b${escapeRegex(realFirst)}\\b`, "g"), REPLACEMENTS.firstName);
  }
  if (realLast) {
    text = text.replace(new RegExp(`\\b${escapeRegex(realLast)}\\b`, "g"), REPLACEMENTS.lastName);
  }
  return text;
}

function anonymizeEmails(text: string): string {
  return text.replace(/[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}/g, REPLACEMENTS.email);
}

function anonymizeLinkedInIds(text: string): string {
  // midToken, midSig, otpToken, eid, loid query params
  // Handle both plain & and HTML &amp; as separators, and =3D (QP-encoded =)
  text = text.replace(
    /([?&]|&amp;)(midToken|midSig|otpToken|eid|loid)(=3D|=)([^&\s"'>)]+)/g,
    "$1$2$3REDACTED"
  );
  // lipi URN values
  text = text.replace(/lipi(=3D|=)[^&\s"'>)]+/g, "lipi$1REDACTED");
  // trk / trkEmail params
  text = text.replace(/(?:trkEmail|trk)(=3D|=)[^&\s"'>)]+/g, "trk$1REDACTED");
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
  recipientName: string
): string {
  let body = rawBody;
  body = anonymizeUrls(body);
  body = anonymizeEmails(body);
  body = anonymizeLinkedInIds(body);
  body = anonymizeNames(body, senderFirst, senderLast);

  // Recipient name scrub
  if (recipientName) {
    const [rFirst, ...rRest] = recipientName.split(" ");
    const rLast = rRest.join(" ");
    body = anonymizeNames(body, rFirst, rLast);
  }

  // NOTE: Free-text content like headlines, taglines, and other profile data
  // cannot be reliably detected automatically. After running this script,
  // manually review the output for any remaining PII in body text.

  return body;
}

// ---------------------------------------------------------------------------
// Main processing
// ---------------------------------------------------------------------------

function extractSenderInfo(headers: Array<{ name: string; value: string; raw: string }>): {
  firstName: string;
  lastName: string;
} {
  // Try From: "First Last via LinkedIn" first — gives full name
  const from = headers.find((h) => h.name.toLowerCase() === "from")?.value ?? "";
  const fromMatch = from.match(/^([A-Z][a-z]+)\s+([A-Z][a-z]+)\s+via/);
  if (fromMatch) {
    return { firstName: fromMatch[1], lastName: fromMatch[2] };
  }
  // Fall back to Subject: "Elias just messaged you" — first name only
  const subject = headers.find((h) => h.name.toLowerCase() === "subject")?.value ?? "";
  const subjectMatch = subject.match(/^(\w+)\s+just messaged/i);
  if (subjectMatch) {
    return { firstName: subjectMatch[1], lastName: "" };
  }
  return { firstName: "", lastName: "" };
}

function extractRecipientName(
  headers: Array<{ name: string; value: string; raw: string }>
): string {
  const to = headers.find((h) => h.name.toLowerCase() === "to")?.value ?? "";
  const m = to.match(/^([^<]+)</);
  return m ? m[1].trim() : "";
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
        const bodyToProcess = isQP ? undoQPSoftBreaks(partBody) : partBody;
        const anonymized = anonymizeBody(bodyToProcess, partCT, senderFirst, senderLast, recipientName);
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
