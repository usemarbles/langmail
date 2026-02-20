use langmail_core::preprocess;

static FORWARDED_EML: &[u8] = include_bytes!("fixtures/forwarded.eml");

fn output() -> langmail_core::ProcessedEmail {
    preprocess(FORWARDED_EML).unwrap()
}

#[test]
fn test_metadata() {
    let out = output();
    assert_eq!(out.from.as_ref().unwrap().email, "bob@example.com");
    assert_eq!(out.to[0].email, "charlie@example.com");
    assert_eq!(out.subject.as_deref(), Some("Fwd: Project update"));
    assert_eq!(out.date.as_deref(), Some("2026-02-10T15:30:00Z"));
}

#[test]
fn test_forwarder_intro_preserved() {
    let body = &output().body;
    assert!(
        body.contains("FYI, thought you should see this"),
        "forwarder's intro text should be preserved, body:\n{body}"
    );
}

#[test]
fn test_forwarded_content_preserved() {
    let body = &output().body;
    assert!(
        body.contains("project is on track"),
        "forwarded message body should not be discarded, body:\n{body}"
    );
    assert!(
        body.contains("Feature A is complete"),
        "forwarded message details should be present, body:\n{body}"
    );
    assert!(
        body.contains("Feature B is in review"),
        "forwarded message details should be present, body:\n{body}"
    );
}
