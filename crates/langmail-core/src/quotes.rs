use once_cell::sync::Lazy;
use regex::Regex;

/// Patterns that indicate the start of a quoted reply section.
///
/// These cover the most common email clients:
/// - Gmail: "On <date>, <name> <email> wrote:"
/// - Outlook: "-----Original Message-----" or "From: ... Sent: ..."
/// - Apple Mail: "On <date>, at <time>, <name> wrote:"
/// - Generic: "> " prefixed lines
/// - Various localized patterns (German, French, Spanish)
///
/// Forwarded messages are intentionally NOT matched here. Unlike quoted
/// replies (which repeat already-seen content), a forwarded message carries
/// new information and must be preserved for LLM consumption.
static QUOTE_HEADERS: Lazy<Vec<Regex>> = Lazy::new(|| {
    [
        // Gmail / generic: "On <date>, <name> wrote:"
        r"(?m)^On .{10,80} wrote:\s*$",
        // Outlook: "-----Original Message-----"
        r"(?m)^-{2,}\s*Original Message\s*-{2,}\s*$",
        // Outlook-style header block: "From: ... Sent: ..."
        r"(?m)^From:\s+.+\nSent:\s+",
        // Apple Mail: "On <date>, at <time>,"
        r"(?m)^On .{10,80}, at .{4,20}, .{1,60} wrote:\s*$",
        // German: "Am <date> schrieb <name>:"
        r"(?m)^Am .{10,80} schrieb .{1,60}:\s*$",
        // French: "Le <date>, <name> a écrit :"
        r"(?m)^Le .{10,80} a écrit\s*:\s*$",
        // Spanish: "El <date>, <name> escribió:"
        r"(?m)^El .{10,80} escribi[óo]\s*:\s*$",
        // Generic "wrote:" at end of line
        r"(?m)^.{10,120} wrote:\s*$",
        // Line of ">" quoted text after a blank line
        r"(?m)^\s*\n(>{1}\s?.+\n){2,}",
    ]
    .iter()
    .filter_map(|pattern| Regex::new(pattern).ok())
    .collect()
});

/// Strip quoted reply content from an email body.
///
/// Returns just the new/original content from the most recent message,
/// removing quoted text from previous messages in the thread.
pub fn strip_quotes(body: &str) -> String {
    let mut earliest_quote_start = body.len();

    for pattern in QUOTE_HEADERS.iter() {
        if let Some(m) = pattern.find(body) {
            if m.start() < earliest_quote_start {
                earliest_quote_start = m.start();
            }
        }
    }

    // Also handle the case where the entire email is ">" quoted lines
    // from the very beginning (bottom-posting). In that case, try to
    // find the non-quoted portion.
    if earliest_quote_start == 0 {
        // Check if there's any non-quoted content
        let lines: Vec<&str> = body.lines().collect();
        let first_non_quoted = lines
            .iter()
            .position(|line| !line.starts_with('>') && !line.trim().is_empty());

        if let Some(idx) = first_non_quoted {
            // Find the end of the non-quoted section
            let end = lines[idx..]
                .iter()
                .position(|line| line.starts_with('>'))
                .map(|e| e + idx)
                .unwrap_or(lines.len());

            return lines[idx..end].join("\n");
        }
    }

    body[..earliest_quote_start].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gmail_quote() {
        let body = "Thanks for the update!\n\nOn Thu, Feb 5, 2026 at 10:00 AM Alice <alice@example.com> wrote:\n> Original message here\n> More text\n";
        let result = strip_quotes(body);
        assert!(result.contains("Thanks for the update!"));
        assert!(!result.contains("Original message here"));
    }

    #[test]
    fn test_outlook_quote() {
        let body = "Sounds good to me.\n\n-----Original Message-----\nFrom: Alice\nSent: Thursday\nSubject: Hello\n\nOriginal text\n";
        let result = strip_quotes(body);
        assert!(result.contains("Sounds good to me."));
        assert!(!result.contains("Original text"));
    }

    #[test]
    fn test_no_quotes() {
        let body = "Just a plain email with no quoted content.\n\nSecond paragraph.\n";
        let result = strip_quotes(body);
        assert_eq!(result, body);
    }

    #[test]
    fn test_german_quote_header() {
        let body = "Danke für die Info!\n\nAm 05.02.2026 um 10:00 schrieb Alice Müller:\n> Originalnachricht\n";
        let result = strip_quotes(body);
        assert!(result.contains("Danke"));
        assert!(!result.contains("Originalnachricht"));
    }

    #[test]
    fn test_forwarded_message_preserved() {
        // Forwarded messages are NOT stripped — they carry new information.
        let body = "FYI, see below.\n\n-------- Forwarded Message --------\nFrom: Alice\nSubject: Hello\n\nForwarded content\n";
        let result = strip_quotes(body);
        assert!(result.contains("FYI"), "intro should be present");
        assert!(
            result.contains("Forwarded content"),
            "forwarded body should not be stripped"
        );
    }

    #[test]
    fn test_generic_wrote_pattern() {
        let body = "I agree.\n\nJohn Doe <john@example.com> wrote:\n> Some quoted text\n";
        let result = strip_quotes(body);
        assert!(result.contains("I agree."));
        assert!(!result.contains("Some quoted text"));
    }
}
