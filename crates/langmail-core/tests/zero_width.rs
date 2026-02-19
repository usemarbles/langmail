use langmail_core::preprocess;

static ZERO_WIDTH_EML: &[u8] = include_bytes!("fixtures/zero-width.eml");

#[test]
fn test_zero_width_characters_stripped_from_body() {
    let output = preprocess(ZERO_WIDTH_EML).unwrap();
    assert!(
        !output.body.contains('\u{200C}'),
        "body should not contain U+200C (ZWNJ), but found in:\n{}",
        output.body
    );
    assert!(
        !output.body.contains('\u{200B}'),
        "body should not contain U+200B (ZWSP)"
    );
    assert!(
        !output.body.contains('\u{200D}'),
        "body should not contain U+200D (ZWJ)"
    );
    assert!(
        !output.body.contains('\u{FEFF}'),
        "body should not contain U+FEFF (BOM/ZWNBSP)"
    );
}

#[test]
fn test_zero_width_no_html_entity_remnants() {
    let output = preprocess(ZERO_WIDTH_EML).unwrap();
    assert!(
        !output.body.contains("&zwnj;"),
        "body should not contain &zwnj; HTML entities, but found in:\n{}",
        output.body
    );
}
