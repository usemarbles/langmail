//! Gmail adapter — normalizes a `gmail_v1.Schema$Message` JSON payload
//! into langmail's shared cleaning pipeline.
//!
//! Accepts either the raw `Schema$Message` object or the full googleapis
//! response (`{ data: Schema$Message, status, ... }`). Requires
//! `format: "full"` so `payload` is present with headers and
//! base64url-encoded body parts.
//!
//! Body selection: walks `payload.parts` depth-first and picks the first
//! non-attachment leaf of each type. When both `text/html` and
//! `text/plain` are present, HTML wins — it's converted to Markdown
//! inside the shared pipeline before quote/signature stripping. Parts
//! with `Content-Disposition: attachment` or a `filename` are skipped.
//!
//! This module lives in Rust so every binding (Node, Python, …) inherits
//! Gmail support without re-implementing provider logic.

use std::collections::HashMap;

use base64::Engine;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use serde_json::Value;

use crate::types::{Address, LangmailError, ParsedInput, PreprocessOptions, ProcessedEmail};

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// Preprocess a Gmail API message (JSON string) through langmail's cleaning
/// pipeline, with default options.
///
/// `msg_json` is the JSON-serialized Gmail `Schema$Message` — either the
/// bare message or the full googleapis response (`{ data: {...}, ... }`).
pub fn preprocess_gmail(msg_json: &str) -> Result<ProcessedEmail, LangmailError> {
    preprocess_gmail_with_options(msg_json, &PreprocessOptions::default())
}

/// Preprocess a Gmail API message with custom [`PreprocessOptions`].
pub fn preprocess_gmail_with_options(
    msg_json: &str,
    options: &PreprocessOptions,
) -> Result<ProcessedEmail, LangmailError> {
    let value: Value = serde_json::from_str(msg_json)
        .map_err(|e| LangmailError::InvalidGmailMessage(format!("invalid JSON: {}", e)))?;

    // Unwrap googleapis response shape: `{ data: <message>, status, ... }`.
    // Only take `.data` when it's an object AND has a `payload` — otherwise
    // a top-level message that happens to have a `data` attachment field
    // would be mis-detected.
    let message_value = match value.get("data") {
        Some(data) if data.is_object() && data.get("payload").is_some_and(|p| p.is_object()) => {
            data
        }
        _ => &value,
    };

    if !message_value.is_object() {
        return Err(LangmailError::InvalidGmailMessage(
            "expected a Gmail message object".to_string(),
        ));
    }

    let message: GmailMessage = serde_json::from_value(message_value.clone()).map_err(|e| {
        LangmailError::InvalidGmailMessage(format!("invalid Gmail message shape: {}", e))
    })?;

    let payload = message.payload.ok_or_else(|| {
        LangmailError::InvalidGmailMessage(
            "message.payload is missing. Did you fetch with format: \"full\"?".to_string(),
        )
    })?;

    let (html, text) = extract_bodies(&payload)?;
    let headers = normalize_headers(payload.headers.as_deref().unwrap_or(&[]));

    let input = ParsedInput {
        html,
        text,
        subject: headers.get("subject").cloned(),
        from: headers.get("from").and_then(|v| parse_address(v)),
        to: headers
            .get("to")
            .map(|v| parse_address_list(v))
            .unwrap_or_default(),
        cc: headers
            .get("cc")
            .map(|v| parse_address_list(v))
            .unwrap_or_default(),
        date: headers.get("date").and_then(|v| parse_rfc2822_date(v)),
        rfc_message_id: headers.get("message-id").map(|v| strip_angle_brackets(v)),
        in_reply_to: headers.get("in-reply-to").and_then(|v| parse_id_list(v)),
        references: headers.get("references").and_then(|v| parse_id_list(v)),
    };

    Ok(crate::preprocess_parsed(input, options))
}

// ---------------------------------------------------------------------------
// JSON shape (private — structural match against Schema$Message)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct GmailMessage {
    payload: Option<GmailPart>,
}

