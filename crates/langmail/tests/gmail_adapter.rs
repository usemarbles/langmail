//! Integration tests for the Gmail adapter.
//!
//! Mirrors the coverage that previously lived in
//! `packages/langmail/test/gmail.test.js`. The JS wrapper is now a
//! thin `JSON.stringify` passthrough, so all behavioral verification
//! belongs here in Rust.

use std::path::PathBuf;

use langmail::adapters::{preprocess_gmail, preprocess_gmail_with_options};
use langmail::{LangmailError, PreprocessOptions};

// ---------------------------------------------------------------------------
// Fixture loading
// ---------------------------------------------------------------------------

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("adapters")
        .join("gmail")
        .join(name)
}

fn load_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture_path(name))
        .unwrap_or_else(|e| panic!("failed to read fixture {name}: {e}"))
}

// ---------------------------------------------------------------------------
// simple.json — text/plain message
// ---------------------------------------------------------------------------

#[test]
fn simple_parses_headers_and_body() {
    let result = preprocess_gmail(&load_fixture("simple.json")).unwrap();
    assert_eq!(result.subject.as_deref(), Some("Hello Bob"));
    let from = result.from.expect("from should be set");
    assert_eq!(from.name.as_deref(), Some("Alice"));
    assert_eq!(from.email, "alice@example.com");
    assert_eq!(result.to.len(), 1);
    assert_eq!(result.to[0].email, "bob@example.com");
    assert_eq!(result.to[0].name.as_deref(), Some("Bob"));
    assert!(
        result.body.contains("Just wanted to say hi!"),
        "body was: {:?}",
        result.body
    );
}

#[test]
fn simple_strips_angle_brackets_from_message_id() {
    let result = preprocess_gmail(&load_fixture("simple.json")).unwrap();
    assert_eq!(result.rfc_message_id.as_deref(), Some("abc123@example.com"));
}

#[test]
fn simple_converts_date_to_iso_8601_without_fractional_seconds() {
    let result = preprocess_gmail(&load_fixture("simple.json")).unwrap();
    // Matches the MIME path's format so the `date` field is byte-identical
    // across `preprocess` and `preprocess_gmail`.
    assert_eq!(result.date.as_deref(), Some("2026-02-05T10:00:00Z"));
}

#[test]
fn simple_returns_none_for_malformed_date() {
    let mut msg: serde_json::Value = serde_json::from_str(&load_fixture("simple.json")).unwrap();
    for h in msg["payload"]["headers"].as_array_mut().unwrap() {
        if h["name"] == "Date" {
            h["value"] = serde_json::json!("not a real date");
        }
    }
    let result = preprocess_gmail(&msg.to_string()).unwrap();
    assert_eq!(result.date, None);
}

#[test]
fn simple_strips_signature_by_default() {
    let result = preprocess_gmail(&load_fixture("simple.json")).unwrap();
    assert!(
        !result.body.contains("Alice\nCEO"),
        "body was: {:?}",
        result.body
    );
    assert!(
        !result.body.trim_end().ends_with("Alice"),
        "body should not end with the signature name; got: {:?}",
        result.body
    );
}

// ---------------------------------------------------------------------------
// multipart-alternative.json — HTML-over-plain with quoted Cc name
// ---------------------------------------------------------------------------

#[test]
fn multipart_prefers_html_over_plain() {
    let result = preprocess_gmail(&load_fixture("multipart-alternative.json")).unwrap();
    // HTML and plain both say "Great to hear from you.", but only the HTML
    // version has the CTA — its presence proves the HTML branch was taken.
    let cta = result.primary_cta.expect("primary_cta should be set");
    assert_eq!(cta.url, "https://example.com/view");
    assert!(cta.text.contains("View"), "CTA text was: {:?}", cta.text);
}

#[test]
fn multipart_parses_quoted_comma_in_cc() {
    let result = preprocess_gmail(&load_fixture("multipart-alternative.json")).unwrap();
    assert_eq!(result.cc.len(), 2);
    assert_eq!(result.cc[0].name.as_deref(), Some("Carol, Support"));
    assert_eq!(result.cc[0].email, "carol@example.com");
    assert_eq!(result.cc[1].email, "dev@example.com");
    assert_eq!(result.cc[1].name, None);
}

