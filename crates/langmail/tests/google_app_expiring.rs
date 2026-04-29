use langmail::preprocess;

static EML: &[u8] = include_bytes!("../../../fixtures/google-app-access-expiring.eml");

#[test]
fn test_implausible_date_falls_back_to_received() {
    let output = preprocess(EML).unwrap();
    let date = output.date.expect("email should have a date");

    assert!(
        date.starts_with("2026-03-08"),
        "expected fallback to Received header date in March 2026, got {date}"
    );
}