#[derive(Debug, Deserialize)]
struct GmailPart {
    #[serde(rename = "mimeType")]
    mime_type: Option<String>,
    #[serde(default)]
    filename: Option<String>,
    #[serde(default)]
    headers: Option<Vec<GmailHeader>>,
    #[serde(default)]
    body: Option<GmailBody>,
    #[serde(default)]
    parts: Option<Vec<GmailPart>>,
}

#[derive(Debug, Deserialize)]
struct GmailHeader {
    name: Option<String>,
    value: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GmailBody {
    #[serde(default)]
    data: Option<String>,
    #[serde(default, rename = "attachmentId")]
    attachment_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Body extraction
// ---------------------------------------------------------------------------

/// Walk the part tree depth-first picking the first non-attachment
/// `text/html` leaf and the first non-attachment `text/plain` leaf.
///
/// Errors when a candidate leaf has `body.attachmentId` instead of inline
/// `data` — Gmail does this for bodies over the inline size threshold,
/// and silent fall-through would yield an empty body; surfacing the
/// error lets the caller fetch the part via
/// `users.messages.attachments.get`.
fn extract_bodies(payload: &GmailPart) -> Result<(Option<String>, Option<String>), LangmailError> {
    let mut html: Option<String> = None;
    let mut text: Option<String> = None;
    visit_part(payload, &mut html, &mut text)?;
    Ok((html, text))
}

fn visit_part(
    part: &GmailPart,
    html: &mut Option<String>,
    text: &mut Option<String>,
) -> Result<(), LangmailError> {
    let mime = part
        .mime_type
        .as_deref()
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();

    let is_attachment = is_attachment_part(part);
    let is_html_candidate = !is_attachment && mime == "text/html" && html.is_none();
    let is_text_candidate = !is_attachment && mime == "text/plain" && text.is_none();

    if is_html_candidate || is_text_candidate {
        if let Some(body) = &part.body {
            if let Some(data) = body.data.as_deref().filter(|s| !s.is_empty()) {
                let decoded = decode_base64url(data);
                if is_html_candidate {
                    *html = Some(decoded);
                } else if is_text_candidate {
                    *text = Some(decoded);
                }
            } else if let Some(att_id) = body.attachment_id.as_deref().filter(|s| !s.is_empty()) {
                return Err(LangmailError::BodyRequiresAttachmentFetch {
                    mime_type: mime.clone(),
                    attachment_id: att_id.to_string(),
                });
            }
        }
    }

    if let Some(parts) = &part.parts {
        for child in parts {
            visit_part(child, html, text)?;
        }
    }
    Ok(())
}

fn is_attachment_part(part: &GmailPart) -> bool {
    if part.filename.as_deref().is_some_and(|s| !s.is_empty()) {
        return true;
    }
    if let Some(headers) = &part.headers {
        for h in headers {
            if let (Some(name), Some(value)) = (h.name.as_deref(), h.value.as_deref()) {
                if name.eq_ignore_ascii_case("content-disposition")
                    && value
                        .trim_start()
                        .to_ascii_lowercase()
                        .starts_with("attachment")
                {
                    return true;
                }
            }
        }
    }
    false
}

/// Decode Gmail's base64url body.
///
/// Gmail emits RFC 4648 §5 (URL-safe alphabet, no padding). We strip any
/// whitespace and trailing `=` padding then decode with the no-padding
/// engine so we accept both strictly-padded and unpadded inputs.
///
/// Invalid base64 is coerced to an empty string rather than erroring —
/// the JS adapter silently produced mojibake on corrupt data, and a
/// provider returning malformed base64 is not something callers can act
/// on.
fn decode_base64url(data: &str) -> String {
    let cleaned: String = data.chars().filter(|c| !c.is_whitespace()).collect();
    let trimmed = cleaned.trim_end_matches('=');
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(trimmed)
        .unwrap_or_default();
    String::from_utf8(bytes).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Header normalization
// ---------------------------------------------------------------------------

/// Flatten Gmail's `[{name, value}, …]` header array into a case-insensitive
/// lookup map. Multiple occurrences use last-wins, matching mail-parser's
/// behavior for the headers we consume (Subject, From, Date, Message-ID).
fn normalize_headers(headers: &[GmailHeader]) -> HashMap<String, String> {
    let mut map = HashMap::with_capacity(headers.len());
    for h in headers {
        if let (Some(name), Some(value)) = (h.name.as_deref(), h.value.as_deref()) {
            map.insert(name.to_ascii_lowercase(), value.to_string());
        }
    }
    map
}

// ---------------------------------------------------------------------------
// Address parsing
// ---------------------------------------------------------------------------

// `"Display Name" <user@example.com>` or `Display Name <user@example.com>`
static ANGLE_ADDRESS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^(?:"([^"]*)"|([^<]*?))\s*<([^>]+)>\s*$"#).unwrap());

// Legacy RFC 5322 comment form: `user@example.com (Display Name)`
static COMMENT_ADDRESS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\S+@\S+?)\s*\(([^)]*)\)\s*$").unwrap());

