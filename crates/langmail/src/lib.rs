mod cta;
mod html;
mod quotes;
mod signature;
mod thread;
mod types;

pub use types::*;

use mail_parser::{MessageParser, MimeHeaders};

/// Preprocess a raw email (RFC 5322 / EML format) into an LLM-ready structure.
///
/// This is the primary entry point for langmail. It takes raw email bytes and
/// returns a structured [`ProcessedEmail`] with clean body text, metadata, and
/// thread information — optimized for feeding into language models.
///
/// # Example
/// ```
/// let raw = b"From: alice@example.com\r\nTo: bob@example.com\r\nSubject: Hello\r\n\r\nHi Bob!";
/// let output = langmail::preprocess(raw).unwrap();
/// assert!(output.body.contains("Hi Bob!"));
/// ```
pub fn preprocess(raw: &[u8]) -> Result<ProcessedEmail, LangmailError> {
    preprocess_with_options(raw, &PreprocessOptions::default())
}

/// Preprocess with custom options.
pub fn preprocess_with_options(
    raw: &[u8],
    options: &PreprocessOptions,
) -> Result<ProcessedEmail, LangmailError> {
    let message = MessageParser::default()
        .parse(raw)
        .ok_or(LangmailError::ParseFailed)?;

    let subject = message.subject().map(|s| s.to_string());

    let from = message
        .from()
        .and_then(|addrs| addrs.first())
        .map(|addr| Address {
            name: addr.name().map(|n| n.to_string()),
            email: addr.address().map(|a| a.to_string()).unwrap_or_default(),
        });

    let to = extract_addresses(message.to());
    let cc = extract_addresses(message.cc());

    let date = message.date().map(datetime_to_utc_iso8601);

    let rfc_message_id = message.message_id().map(|id| id.to_string());

    let in_reply_to = message
        .in_reply_to()
        .as_text_list()
        .map(|list| list.iter().map(|s| s.to_string()).collect());

    let references = message
        .references()
        .as_text_list()
        .map(|list| list.iter().map(|s| s.to_string()).collect());

    // Extract raw HTML for CTA detection before text conversion
    let raw_html: Option<String> = if has_html_part(&message) {
        message.body_html(0).map(|h| clean_invisible_characters(&h))
    } else {
        None
    };

    // Extract body: prefer HTML (richer content), fall back to plain text
    let raw_body = extract_body(&message);
    let raw_body = clean_invisible_characters(&raw_body);

    Ok(process_post_parse(
        raw_html,
        raw_body,
        subject,
        from,
        to,
        cc,
        date,
        rfc_message_id,
        in_reply_to,
        references,
        options,
    ))
}

/// Preprocess an already-parsed email, bypassing the MIME parser.
///
/// Accepts decoded HTML/text and pre-parsed headers (as returned by provider
/// APIs like Gmail, Microsoft Graph, Postmark, …) and runs them through the
/// same cleaning pipeline as [`preprocess`].  When `html` is provided it is
/// preferred over `text`; the HTML is converted to Markdown before
/// quote/signature stripping.
pub fn preprocess_parsed(
    input: ParsedInput,
    options: &PreprocessOptions,
) -> Result<ProcessedEmail, LangmailError> {
    // Strip invisible characters from the raw HTML once, up front — the
    // cleaned copy is used for both CTA/thread extraction and markdown
    // conversion, matching the behavior of `preprocess_with_options`.
    let raw_html: Option<String> = input.html.as_deref().map(clean_invisible_characters);

    // Body source: prefer HTML→Markdown, then plain text, then empty.
    let raw_body = if let Some(html) = raw_html.as_deref() {
        html::html_to_markdown(html)
    } else if let Some(text) = input.text.as_deref() {
        text.to_string()
    } else {
        String::new()
    };
    let raw_body = clean_invisible_characters(&raw_body);

    Ok(process_post_parse(
        raw_html,
        raw_body,
        input.subject,
        input.from,
        input.to,
        input.cc,
        input.date,
        input.rfc_message_id,
        input.in_reply_to,
        input.references,
        options,
    ))
}

