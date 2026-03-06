use langmail::preprocess;

const RAW: &[u8] = include_bytes!("fixtures/linear-login.eml");

/// Characters that clean_invisible_characters strips.
const INVISIBLE_CHARS: &[char] = &[
    '\u{034F}', // COMBINING GRAPHEME JOINER
    '\u{200B}', // ZERO WIDTH SPACE
    '\u{200C}', // ZERO WIDTH NON-JOINER
    '\u{200D}', // ZERO WIDTH JOINER
    '\u{FEFF}', // ZERO WIDTH NO-BREAK SPACE (BOM)
    '\u{00AD}', // SOFT HYPHEN
    '\u{2007}', // FIGURE SPACE
];

#[test]
fn no_invisible_characters() {
    let output = preprocess(RAW).unwrap();
    for c in INVISIBLE_CHARS {
        assert!(
            !output.body.contains(*c),
            "body contains invisible char U+{:04X}",
            *c as u32
        );
    }
}

#[test]
fn no_whitespace_only_lines() {
    let output = preprocess(RAW).unwrap();
    for (i, line) in output.body.lines().enumerate() {
        assert!(
            line.is_empty() || !line.trim().is_empty(),
            "line {} is whitespace-only: {:?}",
            i + 1,
            line
        );
    }
}

#[test]
fn no_excessive_empty_lines() {
    let output = preprocess(RAW).unwrap();
    assert!(
        !output.body.contains("\n\n\n"),
        "body contains 3+ consecutive newlines:\n{}",
        output.body
    );
}

#[test]
fn content_preserved() {
    let output = preprocess(RAW).unwrap();
    assert!(
        output.body.contains("New login to Max"),
        "missing 'New login to Max'"
    );
    assert!(
        output.body.contains("Platform: Max Desktop on macOS"),
        "missing platform info"
    );
    assert!(
        output.body.contains("Munich, BY, DE"),
        "missing location info"
    );
    assert!(
        output.body.contains("Security & access"),
        "missing security link text"
    );
}