/// Parse a single `"Name" <email>`, `email (Name)`, or bare `email` into an
/// [`Address`]. Returns `None` for empty or non-email input.
fn parse_address(raw: &str) -> Option<Address> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(caps) = ANGLE_ADDRESS.captures(trimmed) {
        let name = caps
            .get(1)
            .or_else(|| caps.get(2))
            .map(|m| m.as_str().trim())
            .unwrap_or("");
        let email = caps
            .get(3)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        return Some(Address {
            name: if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            },
            email,
        });
    }

    if let Some(caps) = COMMENT_ADDRESS.captures(trimmed) {
        let email = caps
            .get(1)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let name = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
        return Some(Address {
            name: if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            },
            email,
        });
    }

    // Bare email — no angle brackets, no display name.
    if trimmed.contains('@') {
        return Some(Address {
            name: None,
            email: trimmed.to_string(),
        });
    }

    None
}

/// Parse a comma-separated address list, respecting quoted display names
/// and angle-bracketed email addresses.
fn parse_address_list(raw: &str) -> Vec<Address> {
    split_address_list(raw)
        .into_iter()
        .filter_map(parse_address)
        .collect()
}

/// Split on commas that are not inside quotes or angle brackets. Keeps
/// `"Lastname, Firstname" <x@y>` intact as a single token.
fn split_address_list(raw: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut in_quotes = false;
    let mut angle_depth: i32 = 0;
    let mut prev: char = '\0';

    for (idx, ch) in raw.char_indices() {
        match ch {
            '"' if prev != '\\' => in_quotes = !in_quotes,
            '<' if !in_quotes => angle_depth += 1,
            '>' if !in_quotes && angle_depth > 0 => angle_depth -= 1,
            ',' if !in_quotes && angle_depth == 0 => {
                let slice = &raw[start..idx];
                if !slice.trim().is_empty() {
                    out.push(slice);
                }
                start = idx + ch.len_utf8();
            }
            _ => {}
        }
        prev = ch;
    }
    let tail = &raw[start..];
    if !tail.trim().is_empty() {
        out.push(tail);
    }
    out
}

// ---------------------------------------------------------------------------
// Date parsing (RFC 2822 → ISO 8601 UTC, no fractional seconds)
// ---------------------------------------------------------------------------

