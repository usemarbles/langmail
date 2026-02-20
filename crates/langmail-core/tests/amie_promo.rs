use langmail_core::preprocess;

static AMIE_PROMO_EML: &[u8] = include_bytes!("fixtures/amie-promo.eml");

fn output() -> langmail_core::ProcessedEmail {
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
    assert!(body.contains("craft integration"), "missing 'craft integration'");
    // Paragraph break between opening line and section heading
    assert!(
        body.contains("hey!\n\ncraft integration"),
        "missing paragraph break, body:\n{body}"
    );
}

#[test]
fn test_ordered_list() {
    let body = &output().body;
    assert!(body.contains(" 1. in craft:"), "missing first ordered item");
    assert!(body.contains(" 5. in Amie:"), "missing fifth ordered item");
    assert!(body.contains("> connect"), "missing connect step");
}

#[test]
fn test_unordered_list() {
    let body = &output().body;
    assert!(body.contains(" * better browser"), "missing first bullet");
    assert!(
        body.contains(" * improved performance"),
        "missing third bullet"
    );
}

#[test]
fn test_hr_separator() {
    let body = &output().body;
    assert!(
        body.contains("--------"),
        "missing HR separator (8+ dashes), body:\n{body}"
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
            assert_eq!(
                line, "Unsubscribe",
                "Unsubscribe line should have no leading/trailing spaces, got: {line:?}"
            );
        }
    }
}

#[test]
fn test_no_markdown_artifacts() {
    let body = &output().body;
    assert!(!body.contains("**"), "body contains markdown bold markers");
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
    assert!(
        ctx.contains("DATE: 2026-02-10T14:28:26Z\n"),
        "wrong DATE"
    );
    assert!(ctx.contains("CONTENT:\n"), "missing CONTENT marker");
}
