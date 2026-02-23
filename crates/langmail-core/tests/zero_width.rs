use langmail_core::preprocess;

static ZERO_WIDTH_EML: &[u8] = include_bytes!("fixtures/zero-width.eml");

#[test]
fn test_no_invisible_characters_in_body() {
    let output = preprocess(ZERO_WIDTH_EML).unwrap();
    let invisible = [
        ('\u{034F}', "COMBINING GRAPHEME JOINER"),
        ('\u{200B}', "ZERO WIDTH SPACE"),
        ('\u{200C}', "ZERO WIDTH NON-JOINER"),
        ('\u{200D}', "ZERO WIDTH JOINER"),
        ('\u{FEFF}', "ZERO WIDTH NO-BREAK SPACE"),
        ('\u{00AD}', "SOFT HYPHEN"),
        ('\u{2007}', "FIGURE SPACE"),
    ];
    for (ch, name) in invisible {
        assert!(
            !output.body.contains(ch),
            "body should not contain U+{:04X} ({name})",
            ch as u32,
        );
    }
}

#[test]
fn test_no_html_entity_remnants() {
    let output = preprocess(ZERO_WIDTH_EML).unwrap();
    assert!(
        !output.body.contains("&zwnj;"),
        "body should not contain &zwnj; HTML entities, but found in:\n{}",
        output.body
    );
}

#[test]
fn test_nbsp_converted_to_regular_space() {
    let output = preprocess(ZERO_WIDTH_EML).unwrap();
    assert!(
        !output.body.contains('\u{00A0}'),
        "body should not contain non-breaking spaces (U+00A0)"
    );
}

#[test]
fn test_html_content_extracted() {
    let output = preprocess(ZERO_WIDTH_EML).unwrap();
    assert!(
        output.body.contains("Guten Tag Max Mustermann"),
        "body should contain the actual message from HTML part, got:\n{}",
        output.body
    );
}

#[test]
fn test_no_excessive_empty_lines() {
    let output = preprocess(ZERO_WIDTH_EML).unwrap();
    assert!(
        !output.body.contains("\n\n\n"),
        "body should not contain 3+ consecutive newlines, but found in:\n{}",
        output.body
    );
}

#[test]
fn test_date_converted_to_utc() {
    // Email Date header: Tue, 02 Dec 2025 08:53:35 -0600
    // UTC equivalent:    2025-12-02T14:53:35Z  (+6h for -0600 offset)
    let output = preprocess(ZERO_WIDTH_EML).unwrap();
    assert_eq!(
        output.date.as_deref(),
        Some("2025-12-02T14:53:35Z"),
        "date should be converted from local timezone to UTC"
    );
}
