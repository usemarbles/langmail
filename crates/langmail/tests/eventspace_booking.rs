use langmail::preprocess;

static EVENTSPACE_EML: &[u8] = include_bytes!("fixtures/eventspace-booking.eml");

#[test]
fn metadata_from() {
    let output = preprocess(EVENTSPACE_EML).unwrap();
    let from = output.from.as_ref().unwrap();
    assert_eq!(from.email, "noreply@example.com");
    assert_eq!(from.name.as_deref(), Some("Max Mustermann"));
}

#[test]
fn metadata_to() {
    let output = preprocess(EVENTSPACE_EML).unwrap();
    assert_eq!(output.to.len(), 1);
    assert_eq!(output.to[0].email, "test@example.com");
}

#[test]
fn metadata_cc() {
    let output = preprocess(EVENTSPACE_EML).unwrap();
    assert_eq!(output.cc.len(), 2);
    let cc_emails: Vec<&str> = output.cc.iter().map(|a| a.email.as_str()).collect();
    assert!(cc_emails.contains(&"colleague1@example.com"));
    assert!(cc_emails.contains(&"colleague2@example.com"));
}

#[test]
fn metadata_subject() {
    let output = preprocess(EVENTSPACE_EML).unwrap();
    assert_eq!(output.subject.as_deref(), Some("Re: Buchung Eventspace"));
}

#[test]
fn metadata_date() {
    let output = preprocess(EVENTSPACE_EML).unwrap();
    assert_eq!(output.date.as_deref(), Some("2026-03-02T15:41:22Z"));
}

#[test]
fn metadata_threading() {
    let output = preprocess(EVENTSPACE_EML).unwrap();
    assert!(output.rfc_message_id.is_some());
    assert!(output.in_reply_to.is_some());
    assert!(output.references.is_some());
    assert_eq!(output.references.as_ref().unwrap().len(), 3);
}

#[test]
fn body_contains_top_level_reply() {
    let output = preprocess(EVENTSPACE_EML).unwrap();
    assert!(
        output
            .body
            .contains("korrekt, in diesem Fall gilt auch der Samstag als Werktag"),
        "top-level reply content missing from body"
    );
    assert!(
        output
            .body
            .contains("An welche Adresse soll das Angebot ausgestellt werden"),
        "follow-up question missing from body"
    );
}

#[test]
fn body_does_not_contain_quoted_replies() {
    let output = preprocess(EVENTSPACE_EML).unwrap();
    assert!(
        !output.body.contains("Super, danke dir"),
        "quoted reply should be stripped"
    );
    assert!(
        !output.body.contains("Liebes VENUE1"),
        "original inquiry should be stripped"
    );
    assert!(
        !output.body.contains("Event Space noch verfügbar"),
        "availability reply should be stripped"
    );
}

#[test]
fn signature_is_stripped() {
    let output = preprocess(EVENTSPACE_EML).unwrap();
    assert!(
        output.signature.is_some(),
        "signature should be detected and extracted"
    );
    assert!(
        !output.body.contains("VENUE1 Events Team"),
        "signature block (team name) should not appear in body"
    );
    assert!(
        !output.body.contains("Thomas Schmidt, Anna Mueller"),
        "signature block (team members) should not appear in body"
    );
    assert!(
        !output.body.contains("Venue1 GmbH"),
        "company legal info should not appear in body"
    );
    assert!(
        !output.body.contains("Musterstrasse 42"),
        "company address should not appear in body"
    );
    assert!(
        !output.body.contains("Gefördert durch"),
        "promotional footer should not appear in body"
    );
}

#[test]
fn body_ends_after_message_content() {
    let output = preprocess(EVENTSPACE_EML).unwrap();
    // The actual message content ends with the questions about the offer
    assert!(
        output.body.contains("Ist Catering oder sind Getränke gewünscht"),
        "body should contain the last question"
    );
    // Body should end around the sign-off, not contain the long signature
    assert!(
        output.body.lines().count() < 15,
        "body should be short (got {} lines): {}",
        output.body.lines().count(),
        output.body
    );
}

#[test]
fn german_quote_header_stripped() {
    let output = preprocess(EVENTSPACE_EML).unwrap();
    assert!(
        !output.body.contains("Am Fr., 27. Feb. 2026 um 11:53"),
        "German quote header should be stripped"
    );
}
