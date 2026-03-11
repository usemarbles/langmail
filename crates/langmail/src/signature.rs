use once_cell::sync::Lazy;
use regex::Regex;

/// Maximum number of lines a short signature can span (delimiter-based).
/// Most email signatures are 1-8 lines; we're generous with 10.
const MAX_SIGNATURE_LINES: usize = 10;

/// Maximum number of lines a corporate signature can span after a sign-off.
/// Corporate signatures often include name, title, company, address,
/// phone numbers, promotional links, and legal disclaimers.
const MAX_CORPORATE_SIGNATURE_LINES: usize = 60;

/// Patterns that indicate the start of an email signature.
static SIGNATURE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    [
        // Standard delimiter: "-- " (note the trailing space per RFC 3676)
        r"(?m)^-- \s*$",
        // Common variant without trailing space
        r"(?m)^--\s*$",
        // "Sent from" patterns (mobile devices)
        r"(?mi)^Sent from my ",
        r"(?mi)^Sent from Mail for ",
        r"(?mi)^Sent from Outlook",
        r"(?mi)^Sent via ",
        r"(?mi)^Gesendet von ",     // German
        r"(?mi)^Envoyé de mon ",    // French
        r"(?mi)^Enviado desde mi ", // Spanish
        // "Get Outlook for" / app promos
        r"(?mi)^Get Outlook for ",
        // Common sign-offs followed by a name on the next line
        // (handled heuristically in the main function)
    ]
    .iter()
    .filter_map(|pattern| Regex::new(pattern).ok())
    .collect()
});

/// Sign-off phrases that may indicate the beginning of a signature block.
/// These are only used when they appear near the end of the message.
static SIGNOFF_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    [
        r"(?mi)^(Best( regards)?|Kind regards|Regards|Cheers|Thanks|Thank you|Sincerely|Yours truly|Warm regards|With thanks|Many thanks|All the best|Take care),?\s*$",
        r"(?mi)^(Mit freundlichen Grüßen|Beste Grüße|Viele Grüße|Liebe Grüße|MfG),?\s*$",  // German
        r"(?mi)^(Cordialement|Bien cordialement|Merci|Cdlt),?\s*$",              // French
        r"(?mi)^(Saludos|Atentamente|Gracias),?\s*$",                            // Spanish
    ]
    .iter()
    .filter_map(|pattern| Regex::new(pattern).ok())
    .collect()
});

/// Extract signature from the email body.
///
/// Returns a tuple of (body_without_signature, optional_signature).
/// Uses a combination of delimiter detection and heuristic analysis.
pub fn extract_signature(body: &str) -> (String, Option<String>) {
    let lines: Vec<&str> = body.lines().collect();

    if lines.is_empty() {
        return (body.to_string(), None);
    }

    // Strategy 1: Look for explicit signature delimiters
    for pattern in SIGNATURE_PATTERNS.iter() {
        if let Some(m) = pattern.find(body) {
            let remaining_lines = body[m.start()..].lines().count();
            if remaining_lines <= MAX_SIGNATURE_LINES {
                let text = body[..m.start()].to_string();
                let sig = body[m.start()..].trim().to_string();
                return (text, Some(sig));
            }
        }
    }

    // Strategy 2: Look for sign-off patterns near the end of the message
    let search_window = lines.len().saturating_sub(MAX_CORPORATE_SIGNATURE_LINES);
    for (i, _line) in lines[search_window..].iter().enumerate() {
        let abs_idx = search_window + i;

        for pattern in SIGNOFF_PATTERNS.iter() {
            if let Some(line_text) = lines.get(abs_idx) {
                if pattern.is_match(line_text) {
                    let remaining = &lines[abs_idx..];
                    if looks_like_signature(remaining) {
                        let text = lines[..abs_idx].join("\n");
                        let sig = remaining.join("\n");
                        return (text, Some(sig));
                    }
                }
            }
        }
    }

    (body.to_string(), None)
}

/// Heuristic check: does this block of lines look like an email signature?
///
/// Signatures tend to have relatively short lines (not full paragraphs).
/// Short signatures (≤10 lines) just need a name line after the sign-off.
/// Longer corporate signatures (up to 60 lines) must also contain typical
/// signature markers like phone numbers, URLs, email addresses, or company info.
fn looks_like_signature(lines: &[&str]) -> bool {
    if lines.is_empty() || lines.len() > MAX_CORPORATE_SIGNATURE_LINES {
        return false;
    }

    // All lines should be relatively short (signatures aren't paragraphs).
    // Corporate signatures with promo lines can be up to ~200 chars.
    let all_short = lines.iter().all(|l| l.len() < 200);

    // At least one non-empty line after the sign-off
    let has_content = lines.iter().skip(1).any(|l| !l.trim().is_empty());

    if !all_short || !has_content {
        // Still allow a bare sign-off (e.g., "Best,\n")
        return lines.len() <= 2;
    }

    // Short signatures pass with basic checks
    if lines.len() <= MAX_SIGNATURE_LINES {
        return true;
    }

    // Longer blocks must look like a corporate signature:
    // they should contain typical markers like phone, email, URL, or company info.
    has_corporate_signature_markers(lines)
}