/// Shared post-parse cleaning pipeline.
///
/// Both [`preprocess_with_options`] (MIME path) and [`preprocess_parsed`]
/// (adapter path) converge here once they have extracted the body and
/// headers.  Keeping this logic in one place guarantees identical output
/// regardless of input shape.
///
/// `raw_html` — already invisible-char-cleaned HTML, or `None` if the source
/// had no HTML part.  Used for CTA detection and thread extraction.
/// `raw_body` — already invisible-char-cleaned body (HTML→Markdown or text).
#[allow(clippy::too_many_arguments)]
fn process_post_parse(
    raw_html: Option<String>,
    raw_body: String,
    subject: Option<String>,
    from: Option<Address>,
    to: Vec<Address>,
    cc: Vec<Address>,
    date: Option<String>,
    rfc_message_id: Option<String>,
    in_reply_to: Option<Vec<String>>,
    references: Option<Vec<String>>,
    options: &PreprocessOptions,
) -> ProcessedEmail {
    let primary_cta = raw_html.as_deref().and_then(cta::extract_cta);

    // Conditionally strip quoted replies
    let body = if options.strip_quotes {
        quotes::strip_quotes(&raw_body)
    } else {
        raw_body.clone()
    };

    // Conditionally strip signature
    let (clean_body, signature) = if options.strip_signature {
        signature::extract_signature(&body)
    } else {
        (body, None)
    };

    let mut body = collapse_empty_lines(&trim_whitespace_lines(clean_body.trim()));

    // Truncate to max_body_length characters (not bytes) on a char boundary
    if options.max_body_length > 0 {
        let char_count = body.chars().count();
        if char_count > options.max_body_length {
            let byte_end = body
                .char_indices()
                .nth(options.max_body_length)
                .map(|(idx, _)| idx)
                .unwrap_or(body.len());
            body.truncate(byte_end);
        }
    }

    // Extract thread messages from HTML blockquotes (for ThreadHistory render mode)
    let thread_messages = raw_html
        .as_deref()
        .map(thread::extract_thread_messages)
        .unwrap_or_default();

    ProcessedEmail {
        clean_body_length: body.len(),
        body,
        subject,
        from,
        to,
        cc,
        date,
        rfc_message_id,
        in_reply_to,
        references,
        signature,
        raw_body_length: raw_body.len(),
        primary_cta,
        thread_messages,
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn has_html_part(message: &mail_parser::Message) -> bool {
    message
        .parts
        .iter()
        .any(|p| p.is_content_type("text", "html"))
}

fn extract_body(message: &mail_parser::Message) -> String {
    // Prefer HTML body when an actual text/html part exists (richer content).
    // mail-parser auto-generates HTML from plain text, so we check for a real part.
    if has_html_part(message) {
        if let Some(html_body) = message.body_html(0) {
            // Strip invisible characters from the HTML before passing to the
            // Markdown converter so they never appear in the output.
            let clean_html = clean_invisible_characters(&html_body);
            return html::html_to_markdown(&clean_html);
        }
    }

    // Fall back to plain text body
    if let Some(text) = message.body_text(0) {
        return text.to_string();
    }

    String::new()
}

/// Invisible characters to remove entirely
const INVISIBLE_CHARS: &[char] = &[
    '\u{034F}', // COMBINING GRAPHEME JOINER
    '\u{200B}', // ZERO WIDTH SPACE
    '\u{200C}', // ZERO WIDTH NON-JOINER
    '\u{200D}', // ZERO WIDTH JOINER
    '\u{FEFF}', // ZERO WIDTH NO-BREAK SPACE (BOM)
    '\u{00AD}', // SOFT HYPHEN
    '\u{2007}', // FIGURE SPACE
];

const ZERO_WIDTH_ENTITIES: &[&str] = &["&zwnj;", "&#x200c;", "&#8204;"];

/// Removes invisible Unicode characters commonly used as email spacers,
/// converts non-breaking spaces to regular spaces, and collapses excessive
/// empty lines.
fn clean_invisible_characters(s: &str) -> String {
    let mut result = s.to_string();
    for entity in ZERO_WIDTH_ENTITIES {
        result = result.replace(entity, "");
    }
    result
        .chars()
        .map(|c| if c == '\u{00A0}' { ' ' } else { c })
        .filter(|c| !INVISIBLE_CHARS.contains(c))
        .collect()
}

/// Trims trailing whitespace from every line, collapsing whitespace-only lines to empty.
pub(crate) fn trim_whitespace_lines(s: &str) -> String {
    s.lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Collapses 3+ consecutive newlines to maximum 2 newlines.
/// Preserves paragraph breaks (2 newlines) but removes excessive spacing.
pub(crate) fn collapse_empty_lines(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut newline_count = 0u32;
    for c in s.chars() {
        if c == '\n' {
            newline_count += 1;
            if newline_count <= 2 {
                result.push(c);
            }
        } else {
            newline_count = 0;
            result.push(c);
        }
    }
    result
}

/// Converts a mail-parser `DateTime` to a UTC ISO 8601 string using its Unix timestamp.
fn datetime_to_utc_iso8601(d: &mail_parser::DateTime) -> String {
    let ts = d.to_timestamp();
    // Decompose Unix timestamp into calendar fields (no external crate needed)
    let secs_per_day = 86400i64;
    let mut days = ts / secs_per_day;
    let mut time = ts % secs_per_day;
    if time < 0 {
        days -= 1;
        time += secs_per_day;
    }
    let hour = time / 3600;
    let minute = (time % 3600) / 60;
    let second = time % 60;

    // Civil date from days since Unix epoch (algorithm from http://howardhinnant.github.io/date_algorithms.html)
    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };

    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn extract_addresses(address_opt: Option<&mail_parser::Address>) -> Vec<Address> {
    match address_opt {
        Some(addresses) => addresses
            .iter()
            .map(|addr| Address {
                name: addr.name().map(|n| n.to_string()),
                email: addr.address().map(|a| a.to_string()).unwrap_or_default(),
            })
            .collect(),
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_email() -> Vec<u8> {
        concat!(
            "From: Alice <alice@example.com>\r\n",
            "To: Bob <bob@example.com>\r\n",
            "Subject: Hello Bob\r\n",
            "Date: Thu, 05 Feb 2026 10:00:00 +0000\r\n",
            "Message-ID: <abc123@example.com>\r\n",
            "Content-Type: text/plain; charset=utf-8\r\n",
            "\r\n",
            "Hey Bob,\r\n",
            "\r\n",
            "Just wanted to say hi!\r\n",
            "\r\n",
            "Best,\r\n",
            "Alice\r\n",
        )
        .as_bytes()
        .to_vec()
    }

    fn reply_email() -> Vec<u8> {
        concat!(
            "From: Bob <bob@example.com>\r\n",
            "To: Alice <alice@example.com>\r\n",
            "Subject: Re: Hello Bob\r\n",
            "Date: Thu, 05 Feb 2026 11:00:00 +0000\r\n",
            "Message-ID: <def456@example.com>\r\n",
            "In-Reply-To: <abc123@example.com>\r\n",
            "References: <abc123@example.com>\r\n",
            "Content-Type: text/plain; charset=utf-8\r\n",
            "\r\n",
            "Hi Alice!\r\n",
            "\r\n",
            "Great to hear from you.\r\n",
            "\r\n",
            "On Thu, 05 Feb 2026 at 10:00, Alice <alice@example.com> wrote:\r\n",
            "> Hey Bob,\r\n",
            ">\r\n",
            "> Just wanted to say hi!\r\n",
            ">\r\n",
            "> Best,\r\n",
            "> Alice\r\n",
        )
        .as_bytes()
        .to_vec()
    }

    #[test]
    fn test_simple_email() {
        let output = preprocess(&simple_email()).unwrap();
        assert_eq!(output.subject.as_deref(), Some("Hello Bob"));
        assert_eq!(output.from.as_ref().unwrap().email, "alice@example.com");
        assert_eq!(output.from.as_ref().unwrap().name.as_deref(), Some("Alice"));
        assert_eq!(output.to.len(), 1);
        assert_eq!(output.to[0].email, "bob@example.com");
        assert!(output.body.contains("Just wanted to say hi!"));
    }

    #[test]
    fn test_reply_strips_quotes() {
        let output = preprocess(&reply_email()).unwrap();
        assert!(output.body.contains("Great to hear from you."));
        assert!(!output.body.contains("Just wanted to say hi!"));
        assert_eq!(
            output.in_reply_to.as_ref().unwrap(),
            &["abc123@example.com"]
        );
    }

    #[test]
    fn test_clean_body_shorter_than_raw() {
        let output = preprocess(&reply_email()).unwrap();
        assert!(output.clean_body_length < output.raw_body_length);
    }

    #[test]
    fn test_clean_invisible_removes_all_types() {
        let input = "he\u{200B}ll\u{200C}o \u{200D}wo\u{FEFF}rld";
        assert_eq!(clean_invisible_characters(input), "hello world");
    }

    #[test]
    fn test_clean_invisible_extra_chars() {
        let input = "he\u{034F}llo\u{00AD} wo\u{2007}rld";
        assert_eq!(clean_invisible_characters(input), "hello world");
    }

    #[test]
    fn test_clean_invisible_nbsp_to_space() {
        let input = "hello\u{00A0}world";
        assert_eq!(clean_invisible_characters(input), "hello world");
    }

    #[test]
    fn test_clean_invisible_normal_text_unchanged() {
        let input = "Hello, world! 🎉 Ümlauts and ñ are fine.";
        assert_eq!(clean_invisible_characters(input), input);
    }

    #[test]
    fn test_trim_whitespace_lines() {
        assert_eq!(trim_whitespace_lines("a\n   \nb"), "a\n\nb");
        assert_eq!(trim_whitespace_lines("a\n \t \nb"), "a\n\nb");
        assert_eq!(trim_whitespace_lines("a\n\nb"), "a\n\nb");
        assert_eq!(trim_whitespace_lines("hello\nworld"), "hello\nworld");
    }

    #[test]
    fn test_collapse_empty_lines() {
        assert_eq!(collapse_empty_lines("a\n\n\nb"), "a\n\nb");
        assert_eq!(collapse_empty_lines("a\n\n\n\n\nb"), "a\n\nb");
        assert_eq!(collapse_empty_lines("a\n\nb"), "a\n\nb");
        assert_eq!(collapse_empty_lines("a\nb"), "a\nb");
    }

    #[test]
    fn test_email_with_invisible_chars_cleaned() {
        let raw = concat!(
            "From: Alice <alice@example.com>\r\n",
            "To: Bob <bob@example.com>\r\n",
            "Subject: Test\r\n",
            "Content-Type: text/plain; charset=utf-8\r\n",
            "\r\n",
            "Hello\u{200B} world\u{FEFF}!\r\n",
        )
        .as_bytes();
        let output = preprocess(raw).unwrap();
        assert_eq!(output.body, "Hello world!");
    }

    // --- Tests for preprocess_with_options ---

    fn make_email(body: &str) -> Vec<u8> {
        format!(
            "From: Alice <alice@example.com>\r\n\
             To: Bob <bob@example.com>\r\n\
             Subject: Test\r\n\
             Content-Type: text/plain; charset=utf-8\r\n\
             \r\n\
             {body}\r\n"
        )
        .into_bytes()
    }

    #[test]
    fn test_options_no_strip_quotes() {
        let raw = reply_email();
        let options = PreprocessOptions {
            strip_quotes: false,
            strip_signature: true,
            ..Default::default()
        };
        let output = preprocess_with_options(&raw, &options).unwrap();
        // Quotes should be preserved
        assert!(output.body.contains("Just wanted to say hi!"));
        assert!(output.body.contains("Great to hear from you."));
    }

    #[test]
    fn test_options_no_strip_signature() {
        let raw = make_email("Hello there.\n\n-- \nAlice\nCEO, Acme Corp");
        let options = PreprocessOptions {
            strip_quotes: true,
            strip_signature: false,
            ..Default::default()
        };
        let output = preprocess_with_options(&raw, &options).unwrap();
        assert!(output.body.contains("Hello there."));
        assert!(output.body.contains("Alice"));
        assert!(output.body.contains("CEO, Acme Corp"));
        assert!(output.signature.is_none());
    }

    #[test]
    fn test_options_max_body_length_ascii() {
        let raw = make_email("Hello world, this is a test message.");
        let options = PreprocessOptions {
            max_body_length: 5,
            ..Default::default()
        };
        let output = preprocess_with_options(&raw, &options).unwrap();
        assert_eq!(output.body, "Hello");
        assert_eq!(output.body.chars().count(), 5);
    }

    #[test]
    fn test_options_max_body_length_multibyte_no_panic() {
        // "Héllo 🌍 wörld" — contains 2-byte (é, ö) and 4-byte (🌍) chars.
        // Truncating at character boundaries must not panic.
        let raw = make_email("Héllo 🌍 wörld");
        let options = PreprocessOptions {
            max_body_length: 7,
            ..Default::default()
        };
        let output = preprocess_with_options(&raw, &options).unwrap();
        assert_eq!(output.body.chars().count(), 7);
        assert_eq!(output.body, "Héllo 🌍");
    }

    #[test]
    fn test_options_max_body_length_emoji_boundary() {
        // Each emoji is one char but 4 bytes. Truncate at 2 chars.
        let raw = make_email("🎉🎊🎈");
        let options = PreprocessOptions {
            max_body_length: 2,
            ..Default::default()
        };
        let output = preprocess_with_options(&raw, &options).unwrap();
        assert_eq!(output.body, "🎉🎊");
        assert_eq!(output.body.chars().count(), 2);
    }

    #[test]
    fn test_options_max_body_length_zero_means_no_limit() {
        let raw = make_email("Hello world");
        let options = PreprocessOptions {
            max_body_length: 0,
            ..Default::default()
        };
        let output = preprocess_with_options(&raw, &options).unwrap();
        assert_eq!(output.body, "Hello world");
    }

    #[test]
    fn test_options_max_body_length_larger_than_body() {
        let raw = make_email("Short");
        let options = PreprocessOptions {
            max_body_length: 1000,
            ..Default::default()
        };
        let output = preprocess_with_options(&raw, &options).unwrap();
        assert_eq!(output.body, "Short");
    }

    #[test]
    fn test_options_default_matches_preprocess() {
        let raw = reply_email();
        let a = preprocess(&raw).unwrap();
        let b = preprocess_with_options(&raw, &PreprocessOptions::default()).unwrap();
        assert_eq!(a.body, b.body);
        assert_eq!(a.signature, b.signature);
        assert_eq!(a.clean_body_length, b.clean_body_length);
        assert_eq!(a.raw_body_length, b.raw_body_length);
    }

    // --- Tests for preprocess_parsed ---

    #[test]
    fn test_preprocess_parsed_text_only() {
        let input = ParsedInput {
            text: Some("Hey Bob,\n\nJust wanted to say hi!\n\nBest,\nAlice".to_string()),
            subject: Some("Hello Bob".to_string()),
            from: Some(Address {
                name: Some("Alice".to_string()),
                email: "alice@example.com".to_string(),
            }),
            to: vec![Address {
                name: Some("Bob".to_string()),
                email: "bob@example.com".to_string(),
            }],
            ..Default::default()
        };
        let out = preprocess_parsed(input, &PreprocessOptions::default()).unwrap();
        assert_eq!(out.subject.as_deref(), Some("Hello Bob"));
        assert_eq!(out.from.as_ref().unwrap().email, "alice@example.com");
        assert!(out.body.contains("Just wanted to say hi!"));
        assert!(out.thread_messages.is_empty());
    }

    #[test]
    fn test_preprocess_parsed_html_only() {
        let input = ParsedInput {
            html: Some("<p>Hello <strong>world</strong>!</p>".to_string()),
            subject: Some("Test".to_string()),
            ..Default::default()
        };
        let out = preprocess_parsed(input, &PreprocessOptions::default()).unwrap();
        assert!(out.body.contains("Hello"));
        assert!(out.body.contains("world"));
        // HTML tags should not appear in the body after markdown conversion
        assert!(!out.body.contains("<p>"));
        assert!(!out.body.contains("<strong>"));
    }

    #[test]
    fn test_preprocess_parsed_prefers_html_over_text() {
        let input = ParsedInput {
            html: Some("<p>From HTML</p>".to_string()),
            text: Some("From text".to_string()),
            ..Default::default()
        };
        let out = preprocess_parsed(input, &PreprocessOptions::default()).unwrap();
        assert!(out.body.contains("From HTML"));
        assert!(!out.body.contains("From text"));
    }

    #[test]
    fn test_preprocess_parsed_empty_bodies() {
        let input = ParsedInput {
            subject: Some("Empty".to_string()),
            ..Default::default()
        };
        let out = preprocess_parsed(input, &PreprocessOptions::default()).unwrap();
        assert_eq!(out.body, "");
        assert_eq!(out.subject.as_deref(), Some("Empty"));
    }

    #[test]
    fn test_preprocess_parsed_strips_quotes() {
        let input = ParsedInput {
            text: Some(
                concat!(
                    "Hi Alice!\n\n",
                    "Great to hear from you.\n\n",
                    "On Thu, 05 Feb 2026 at 10:00, Alice <alice@example.com> wrote:\n",
                    "> Hey Bob,\n",
                    "> Just wanted to say hi!\n",
                )
                .to_string(),
            ),
            ..Default::default()
        };
        let out = preprocess_parsed(input, &PreprocessOptions::default()).unwrap();
        assert!(out.body.contains("Great to hear from you."));
        assert!(!out.body.contains("Just wanted to say hi!"));
    }

    #[test]
    fn test_preprocess_parsed_respects_strip_quotes_option() {
        let input = ParsedInput {
            text: Some(
                concat!(
                    "Hi Alice!\n\n",
                    "Great to hear from you.\n\n",
                    "On Thu, 05 Feb 2026 at 10:00, Alice <alice@example.com> wrote:\n",
                    "> Hey Bob,\n",
                    "> Just wanted to say hi!\n",
                )
                .to_string(),
            ),
            ..Default::default()
        };
        let options = PreprocessOptions {
            strip_quotes: false,
            ..Default::default()
        };
        let out = preprocess_parsed(input, &options).unwrap();
        assert!(out.body.contains("Great to hear from you."));
        assert!(out.body.contains("Just wanted to say hi!"));
    }

    #[test]
    fn test_preprocess_parsed_passes_through_threading_headers() {
        let input = ParsedInput {
            text: Some("Hi".to_string()),
            rfc_message_id: Some("msg-42@example.com".to_string()),
            in_reply_to: Some(vec!["parent-1@example.com".to_string()]),
            references: Some(vec![
                "root@example.com".to_string(),
                "parent-1@example.com".to_string(),
            ]),
            ..Default::default()
        };
        let out = preprocess_parsed(input, &PreprocessOptions::default()).unwrap();
        assert_eq!(out.rfc_message_id.as_deref(), Some("msg-42@example.com"));
        assert_eq!(out.in_reply_to.as_ref().unwrap(), &["parent-1@example.com"]);
        assert_eq!(
            out.references.as_ref().unwrap(),
            &["root@example.com", "parent-1@example.com"]
        );
    }

    #[test]
    fn test_preprocess_parsed_cleans_invisible_chars() {
        let input = ParsedInput {
            text: Some("Hello\u{200B} world\u{FEFF}!".to_string()),
            ..Default::default()
        };
        let out = preprocess_parsed(input, &PreprocessOptions::default()).unwrap();
        assert_eq!(out.body, "Hello world!");
    }
}
