use langmail_core::preprocess;

static TK_EML: &[u8] = include_bytes!("fixtures/tk-message.eml");

#[test]
fn test_tk_cta_url() {
    let output = preprocess(TK_EML).unwrap();
    let cta = output
        .primary_cta
        .expect("TK email should have a primary CTA");
    assert_eq!(cta.url, "https://applink.tk.de/postfach");
}

#[test]
fn test_tk_cta_text() {
    let output = preprocess(TK_EML).unwrap();
    let cta = output
        .primary_cta
        .expect("TK email should have a primary CTA");
    assert!(
        cta.text.contains("TK-Postfach"),
        "CTA text should mention TK-Postfach, got: {}",
        cta.text
    );
}

#[test]
fn test_tk_cta_confidence() {
    let output = preprocess(TK_EML).unwrap();
    let cta = output
        .primary_cta
        .expect("TK email should have a primary CTA");
    assert!(
        cta.confidence > 0.6,
        "CTA confidence should be > 0.6, got: {}",
        cta.confidence
    );
}