/// Check whether a block of lines contains typical corporate signature markers.
fn has_corporate_signature_markers(lines: &[&str]) -> bool {
    let joined = lines.join("\n");
    let has_phone = PHONE_PATTERN.is_match(&joined);
    let has_email = EMAIL_PATTERN.is_match(&joined);
    let has_url = joined.contains("http://") || joined.contains("https://") || joined.contains("www.");

    // Need at least 2 of these markers to qualify as corporate sig
    let marker_count = has_phone as u8 + has_email as u8 + has_url as u8;
    marker_count >= 2
}

static PHONE_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\+?\d[\d\s\-()]{6,}").unwrap());

static EMAIL_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_delimiter() {
        let body = "Main content here.\n\n-- \nJohn Doe\nCEO, Acme Corp\n+1 555-0123\n";
        let (text, sig) = extract_signature(body);
        assert!(text.contains("Main content here."));
        assert!(sig.is_some());
        assert!(sig.unwrap().contains("John Doe"));
    }

    #[test]
    fn test_sent_from_mobile() {
        let body = "Quick reply.\n\nSent from my iPhone\n";
        let (text, sig) = extract_signature(body);
        assert!(text.contains("Quick reply."));
        assert!(sig.is_some());
        assert!(sig.unwrap().contains("Sent from my iPhone"));
    }

    #[test]
    fn test_signoff_pattern() {
        let body =
            "Let me know if you have questions.\n\nBest regards,\nAlice Smith\nVP Engineering\n";
        let (text, sig) = extract_signature(body);
        assert!(text.contains("Let me know"));
        assert!(sig.is_some());
        let sig_text = sig.unwrap();
        assert!(sig_text.contains("Best regards"));
        assert!(sig_text.contains("Alice Smith"));
    }

    #[test]
    fn test_no_signature() {
        let body = "Just a plain message with no signature.\n\nSecond paragraph too.\n";
        let (text, sig) = extract_signature(body);
        assert_eq!(text, body);
        assert!(sig.is_none());
    }

    #[test]
    fn test_german_signoff() {
        let body = "Bitte senden Sie die Unterlagen.\n\nMit freundlichen Grüßen\nHans Schmidt\n";
        let (text, sig) = extract_signature(body);
        assert!(text.contains("Bitte senden"));
        assert!(sig.is_some());
    }

    #[test]
    fn test_dash_delimiter_without_space() {
        let body = "Here is my response.\n\n--\nBob Jones\nbob@company.com\n";
        let (text, sig) = extract_signature(body);
        assert!(text.contains("Here is my response."));
        assert!(sig.is_some());
    }

    #[test]
    fn test_german_beste_gruesse_signoff() {
        let body = "Hier ist meine Antwort.\n\nBeste Grüße,\nThomas\n";
        let (text, sig) = extract_signature(body);
        assert!(text.contains("Hier ist meine Antwort."));
        assert!(sig.is_some());
        assert!(sig.unwrap().contains("Beste Grüße"));
    }

    #[test]
    fn test_corporate_signature_with_markers() {
        let body = "Message content.\n\nBest regards,\nJane Doe\nCTO\n\nAcme Corp\n123 Main St\n+1 555-0100\njane@acme.com\nhttps://acme.com\n\nDisclaimer text.\nMore legal.\nLine 3.\nLine 4.\nLine 5.\nLine 6.\n";
        let (text, sig) = extract_signature(body);
        assert!(text.contains("Message content."));
        assert!(sig.is_some());
        let sig_text = sig.unwrap();
        assert!(sig_text.contains("Best regards"));
        assert!(sig_text.contains("Acme Corp"));
    }

    #[test]
    fn test_long_block_without_markers_not_stripped() {
        // A long block after a sign-off that doesn't look like a corporate sig
        // (no phone, no email, no URL) should not be stripped
        let mut body = "Message content.\n\nBest regards,\nSomeone\n".to_string();
        for i in 0..20 {
            body.push_str(&format!("Random line {i}\n"));
        }
        let (text, sig) = extract_signature(&body);
        assert!(text.contains("Message content."));
        assert!(sig.is_none(), "should not strip long block without corporate markers");
    }
}