#[test]
fn multipart_renders_html_as_markdown_no_tags() {
    let result = preprocess_gmail(&load_fixture("multipart-alternative.json")).unwrap();
    assert!(result.body.contains("Hi Alice!"));
    assert!(result.body.contains("Great to hear from you."));
    assert!(!result.body.contains("<p>"));
    assert!(!result.body.contains("</body>"));
}

// ---------------------------------------------------------------------------
// threaded-reply.json — quoted reply with thread extraction
// ---------------------------------------------------------------------------

#[test]
fn threaded_strips_quoted_reply() {
    let result = preprocess_gmail(&load_fixture("threaded-reply.json")).unwrap();
    assert!(result.body.contains("Thanks for the update, Alice!"));
    assert!(!result.body.contains("can you take a look at the PR"));
}

#[test]
fn threaded_extracts_thread_messages_from_blockquote() {
    let result = preprocess_gmail(&load_fixture("threaded-reply.json")).unwrap();
    assert!(
        !result.thread_messages.is_empty(),
        "expected thread_messages, got none"
    );
    let first = &result.thread_messages[0];
    assert!(
        first.sender.contains("alice@example.com"),
        "sender was: {:?}",
        first.sender
    );
    assert!(
        first.body.contains("take a look at the PR"),
        "body was: {:?}",
        first.body
    );
}

#[test]
fn threaded_splits_references_on_whitespace() {
    let result = preprocess_gmail(&load_fixture("threaded-reply.json")).unwrap();
    assert_eq!(
        result.references.as_deref(),
        Some(
            &[
                "root-000@example.com".to_string(),
                "abc123@example.com".to_string()
            ][..]
        )
    );
}

#[test]
fn threaded_in_reply_to_is_array() {
    let result = preprocess_gmail(&load_fixture("threaded-reply.json")).unwrap();
    assert_eq!(
        result.in_reply_to.as_deref(),
        Some(&["abc123@example.com".to_string()][..])
    );
}

#[test]
fn threaded_parses_quoted_comma_in_from() {
    let result = preprocess_gmail(&load_fixture("threaded-reply.json")).unwrap();
    let from = result.from.expect("from");
    assert_eq!(from.name.as_deref(), Some("Lastname, Firstname"));
    assert_eq!(from.email, "firstname@example.com");
}

// ---------------------------------------------------------------------------
// Input forms
// ---------------------------------------------------------------------------

#[test]
fn accepts_full_googleapis_response_wrapper() {
    let msg: serde_json::Value = serde_json::from_str(&load_fixture("simple.json")).unwrap();
    let wrapped = serde_json::json!({ "data": msg, "status": 200 });
    let result = preprocess_gmail(&wrapped.to_string()).unwrap();
    assert_eq!(result.subject.as_deref(), Some("Hello Bob"));
}

#[test]
fn forwards_options_to_pipeline() {
    let opts = PreprocessOptions {
        strip_quotes: false,
        ..Default::default()
    };
    let result =
        preprocess_gmail_with_options(&load_fixture("threaded-reply.json"), &opts).unwrap();
    assert!(result.body.contains("take a look at the PR"));
}

