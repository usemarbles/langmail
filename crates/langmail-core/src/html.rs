/// Convert HTML email body to clean plain text suitable for LLM processing.
///
/// This is intentionally opinionated about output format:
/// - Strips all HTML tags
/// - Preserves paragraph structure with single blank lines
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
    let mut last_was_block = false;

    // Block-level elements that should produce line breaks
    const BLOCK_ELEMENTS: &[&str] = &[
        "p",
        "div",
        "br",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "li",
        "tr",
        "blockquote",
        "pre",
        "hr",
        "table",
    ];

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
                let clean_tag = tag_lower.trim_start_matches('/').to_string();

                // Track script/style blocks
                if clean_tag == "script" {
                    in_script = !is_closing;
                } else if clean_tag == "style" {
                    in_style = !is_closing;
                }

                // Add line break for block elements
                if BLOCK_ELEMENTS.contains(&clean_tag.as_str()) && !last_was_block {
                    result.push('\n');
                    last_was_block = true;
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
            let entity_end = chars[i..].iter().position(|&c| c == ';');
            if let Some(end) = entity_end {
                let entity: String = chars[i..=i + end].iter().collect();
                let decoded = decode_entity(&entity);
                result.push_str(&decoded);
                last_was_block = false;
                i += end + 1;
                continue;
            }
        }

        // Regular character
        if ch == '\n' || ch == '\r' {
            // Treat newlines as spaces in HTML context
            if !result.ends_with(' ') && !result.ends_with('\n') {
                result.push(' ');
            }
        } else {
            result.push(ch);
            last_was_block = false;
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
}
