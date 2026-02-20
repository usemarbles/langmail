/// Convert HTML email body to clean plain text suitable for LLM processing.
///
/// This is intentionally opinionated about output format:
/// - Strips all HTML tags
/// - Preserves paragraph structure with blank lines
/// - Renders ordered/unordered lists with numbered/bulleted prefixes
/// - Converts common HTML entities
/// - Removes style/script blocks entirely
/// - Collapses excessive whitespace
pub fn html_to_clean_text(html: &str) -> String {
    let mut result = String::with_capacity(html.len() / 2);
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut tag_name = String::new();
    let mut collecting_tag_name = false;

    // Stack tracking list context: (is_ordered, current_counter)
    let mut list_context: Vec<(bool, u32)> = Vec::new();

    // Paragraph-level elements (open or close) → "\n\n"
    const PARA_ELEMENTS: &[&str] = &[
        "p",
        "div",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "blockquote",
        "pre",
        "table",
    ];
    // Line-level elements → "\n"
    const LINE_ELEMENTS: &[&str] = &["br", "tr", "td", "th"];
    // "hr" handled separately → 80-dash separator
    // "li" handled separately → numbered/bulleted prefix
    // "ol"/"ul" handled separately → list context stack

    let chars: Vec<char> = html.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        if ch == '<' {
            in_tag = true;
            tag_name.clear();
            collecting_tag_name = true;
            i += 1;
            continue;
        }

        if in_tag {
            if ch == '>' {
                in_tag = false;
                collecting_tag_name = false;

                let tag_lower = tag_name.to_lowercase();
                let is_closing = tag_lower.starts_with('/');
                let clean_tag = tag_lower
                    .trim_start_matches('/')
                    .trim_end_matches('/')
                    .trim()
                    .to_string();

                // Track script/style blocks
                if clean_tag == "script" {
                    in_script = !is_closing;
                } else if clean_tag == "style" {
                    in_style = !is_closing;
                } else if clean_tag == "ol" {
                    if is_closing {
                        list_context.pop();
                    } else {
                        list_context.push((true, 0));
                    }
                } else if clean_tag == "ul" {
                    if is_closing {
                        list_context.pop();
                    } else {
                        list_context.push((false, 0));
                    }
                } else if clean_tag == "li" && !is_closing {
                    if let Some(ctx) = list_context.last_mut() {
                        if ctx.0 {
                            ctx.1 += 1;
                            result.push_str(&format!("\n {}. ", ctx.1));
                        } else {
                            result.push_str("\n * ");
                        }
                    } else {
                        result.push_str("\n * ");
                    }
                } else if clean_tag == "hr" {
                    result.push_str("\n\n");
                    result.push_str(&"-".repeat(80));
                    result.push_str("\n\n");
                } else if PARA_ELEMENTS.contains(&clean_tag.as_str()) {
                    result.push_str("\n\n");
                } else if LINE_ELEMENTS.contains(&clean_tag.as_str()) {
                    result.push('\n');
                }
            } else if collecting_tag_name {
                if ch.is_whitespace() {
                    collecting_tag_name = false;
                } else {
                    tag_name.push(ch);
                }
            }
            i += 1;
            continue;
        }

        // Skip content inside script/style tags
        if in_script || in_style {
            i += 1;
            continue;
        }

        // Handle HTML entities
        if ch == '&' {
            // Cap scan to 10 chars to avoid treating bare `&` as entity start
            let scan_limit = std::cmp::min(chars.len() - i, 11);
            let entity_end = chars[i..i + scan_limit]
                .iter()
                .position(|&c| c == ';')
                .filter(|&end| {
                    // Reject if the candidate contains spaces or `<` (bare ampersand)
                    !chars[i + 1..i + end].iter().any(|&c| c == ' ' || c == '<')
                });
            if let Some(end) = entity_end {
                let entity: String = chars[i..=i + end].iter().collect();
                let decoded = decode_entity(&entity);
                result.push_str(&decoded);
                i += end + 1;
                continue;
            }
        }

        // Regular character
        if ch == '\n' || ch == '\r' {
            // Treat newlines as spaces in HTML context (collapse with adjacent whitespace)
            if !result.ends_with(' ') && !result.ends_with('\n') {
                result.push(' ');
            }
        } else if ch == ' ' || ch == '\t' {
            // Collapse consecutive spaces and spaces at the start of a line (after \n).
            // This correctly handles HTML text-node whitespace normalization and
            // prevents indentation between tags (e.g. "<p>\n  <a>") from leaking
            // into the output as a leading space.
            if !result.ends_with('\n') && !result.ends_with(' ') {
                result.push(' ');
            }
        } else {
            result.push(ch);
        }

        i += 1;
    }

    // Clean up: collapse whitespace, normalize line breaks
    collapse_whitespace(&result)
}