/// Parse an RFC 2822 `Date:` header value into an ISO 8601 UTC string
/// matching the MIME path's format (`YYYY-MM-DDTHH:MM:SSZ`).
///
/// Returns `None` for missing, blank, or unparseable inputs — callers
/// treat `None` as "no date", which matches the JS adapter's behavior
/// when `Date.parse()` returns `NaN`.
fn parse_rfc2822_date(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Optional `Day,` prefix (e.g. "Thu, 05 Feb 2026 ...").
    let after_dow = match trimmed.find(',') {
        Some(i) if trimmed[..i].chars().all(|c| c.is_ascii_alphabetic()) => trimmed[i + 1..].trim(),
        _ => trimmed,
    };

    let tokens: Vec<&str> = after_dow.split_ascii_whitespace().collect();
    if tokens.len() < 5 {
        return None;
    }

    let day: u32 = tokens[0].parse().ok()?;
    let month: u32 = match tokens[1] {
        "Jan" => 1,
        "Feb" => 2,
        "Mar" => 3,
        "Apr" => 4,
        "May" => 5,
        "Jun" => 6,
        "Jul" => 7,
        "Aug" => 8,
        "Sep" => 9,
        "Oct" => 10,
        "Nov" => 11,
        "Dec" => 12,
        _ => return None,
    };
    let year_raw: i32 = tokens[2].parse().ok()?;
    let year: i32 = if year_raw < 50 {
        year_raw + 2000
    } else if year_raw < 100 {
        year_raw + 1900
    } else {
        year_raw
    };

    let time_parts: Vec<&str> = tokens[3].split(':').collect();
    if time_parts.len() < 2 {
        return None;
    }
    let hour: u32 = time_parts[0].parse().ok()?;
    let minute: u32 = time_parts[1].parse().ok()?;
    let second: u32 = if time_parts.len() >= 3 {
        time_parts[2].parse().ok()?
    } else {
        0
    };

    let tz_offset_minutes = parse_timezone(tokens[4])?;

    let days = days_from_civil(year as i64, month as i64, day as i64);
    let ts = days * 86_400 + hour as i64 * 3_600 + minute as i64 * 60 + second as i64
        - tz_offset_minutes as i64 * 60;

    Some(crate::timestamp_to_iso8601_utc(ts))
}

fn parse_timezone(tz: &str) -> Option<i32> {
    match tz {
        "GMT" | "UT" | "UTC" | "Z" => Some(0),
        "EDT" => Some(-240),
        "EST" | "CDT" => Some(-300),
        "CST" | "MDT" => Some(-360),
        "MST" | "PDT" => Some(-420),
        "PST" => Some(-480),
        _ => {
            let (sign, digits) = if let Some(rest) = tz.strip_prefix('+') {
                (1i32, rest)
            } else if let Some(rest) = tz.strip_prefix('-') {
                (-1i32, rest)
            } else {
                return None;
            };
            if digits.len() != 4 || !digits.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
            let hh: i32 = digits[..2].parse().ok()?;
            let mm: i32 = digits[2..].parse().ok()?;
            Some(sign * (hh * 60 + mm))
        }
    }
}

/// Howard Hinnant's civil-date → days-since-Unix-epoch algorithm.
/// See <http://howardhinnant.github.io/date_algorithms.html>.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400; // [0, 399]
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146_097 + doe - 719_468
}

// ---------------------------------------------------------------------------
// Message-ID / reference list parsing
// ---------------------------------------------------------------------------

/// Strip the surrounding `<…>` from a single Message-ID. A safe no-op when
/// the value doesn't match a single pair — matches the JS behavior of
/// only stripping exact `^<[^>]+>$` wrappers.
fn strip_angle_brackets(s: &str) -> String {
    let trimmed = s.trim();
    if let Some(inner) = trimmed.strip_prefix('<') {
        if let Some(inner) = inner.strip_suffix('>') {
            if !inner.contains('>') {
                return inner.to_string();
            }
        }
    }
    trimmed.to_string()
}