#[test]
fn missing_payload_is_error() {
    let err = preprocess_gmail(r#"{ "id": "x" }"#).expect_err("should fail");
    match err {
        LangmailError::InvalidGmailMessage(msg) => {
            assert!(msg.contains("payload is missing"), "message was: {:?}", msg);
        }
        other => panic!("expected InvalidGmailMessage, got {:?}", other),
    }
}

#[test]
fn non_object_input_is_error() {
    // JSON parse of `null` succeeds but the value is not an object.
    let err = preprocess_gmail("null").expect_err("should fail");
    match err {
        LangmailError::InvalidGmailMessage(_) => {}
        other => panic!("expected InvalidGmailMessage, got {:?}", other),
    }

    // Invalid JSON entirely.
    let err = preprocess_gmail("not-json").expect_err("should fail");
    match err {
        LangmailError::InvalidGmailMessage(msg) => {
            assert!(msg.contains("invalid JSON"), "message was: {:?}", msg);
        }
        other => panic!("expected InvalidGmailMessage, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Body edge cases
// ---------------------------------------------------------------------------

#[test]
fn empty_body_when_payload_has_no_data_or_parts() {
    let msg = serde_json::json!({
        "id": "empty",
        "payload": {
            "mimeType": "text/plain",
            "headers": [{ "name": "Subject", "value": "Empty" }],
            "body": { "size": 0 }
        }
    });
    let result = preprocess_gmail(&msg.to_string()).unwrap();
    assert_eq!(result.body, "");
    assert_eq!(result.subject.as_deref(), Some("Empty"));
}

#[test]
fn attachment_id_body_surfaces_actionable_error() {
    let msg = serde_json::json!({
        "id": "big",
        "payload": {
            "mimeType": "text/plain",
            "headers": [{ "name": "Subject", "value": "Large body" }],
            "body": { "size": 9999999, "attachmentId": "ATT_abc123" }
        }
    });
    let err = preprocess_gmail(&msg.to_string()).expect_err("should fail");
    match err {
        LangmailError::BodyRequiresAttachmentFetch {
            mime_type,
            attachment_id,
        } => {
            assert_eq!(mime_type, "text/plain");
            assert_eq!(attachment_id, "ATT_abc123");
            // Display impl should mention the actionable path.
            let display = format!(
                "{}",
                LangmailError::BodyRequiresAttachmentFetch {
                    mime_type,
                    attachment_id,
                }
            );
            assert!(
                display.contains("attachments.get"),
                "display was: {}",
                display
            );
        }
        other => panic!("expected BodyRequiresAttachmentFetch, got {:?}", other),
    }
}

#[test]
fn sparse_headers_degrade_gracefully() {
    // `Hello world` → base64url "SGVsbG8gd29ybGQ"
    let msg = serde_json::json!({
        "id": "sparse",
        "payload": {
            "mimeType": "text/plain",
            "headers": [{ "name": "Subject", "value": "Minimal" }],
            "body": { "size": 11, "data": "SGVsbG8gd29ybGQ" }
        }
    });
    let result = preprocess_gmail(&msg.to_string()).unwrap();
    assert_eq!(result.subject.as_deref(), Some("Minimal"));
    assert!(result.from.is_none());
    assert!(result.to.is_empty());
    assert!(result.cc.is_empty());
    assert!(result.date.is_none());
    assert!(result.rfc_message_id.is_none());
    assert!(result.in_reply_to.is_none());
    assert!(result.references.is_none());
    assert!(
        result.body.contains("Hello world"),
        "body was: {:?}",
        result.body
    );
}

// ---------------------------------------------------------------------------
// Header parsing edge cases
// ---------------------------------------------------------------------------

#[test]
fn legacy_comment_form_from_header() {
    let mut msg: serde_json::Value = serde_json::from_str(&load_fixture("simple.json")).unwrap();
    for h in msg["payload"]["headers"].as_array_mut().unwrap() {
        if h["name"] == "From" {
            h["value"] = serde_json::json!("alice@example.com (Alice Example)");
        }
    }
    let result = preprocess_gmail(&msg.to_string()).unwrap();
    let from = result.from.expect("from");
    assert_eq!(from.email, "alice@example.com");
    assert_eq!(from.name.as_deref(), Some("Alice Example"));
}

#[test]
fn whitespace_only_in_reply_to_is_absent() {
    let mut msg: serde_json::Value = serde_json::from_str(&load_fixture("simple.json")).unwrap();
    msg["payload"]["headers"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({ "name": "In-Reply-To", "value": "   " }));
    let result = preprocess_gmail(&msg.to_string()).unwrap();
    assert!(result.in_reply_to.is_none());
}

#[test]
fn non_email_tokens_filtered_from_references() {
    let mut msg: serde_json::Value = serde_json::from_str(&load_fixture("simple.json")).unwrap();
    msg["payload"]["headers"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "name": "References",
            "value": "<root@example.com> (imported from archive) <abc@example.com>"
        }));
    let result = preprocess_gmail(&msg.to_string()).unwrap();
    assert_eq!(
        result.references.as_deref(),
        Some(
            &[
                "root@example.com".to_string(),
                "abc@example.com".to_string()
            ][..]
        )
    );
}