/// Decode common HTML entities.
fn decode_entity(entity: &str) -> String {
    match entity {
        "&amp;" => "&".to_string(),
        "&lt;" => "<".to_string(),
        "&gt;" => ">".to_string(),
        "&quot;" => "\"".to_string(),
        "&apos;" | "&#39;" => "'".to_string(),
        "&nbsp;" | "&#160;" => " ".to_string(),
        "&mdash;" | "&#8212;" => "—".to_string(),
        "&ndash;" | "&#8211;" => "–".to_string(),
        "&hellip;" | "&#8230;" => "…".to_string(),
        "&rsquo;" | "&#8217;" => "'".to_string(),
        "&lsquo;" | "&#8216;" => "'".to_string(),
        "&rdquo;" | "&#8221;" => "\u{201d}".to_string(),
        "&ldquo;" | "&#8220;" => "\u{201c}".to_string(),
        "&bull;" | "&#8226;" => "•".to_string(),
        "&copy;" | "&#169;" => "©".to_string(),
        "&reg;" | "&#174;" => "®".to_string(),
        "&trade;" | "&#8482;" => "™".to_string(),
        "&zwnj;" | "&#8204;" => "\u{200C}".to_string(),
        "&shy;" | "&#173;" => "\u{00AD}".to_string(),
        _ => {
            // Try numeric entities: &#NNN; or &#xHHH;
            if entity.starts_with("&#x") || entity.starts_with("&#X") {
                let hex = &entity[3..entity.len() - 1];
                if let Ok(code) = u32::from_str_radix(hex, 16) {
                    if let Some(ch) = char::from_u32(code) {
                        return ch.to_string();
                    }
                }
            } else if entity.starts_with("&#") {
                let num = &entity[2..entity.len() - 1];
                if let Ok(code) = num.parse::<u32>() {
                    if let Some(ch) = char::from_u32(code) {
                        return ch.to_string();
                    }
                }
            }
            // Unknown entity — return as-is
            entity.to_string()
        }
    }
}

