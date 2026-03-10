use langmail::{preprocess, LlmContextOptions, RenderMode};

static EVENTSPACE_EML: &[u8] = include_bytes!("fixtures/eventspace-booking.eml");
static LATEST_ONLY_SNAPSHOT: &str = include_str!("fixtures/eventspace-booking-latest-only.txt");
static THREAD_HISTORY_SNAPSHOT: &str =
    include_str!("fixtures/eventspace-booking-thread-history.txt");

#[test]
fn latest_only_matches_default_to_llm_context() {
    let output = preprocess(EVENTSPACE_EML).unwrap();
    let default_ctx = output.to_llm_context();
    let explicit_ctx = output.to_llm_context_with_options(&LlmContextOptions {
        render_mode: RenderMode::LatestOnly,
    });
    assert_eq!(default_ctx, explicit_ctx);
}

#[test]
fn latest_only_matches_snapshot() {
    let output = preprocess(EVENTSPACE_EML).unwrap();
    let ctx = output.to_llm_context();
    assert_eq!(ctx, LATEST_ONLY_SNAPSHOT);
}

#[test]
fn thread_history_has_separator_and_header() {
    let output = preprocess(EVENTSPACE_EML).unwrap();
    let ctx = output.to_llm_context_with_options(&LlmContextOptions {
        render_mode: RenderMode::ThreadHistory,
    });
    assert!(
        ctx.contains("\n---\n"),
        "should contain separator, got:\n{ctx}"
    );
    assert!(
        ctx.contains("THREAD HISTORY:"),
        "should contain THREAD HISTORY header, got:\n{ctx}"
    );
}

#[test]
fn thread_history_contains_three_prior_messages() {
    let output = preprocess(EVENTSPACE_EML).unwrap();
    let ctx = output.to_llm_context_with_options(&LlmContextOptions {
        render_mode: RenderMode::ThreadHistory,
    });

    // The thread has 3 prior messages (the latest is the main body)
    let history_section = ctx.split("THREAD HISTORY:").nth(1).unwrap();

    // Each prior message starts with a [timestamp] header line
    let message_headers: Vec<&str> = history_section
        .lines()
        .filter(|l| l.starts_with('['))
        .collect();

    assert_eq!(
        message_headers.len(),
        3,
        "expected 3 prior messages in thread history, got {}: {:?}",
        message_headers.len(),
        message_headers
    );
}

#[test]
fn thread_history_chronological_order() {
    let output = preprocess(EVENTSPACE_EML).unwrap();
    let ctx = output.to_llm_context_with_options(&LlmContextOptions {
        render_mode: RenderMode::ThreadHistory,
    });

    let history_section = ctx.split("THREAD HISTORY:").nth(1).unwrap();

    // Extract the message header lines in order
    let message_headers: Vec<&str> = history_section
        .lines()
        .filter(|l| l.starts_with('['))
        .collect();

    // First message should be the oldest (Feb 26), last should be newest (Feb 27 11:53)
    assert!(
        message_headers[0].contains("2026-02-26"),
        "first message should be from Feb 26, got: {}",
        message_headers[0]
    );
    assert!(
        message_headers[2].contains("2026-02-27"),
        "last message should be from Feb 27, got: {}",
        message_headers[2]
    );
}

#[test]
fn thread_history_no_duplicated_content() {
    let output = preprocess(EVENTSPACE_EML).unwrap();
    let ctx = output.to_llm_context_with_options(&LlmContextOptions {
        render_mode: RenderMode::ThreadHistory,
    });

    // The original inquiry text should appear exactly once
    let count = ctx.matches("Vibe Coding Workshop").count();
    assert_eq!(
        count, 1,
        "original inquiry should appear exactly once, found {count} times"
    );

    // The venue availability reply should appear exactly once
    let count = ctx.matches("Event Space noch verfügbar").count();
    assert_eq!(
        count, 1,
        "availability reply should appear exactly once, found {count} times"
    );
}

#[test]
fn thread_history_latest_message_appears_first() {
    let output = preprocess(EVENTSPACE_EML).unwrap();
    let ctx = output.to_llm_context_with_options(&LlmContextOptions {
        render_mode: RenderMode::ThreadHistory,
    });

    // The latest message content should come before the THREAD HISTORY section
    let separator_pos = ctx.find("---").unwrap();
    let latest_content_pos = ctx.find("Bevor ich dir das Angebot erstelle").unwrap();
    assert!(
        latest_content_pos < separator_pos,
        "latest message should appear before the separator"
    );
}

#[test]
fn thread_history_strips_signatures_from_quoted_messages() {
    let output = preprocess(EVENTSPACE_EML).unwrap();
    let ctx = output.to_llm_context_with_options(&LlmContextOptions {
        render_mode: RenderMode::ThreadHistory,
    });

    let history_section = ctx.split("THREAD HISTORY:").nth(1).unwrap();

    // The VENUE1 Events Team signature block should not appear in thread history
    assert!(
        !history_section.contains("VENUE1 Events Team"),
        "signatures should be stripped from quoted messages"
    );

    // The Venue1 GmbH legal footer should not appear in thread history
    assert!(
        !history_section.contains("Registergericht"),
        "legal footers should be stripped from quoted messages"
    );
}

#[test]
fn thread_history_matches_snapshot() {
    let output = preprocess(EVENTSPACE_EML).unwrap();
    let ctx = output.to_llm_context_with_options(&LlmContextOptions {
        render_mode: RenderMode::ThreadHistory,
    });
    assert_eq!(ctx, THREAD_HISTORY_SNAPSHOT);
}

#[test]
fn render_mode_default_is_latest_only() {
    let options = LlmContextOptions::default();
    assert!(matches!(options.render_mode, RenderMode::LatestOnly));
}
