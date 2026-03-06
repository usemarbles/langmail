use langmail::preprocess;

static AMIE_PROMO_EML: &[u8] = include_bytes!("fixtures/amie-promo.eml");

fn output() -> langmail::ProcessedEmail {
    preprocess(AMIE_PROMO_EML).unwrap()
}

#[test]
fn test_metadata() {
    let output = output();
    let from = output.from.expect("should have from");
    assert_eq!(from.name.as_deref(), Some("Max Mustermann"));
    assert_eq!(from.email, "noreply@example.com");
    assert_eq!(output.to.len(), 1);
    assert_eq!(output.to[0].email, "test@example.com");
    assert_eq!(output.subject.as_deref(), Some("new: craft in Amie"));
    assert_eq!(output.date.as_deref(), Some("2026-02-10T14:28:26Z"));
}

#[test]
fn test_body_paragraphs() {
    let body = &output().body;
    assert!(body.contains("hey!"), "missing 'hey!'");
    // Section heading is wrapped in <strong> → rendered as bold Markdown
    assert!(
        body.contains("**craft integration**"),
        "missing bold heading, body:\n{body}"
    );
    // Paragraph break between opening line and the bold heading
    assert!(
        body.contains("hey!\n\n**craft integration**"),
        "missing paragraph break before heading, body:\n{body}"
    );
}

#[test]
fn test_ordered_list() {
    let body = &output().body;
    // htmd renders ordered lists as `1. item`, spaces collapsed to one by
    // collapse_whitespace
    assert!(body.contains("1. in craft:"), "missing first ordered item");
    assert!(body.contains("5. in Amie:"), "missing fifth ordered item");
    // Last item contains bold "connect" rendered as **connect**
    assert!(body.contains("**connect**"), "missing bold connect step");
}

#[test]
fn test_unordered_list() {
    let body = &output().body;
    // htmd default bullet marker is `*`; collapse_whitespace normalises spacing
    assert!(body.contains("* better browser"), "missing first bullet");
    assert!(
        body.contains("* improved performance"),
        "missing third bullet"
    );
}

#[test]
fn test_hr_separator() {
    let body = &output().body;
    // htmd renders <hr> as `* * *` (default Asterisks style)
    assert!(
        body.contains("* * *"),
        "missing HR separator, body:\n{body}"
    );
}

#[test]
fn test_unsubscribe_preserved() {
    let body = &output().body;
    assert!(body.contains("Unsubscribe"), "missing Unsubscribe text");
}

#[test]
fn test_unsubscribe_no_leading_space() {
    let body = &output().body;
    for line in body.lines() {
        if line.contains("Unsubscribe") {
            assert!(
                !line.starts_with(' '),
                "Unsubscribe line should not have a leading space, got: {line:?}"
            );
        }
    }
}

#[test]
fn test_anchor_hrefs_dropped() {
    // All <a> hrefs (tracking URLs) must be stripped; only link text is kept.
    let body = &output().body;
    assert!(
        !body.contains("]("),
        "body should contain no Markdown link syntax `](`"
    );
    assert!(
        !body.contains("railway.app"),
        "tracking URL domain should not appear in body"
    );
}

#[test]
fn test_no_excessive_empty_lines() {
    let body = &output().body;
    assert!(
        !body.contains("\n\n\n"),
        "body contains 3+ consecutive newlines:\n{body}"
    );
}

#[test]
fn test_no_whitespace_only_lines() {
    let body = &output().body;
    for (i, line) in body.lines().enumerate() {
        assert!(
            line.is_empty() || !line.trim().is_empty(),
            "line {} is whitespace-only: {:?}",
            i + 1,
            line
        );
    }
}

#[test]
fn test_llm_context() {
    let ctx = output().to_llm_context();
    assert!(
        ctx.starts_with("FROM: Max Mustermann <noreply@example.com>\n"),
        "wrong FROM line, got start: {:?}",
        &ctx[..ctx.find('\n').unwrap_or(ctx.len())]
    );
    assert!(ctx.contains("TO: test@example.com\n"), "wrong TO line");
    assert!(
        ctx.contains("SUBJECT: new: craft in Amie\n"),
        "wrong SUBJECT"
    );
    assert!(ctx.contains("DATE: 2026-02-10T14:28:26Z\n"), "wrong DATE");
    assert!(ctx.contains("CONTENT:\n"), "missing CONTENT marker");
}