/// Parse an `In-Reply-To` / `References` header into a list of bare
/// Message-IDs. Returns `None` for blank or no-`@` input so downstream
/// callers can distinguish "absent" from "empty". Filters out trailing
/// `(comment)` fragments — a bare Message-ID always contains `@`.
fn parse_id_list(raw: &str) -> Option<Vec<String>> {
    let ids: Vec<String> = raw
        .split_ascii_whitespace()
        .map(strip_angle_brackets)
        .filter(|s| s.contains('@'))
        .collect();
    if ids.is_empty() {
        None
    } else {
        Some(ids)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_quoted_display_name() {
        let a = parse_address(r#""Carol, Support" <carol@example.com>"#).unwrap();
        assert_eq!(a.name.as_deref(), Some("Carol, Support"));
        assert_eq!(a.email, "carol@example.com");
    }

    #[test]
    fn parses_unquoted_display_name() {
        let a = parse_address("Alice <alice@example.com>").unwrap();
        assert_eq!(a.name.as_deref(), Some("Alice"));
        assert_eq!(a.email, "alice@example.com");
    }

    #[test]
    fn parses_comment_form() {
        let a = parse_address("alice@example.com (Alice Example)").unwrap();
        assert_eq!(a.name.as_deref(), Some("Alice Example"));
        assert_eq!(a.email, "alice@example.com");
    }

    #[test]
    fn parses_bare_email() {
        let a = parse_address("alice@example.com").unwrap();
        assert_eq!(a.name, None);
        assert_eq!(a.email, "alice@example.com");
    }

    #[test]
    fn parse_address_empty_returns_none() {
        assert!(parse_address("").is_none());
        assert!(parse_address("   ").is_none());
    }

    #[test]
    fn splits_address_list_respecting_quotes() {
        let list = parse_address_list(r#""Carol, Support" <carol@example.com>, dev@example.com"#);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name.as_deref(), Some("Carol, Support"));
        assert_eq!(list[1].email, "dev@example.com");
        assert_eq!(list[1].name, None);
    }

    #[test]
    fn rfc2822_date_positive_offset() {
        let iso = parse_rfc2822_date("Thu, 05 Feb 2026 10:00:00 +0000").unwrap();
        assert_eq!(iso, "2026-02-05T10:00:00Z");
    }

    #[test]
    fn rfc2822_date_negative_offset_shifts_to_utc() {
        // 10:00 in UTC-0500 is 15:00 UTC.
        let iso = parse_rfc2822_date("Mon, 12 Jan 2026 10:00:00 -0500").unwrap();
        assert_eq!(iso, "2026-01-12T15:00:00Z");
    }

    #[test]
    fn rfc2822_date_gmt_alias() {
        let iso = parse_rfc2822_date("Thu, 05 Feb 2026 10:00:00 GMT").unwrap();
        assert_eq!(iso, "2026-02-05T10:00:00Z");
    }

    #[test]
    fn rfc2822_date_malformed_returns_none() {
        assert!(parse_rfc2822_date("not a real date").is_none());
        assert!(parse_rfc2822_date("").is_none());
    }

    #[test]
    fn strip_angle_brackets_simple() {
        assert_eq!(strip_angle_brackets("<abc@example.com>"), "abc@example.com");
        assert_eq!(
            strip_angle_brackets("  <abc@example.com>  "),
            "abc@example.com"
        );
    }

    #[test]
    fn strip_angle_brackets_no_wrap() {
        assert_eq!(strip_angle_brackets("abc@example.com"), "abc@example.com");
    }

    #[test]
    fn parse_id_list_splits_and_strips() {
        let ids = parse_id_list("<root@example.com> <abc@example.com>").unwrap();
        assert_eq!(ids, vec!["root@example.com", "abc@example.com"]);
    }

    #[test]
    fn parse_id_list_drops_non_email_tokens() {
        let ids =
            parse_id_list("<root@example.com> (imported from archive) <abc@example.com>").unwrap();
        assert_eq!(ids, vec!["root@example.com", "abc@example.com"]);
    }

    #[test]
    fn parse_id_list_blank_is_none() {
        assert!(parse_id_list("   ").is_none());
        assert!(parse_id_list("").is_none());
    }

    #[test]
    fn decode_base64url_roundtrip() {
        // "Hello world" → "SGVsbG8gd29ybGQ"
        assert_eq!(decode_base64url("SGVsbG8gd29ybGQ"), "Hello world");
    }

    #[test]
    fn decode_base64url_url_safe_chars() {
        // Input uses `-` and `_` — URL-safe alphabet.
        let html = decode_base64url("PGh0bWw-PGJvZHk-PC9ib2R5PjwvaHRtbD4");
        assert_eq!(html, "<html><body></body></html>");
    }
}
