use langmail_core::preprocess;

static LINKEDIN_EML: &[u8] = include_bytes!("fixtures/linkedin.eml");

#[test]
fn test_linkedin_from() {
    let output = preprocess(LINKEDIN_EML).unwrap();
    let from = output.from.expect("should have a from address");
    assert_eq!(from.name.as_deref(), Some("Max Mustermann via LinkedIn"));
    assert_eq!(from.email, "noreply@example.com");
}

#[test]
fn test_linkedin_to() {
    let output = preprocess(LINKEDIN_EML).unwrap();
    assert_eq!(output.to.len(), 1);
    assert_eq!(output.to[0].name.as_deref(), Some("Max Mustermann"));
    assert_eq!(output.to[0].email, "test@example.com");
}

#[test]
fn test_linkedin_subject() {
    let output = preprocess(LINKEDIN_EML).unwrap();
    assert_eq!(output.subject.as_deref(), Some("Max just messaged you"));
}

#[test]
fn test_linkedin_date() {
    let output = preprocess(LINKEDIN_EML).unwrap();
    assert_eq!(output.date.as_deref(), Some("2025-05-24T11:31:25Z"));
}

#[test]
fn test_linkedin_cta_text() {
    let output = preprocess(LINKEDIN_EML).unwrap();
    let cta = output
        .primary_cta
        .expect("LinkedIn email should have a primary CTA");
    assert!(
        cta.text.contains("View message"),
        "CTA text should contain 'View message', got: {}",
        cta.text
    );
}

#[test]
fn test_linkedin_cta_url() {
    let output = preprocess(LINKEDIN_EML).unwrap();
    let cta = output
        .primary_cta
        .expect("LinkedIn email should have a primary CTA");
    assert!(
        cta.url.contains("linkedin.com"),
        "CTA URL should contain 'linkedin.com', got: {}",
        cta.url
    );
}

#[test]
fn test_linkedin_cta_confidence() {
    let output = preprocess(LINKEDIN_EML).unwrap();
    let cta = output
        .primary_cta
        .expect("LinkedIn email should have a primary CTA");
    assert!(
        cta.confidence > 0.7,
        "CTA confidence should be > 0.7, got: {}",
        cta.confidence
    );
}