/// Collapse runs of whitespace and normalize blank lines.
fn collapse_whitespace(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut consecutive_newlines = 0;
    let mut last_was_space = false;

    for ch in text.chars() {
        if ch == '\n' {
            consecutive_newlines += 1;
            last_was_space = false;
            if consecutive_newlines <= 2 {
                result.push('\n');
            }
        } else if ch.is_whitespace() {
            consecutive_newlines = 0;
            if !last_was_space {
                result.push(' ');
                last_was_space = true;
            }
        } else {
            consecutive_newlines = 0;
            last_was_space = false;
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_html() {
        let html = "<p>Hello <b>world</b>!</p>";
        let text = html_to_clean_text(html);
        assert!(text.contains("Hello world!"));
    }

    #[test]
    fn test_strips_script_and_style() {
        let html = r#"
            <style>body { color: red; }</style>
            <p>Visible text</p>
            <script>alert('hidden')</script>
        "#;
        let text = html_to_clean_text(html);
        assert!(text.contains("Visible text"));
        assert!(!text.contains("color: red"));
        assert!(!text.contains("alert"));
    }

    #[test]
    fn test_entities() {
        let html = "<p>Tom &amp; Jerry &mdash; classic</p>";
        let text = html_to_clean_text(html);
        assert!(text.contains("Tom & Jerry — classic"));
    }

    #[test]
    fn test_block_elements_produce_newlines() {
        let html = "<p>First paragraph</p><p>Second paragraph</p>";
        let text = html_to_clean_text(html);
        assert!(text.contains("First paragraph\n"));
        assert!(text.contains("Second paragraph"));
    }

    #[test]
    fn test_preserves_unicode() {
        let html = "<p>Héllo wörld 🌍</p>";
        let text = html_to_clean_text(html);
        assert!(text.contains("Héllo wörld 🌍"));
    }

    #[test]
    fn test_bare_ampersand_preserved() {
        let html = "<p>Security & access</p>";
        let text = html_to_clean_text(html);
        assert!(
            text.contains("Security & access"),
            "bare ampersand should be preserved, got: {text}"
        );
    }

    #[test]
    fn test_shy_entity_decoded() {
        let html = "<p>soft&shy;hyphen</p>";
        let text = html_to_clean_text(html);
        assert!(
            text.contains("soft\u{00AD}hyphen"),
            "&shy; should decode to U+00AD, got: {text}"
        );
    }

    #[test]
    fn test_ordered_list() {
        let html = "<ol><li>first</li><li>second</li><li>third</li></ol>";
        let text = html_to_clean_text(html);
        assert!(text.contains(" 1. first"), "got: {text}");
        assert!(text.contains(" 2. second"), "got: {text}");
        assert!(text.contains(" 3. third"), "got: {text}");
    }

    #[test]
    fn test_unordered_list() {
        let html = "<ul><li>alpha</li><li>beta</li></ul>";
        let text = html_to_clean_text(html);
        assert!(text.contains(" * alpha"), "got: {text}");
        assert!(text.contains(" * beta"), "got: {text}");
    }

    #[test]
    fn test_hr_produces_separator() {
        let html = "<p>before</p><hr/><p>after</p>";
        let text = html_to_clean_text(html);
        assert!(text.contains("--------"), "missing dashes, got: {text}");
        assert!(text.contains("before"), "got: {text}");
        assert!(text.contains("after"), "got: {text}");
    }

    #[test]
    fn test_self_closing_br() {
        let html = "line one<br/>line two";
        let text = html_to_clean_text(html);
        assert!(text.contains("line one\nline two"), "got: {text}");
    }

    #[test]
    fn test_para_elements_double_newline() {
        let html = "<p>first</p><p>second</p>";
        let text = html_to_clean_text(html);
        assert!(text.contains("first\n\nsecond"), "got: {text}");
    }

    #[test]
    fn test_no_leading_space_from_indented_inline_child() {
        // HTML indentation between a block element and its first inline child
        // must not produce a leading space on the output line.
        let html = "<p>\n  <a href=\"https://example.com\">Unsubscribe</a>\n</p>";
        let text = html_to_clean_text(html);
        assert!(
            text.contains("Unsubscribe"),
            "Unsubscribe text missing, got: {text}"
        );
        for line in text.lines() {
            if line.contains("Unsubscribe") {
                assert!(
                    !line.starts_with(' '),
                    "line with Unsubscribe should not have a leading space, got: {line:?}"
                );
            }
        }
    }

    #[test]
    fn test_inline_spaces_within_paragraph_preserved() {
        // Spaces between words inside a paragraph must still be preserved.
        let html = "<p>hello world and more</p>";
        let text = html_to_clean_text(html);
        assert!(
            text.contains("hello world and more"),
            "word spacing lost, got: {text}"
        );
    }
}
