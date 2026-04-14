//! Integration tests for `preprocess_parsed` — the adapter entry point that
//! bypasses the MIME parser. These exercise end-to-end behavior complementary
//! to the focused unit tests in `lib.rs`: parity with `preprocess`, CTA and
//! thread extraction on HTML input, signature stripping on text input.

use langmail::{preprocess, preprocess_parsed, Address, ParsedInput, PreprocessOptions};

/// An HTML body with a blockquote-wrapped reply and a clear primary CTA link.
/// The attribution div uses `gmail_attr` (same marker Gmail's web client emits)
/// so `extract_thread_messages` can pair it with the blockquote.
const THREADED_HTML: &str = r#"<html><body>
<p>Thanks for the update!</p>
<p><a href="https://example.com/review" style="display:inline-block;padding:12px 24px;background:#007bff;color:#fff" class="btn">View the changes now</a></p>
<div class="gmail_attr">On Thu, 05 Feb 2026 at 10:00, Alice &lt;alice@example.com&gt; wrote:</div>
<blockquote class="gmail_quote" style="border-left:2px solid #ccc;padding-left:10px">
<p>Can you take a look at the PR?</p>
</blockquote>
</body></html>"#;

#[test]
fn parsed_html_extracts_primary_cta() {
    let input = ParsedInput {
        html: Some(THREADED_HTML.to_string()),
        subject: Some("Re: PR review".to_string()),
        ..Default::default()
    };
    let out = preprocess_parsed(input, &PreprocessOptions::default()).unwrap();
    let cta = out
        .primary_cta
        .expect("expected primary_cta to be detected on styled button link");
    assert_eq!(cta.url, "https://example.com/review");
    assert!(cta.text.contains("View"));
}

#[test]
fn parsed_html_extracts_thread_messages() {
    let input = ParsedInput {
        html: Some(THREADED_HTML.to_string()),
        ..Default::default()
    };
    let out = preprocess_parsed(input, &PreprocessOptions::default()).unwrap();
    assert!(
        !out.thread_messages.is_empty(),
        "expected thread_messages to be populated from blockquote; got {:?}",
        out.thread_messages
    );
    // Latest-only body must not contain the quoted content
    assert!(!out.body.contains("Can you take a look"));
    // Main message is preserved
    assert!(out.body.contains("Thanks for the update!"));
}

#[test]
fn parsed_text_strips_signature() {
    let input = ParsedInput {
        text: Some("Hello there.\n\n-- \nAlice\nCEO, Acme Corp".to_string()),
        ..Default::default()
    };
    let out = preprocess_parsed(input, &PreprocessOptions::default()).unwrap();
    assert!(out.body.contains("Hello there."));
    assert!(
        !out.body.contains("CEO, Acme Corp"),
        "expected signature to be stripped from body; got {:?}",
        out.body
    );
    assert!(out.signature.is_some());
    assert!(out.signature.as_deref().unwrap().contains("CEO, Acme Corp"));
}

#[test]
fn parsed_text_parity_with_preprocess() {
    // Same content as RFC 5322 and as ParsedInput — the cleaning pipeline
    // should produce an identical body, regardless of the input route.
    let raw = concat!(
        "From: Alice <alice@example.com>\r\n",
        "To: Bob <bob@example.com>\r\n",
        "Subject: Hello Bob\r\n",
        "Date: Thu, 05 Feb 2026 10:00:00 +0000\r\n",
        "Message-ID: <abc123@example.com>\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "\r\n",
        "Hey Bob,\r\n\r\nJust wanted to say hi!\r\n",
    )
    .as_bytes();
    let eml_out = preprocess(raw).unwrap();

    let parsed = ParsedInput {
        text: Some("Hey Bob,\n\nJust wanted to say hi!".to_string()),
        subject: Some("Hello Bob".to_string()),
        from: Some(Address {
            name: Some("Alice".to_string()),
            email: "alice@example.com".to_string(),
        }),
        to: vec![Address {
            name: Some("Bob".to_string()),
            email: "bob@example.com".to_string(),
        }],
        date: Some("2026-02-05T10:00:00Z".to_string()),
        rfc_message_id: Some("abc123@example.com".to_string()),
        ..Default::default()
    };
    let parsed_out = preprocess_parsed(parsed, &PreprocessOptions::default()).unwrap();

    assert_eq!(parsed_out.body, eml_out.body);
    assert_eq!(parsed_out.subject, eml_out.subject);
    assert_eq!(
        parsed_out.from.as_ref().map(|a| &a.email),
        eml_out.from.as_ref().map(|a| &a.email),
    );
    assert_eq!(parsed_out.rfc_message_id, eml_out.rfc_message_id);
}

#[test]
fn parsed_respects_strip_signature_false() {
    let input = ParsedInput {
        text: Some("Hello there.\n\n-- \nAlice\nCEO, Acme Corp".to_string()),
        ..Default::default()
    };
    let opts = PreprocessOptions {
        strip_signature: false,
        ..PreprocessOptions::default()
    };
    let out = preprocess_parsed(input, &opts).unwrap();
    // With signature stripping disabled the sig stays in the body and
    // the dedicated `signature` field is not populated.
    assert!(out.body.contains("CEO, Acme Corp"));
    assert!(out.signature.is_none());
}

#[test]
fn parsed_respects_max_body_length() {
    // Body (post-cleaning) is "Hello, world!" — 13 bytes. Truncating at 5
    // should yield "Hello" exactly.
    let input = ParsedInput {
        text: Some("Hello, world!".to_string()),
        ..Default::default()
    };
    let opts = PreprocessOptions {
        max_body_length: 5,
        ..PreprocessOptions::default()
    };
    let out = preprocess_parsed(input, &opts).unwrap();
    assert_eq!(out.body, "Hello");
    // raw_body_length tracks the pre-truncation length so callers can
    // detect that truncation happened.
    assert!(out.raw_body_length >= 13);
}

#[test]
fn parsed_llm_context_renders() {
    // Confirm the downstream `to_llm_context` path works on a ProcessedEmail
    // produced by the adapter route.
    let input = ParsedInput {
        text: Some("Hello there.".to_string()),
        subject: Some("Hi".to_string()),
        from: Some(Address {
            name: Some("Alice".to_string()),
            email: "alice@example.com".to_string(),
        }),
        to: vec![Address {
            name: None,
            email: "bob@example.com".to_string(),
        }],
        date: Some("2026-02-05T10:00:00Z".to_string()),
        ..Default::default()
    };
    let out = preprocess_parsed(input, &PreprocessOptions::default()).unwrap();
    let ctx = out.to_llm_context();
    assert!(ctx.contains("FROM: Alice <alice@example.com>"));
    assert!(ctx.contains("TO: bob@example.com"));
    assert!(ctx.contains("SUBJECT: Hi"));
    assert!(ctx.contains("DATE: 2026-02-05T10:00:00Z"));
    assert!(ctx.contains("CONTENT:"));
    assert!(ctx.contains("Hello there."));
}
