use langmail_core::preprocess;

static DIRECT_EML: &[u8] = include_bytes!("fixtures/direct.eml");

#[test]
fn test_to_llm_context_contains_headers() {
    let output = preprocess(DIRECT_EML).unwrap();
    let ctx = output.to_llm_context();

    assert!(ctx.contains("FROM: Max Mustermann <noreply@example.com>"));
    assert!(ctx.contains("TO: test@example.com"));
    assert!(ctx.contains("SUBJECT: Feedbacks"));
    assert!(ctx.contains("DATE: 2025-11-08T13:04:26Z"));
}

#[test]
fn test_to_llm_context_contains_body() {
    let output = preprocess(DIRECT_EML).unwrap();
    let ctx = output.to_llm_context();

    assert!(ctx.contains("CONTENT:"));
    assert!(ctx.contains("anbei mein erstes Feedback"));
}

#[test]
fn test_to_llm_context_deterministic() {
    let output = preprocess(DIRECT_EML).unwrap();
    let a = output.to_llm_context();
    let b = output.to_llm_context();
    assert_eq!(a, b);
}

#[test]
fn test_to_llm_context_format() {
    let raw = concat!(
        "From: Alice <alice@example.com>\r\n",
        "To: Bob <bob@example.com>\r\n",
        "Subject: Hello\r\n",
        "Date: Thu, 05 Feb 2026 10:00:00 +0000\r\n",
        "Content-Type: text/plain; charset=utf-8\r\n",
        "\r\n",
        "Hey Bob, just wanted to say hi!\r\n",
    )
    .as_bytes();

    let output = preprocess(raw).unwrap();
    let ctx = output.to_llm_context();

    let expected = "\
FROM: Alice <alice@example.com>
TO: Bob <bob@example.com>
SUBJECT: Hello
DATE: 2026-02-05T10:00:00Z
CONTENT:
Hey Bob, just wanted to say hi!";

    assert_eq!(ctx, expected);
}
