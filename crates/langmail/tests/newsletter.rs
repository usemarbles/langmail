use langmail::preprocess;

static GHOST_NEWSLETTER: &[u8] = include_bytes!("../../../fixtures/ghost-newsletter.eml");
static GITHUB_PR: &[u8] = include_bytes!("../../../fixtures/github-pr-comment.eml");
static EVENTSPACE: &[u8] = include_bytes!("../../../fixtures/eventspace-booking.eml");

fn make_eml(extra_headers: &[(&str, &str)]) -> Vec<u8> {
    let mut s = "From: sender@example.com\r\nTo: user@example.com\r\nSubject: Test\r\n".to_string();
    for (k, v) in extra_headers {
        s.push_str(&format!("{}: {}\r\n", k, v));
    }
    s.push_str("\r\nBody text.\r\n");
    s.into_bytes()
}

// 1. Ghost/Mailgun newsletter → true
#[test]
fn test_newsletter_ghost_mailgun() {
    let result = preprocess(GHOST_NEWSLETTER).unwrap();
    assert!(result.is_newsletter);
}

// 2. GitHub PR notification → false (exclusion overrides inclusion signals)
#[test]
fn test_newsletter_github_pr_notification() {
    let result = preprocess(GITHUB_PR).unwrap();
    assert!(!result.is_newsletter);
}

// 3. Personal email (no bulk headers) → false
#[test]
fn test_newsletter_personal_email() {
    let result = preprocess(EVENTSPACE).unwrap();
    assert!(!result.is_newsletter);
}

// 4. List-Unsubscribe alone → true
#[test]
fn test_newsletter_list_unsubscribe_only() {
    let raw = make_eml(&[("List-Unsubscribe", "<https://example.com/unsub>")]);
    let result = preprocess(&raw).unwrap();
    assert!(result.is_newsletter);
}

// 4b. List-Id alone → true (no List-Unsubscribe)
#[test]
fn test_newsletter_list_id_only() {
    let raw = make_eml(&[("List-Id", "<monthly.example.com>")]);
    let result = preprocess(&raw).unwrap();
    assert!(result.is_newsletter);
}

// 5. In-Reply-To overrides List-Unsubscribe → false
#[test]
fn test_newsletter_in_reply_to_overrides_list_unsubscribe() {
    let raw = make_eml(&[
        ("List-Unsubscribe", "<https://example.com/unsub>"),
        ("In-Reply-To", "<parent-msg@example.com>"),
    ]);
    let result = preprocess(&raw).unwrap();
    assert!(!result.is_newsletter);
}

// 6. List-Post overrides Precedence: list → false
#[test]
fn test_newsletter_list_post_overrides_precedence() {
    let raw = make_eml(&[
        ("Precedence", "list"),
        ("List-Post", "<mailto:list@example.com>"),
    ]);
    let result = preprocess(&raw).unwrap();
    assert!(!result.is_newsletter);
}

// 7. X-GitHub-* header alone → false
#[test]
fn test_newsletter_xgithub_header_excluded() {
    let raw = make_eml(&[("X-GitHub-Sender", "octocat")]);
    let result = preprocess(&raw).unwrap();
    assert!(!result.is_newsletter);
}

// 8. Precedence: bulk → true
#[test]
fn test_newsletter_precedence_bulk() {
    let raw = make_eml(&[("Precedence", "bulk")]);
    let result = preprocess(&raw).unwrap();
    assert!(result.is_newsletter);
}

// 9. DKIM d= matching ESP domain → true
#[test]
fn test_newsletter_dkim_esp_domain() {
    let raw = make_eml(&[(
        "DKIM-Signature",
        "v=1; a=rsa-sha256; c=relaxed/relaxed; d=mailgun.org; s=mg; h=from:to:subject; b=abc123",
    )]);
    let result = preprocess(&raw).unwrap();
    assert!(result.is_newsletter);
}

// 10. X-Mailgun-* header → true
#[test]
fn test_newsletter_xheader_mailgun() {
    let raw = make_eml(&[("X-Mailgun-Variables", "{}")]);
    let result = preprocess(&raw).unwrap();
    assert!(result.is_newsletter);
}
