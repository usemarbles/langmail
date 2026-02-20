use htmd::{Element, HtmlToMarkdown};
use htmd::element_handler::Handlers;

/// Convert an HTML email body to clean Markdown suitable for LLM processing.
///
/// Uses htmd for proper HTML parsing and Markdown conversion, configured for
/// email-specific needs:
/// - Skips non-semantic tags (script, style, img, head, svg)
/// - Drops anchor hrefs (email links are almost always tracking URLs)
/// - Applies whitespace normalisation as a post-processing step
pub fn html_to_markdown(html: &str) -> String {
    let converter = HtmlToMarkdown::builder()
        .skip_tags(vec!["script", "style", "img", "head", "svg"])
        // Render only the link text; drop the href entirely.
        // Email links are almost always opaque tracking URLs with no semantic
        // value for LLM consumption.
        .add_handler(vec!["a"], |handlers: &dyn Handlers, element: Element| {
            Some(handlers.walk_children(element.node))
        })
        // HTML emails use <table> for layout, not data. Rendering them as
        // Markdown tables escapes every `|` in cell content as `&#124;` and
        // produces unreadable output. Override the entire table family to
        // simply walk children, with each cell treated as a paragraph.
        .add_handler(
            vec!["table", "thead", "tbody", "tfoot", "tr"],
            |handlers: &dyn Handlers, element: Element| {
                Some(handlers.walk_children(element.node))
            },
        )
        .add_handler(
            vec!["td", "th"],
            |handlers: &dyn Handlers, element: Element| {
                let content = handlers.walk_children(element.node).content;
                let trimmed = content.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(format!("\n\n{trimmed}\n\n").into())
                }
            },
        )
        .build();

    let md = converter.convert(html).unwrap_or_default();
    collapse_whitespace(&md)
}

/// Collapse runs of whitespace and limit consecutive blank lines to two.
///
/// Multiple spaces collapse to one; more than two consecutive newlines collapse
/// to two. This keeps the Markdown readable without destroying structure.
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
        let text = html_to_markdown(html);
        assert!(text.contains("Hello **world**!"), "got: {text}");
    }

    #[test]
    fn test_strips_script_and_style() {
        let html = r#"
            <style>body { color: red; }</style>
            <p>Visible text</p>
            <script>alert('hidden')</script>
        "#;
        let text = html_to_markdown(html);
        assert!(text.contains("Visible text"));
        assert!(!text.contains("color: red"));
        assert!(!text.contains("alert"));
    }

    #[test]
    fn test_entities() {
        let html = "<p>Tom &amp; Jerry &mdash; classic</p>";
        let text = html_to_markdown(html);
        assert!(text.contains("Tom & Jerry — classic"));
    }

    #[test]
    fn test_block_elements_produce_newlines() {
        let html = "<p>First paragraph</p><p>Second paragraph</p>";
        let text = html_to_markdown(html);
        assert!(text.contains("First paragraph\n"));
        assert!(text.contains("Second paragraph"));
    }

    #[test]
    fn test_preserves_unicode() {
        let html = "<p>Héllo wörld 🌍</p>";
        let text = html_to_markdown(html);
        assert!(text.contains("Héllo wörld 🌍"));
    }

    #[test]
    fn test_bare_ampersand_preserved() {
        let html = "<p>Security &amp; access</p>";
        let text = html_to_markdown(html);
        assert!(
            text.contains("Security & access"),
            "ampersand should be preserved, got: {text}"
        );
    }

    #[test]
    fn test_anchor_text_preserved_href_dropped() {
        // Anchor links should render as plain text — tracking URLs are dropped.
        let html = r#"<p><a href="https://tracking.example.com/click/abc123">Click here</a></p>"#;
        let text = html_to_markdown(html);
        assert!(text.contains("Click here"), "link text missing, got: {text}");
        assert!(
            !text.contains("tracking.example.com"),
            "href should be dropped, got: {text}"
        );
        assert!(
            !text.contains("]("),
            "no Markdown link syntax expected, got: {text}"
        );
    }

    #[test]
    fn test_ordered_list() {
        let html = "<ol><li>first</li><li>second</li><li>third</li></ol>";
        let text = html_to_markdown(html);
        assert!(text.contains("1. first"), "got: {text}");
        assert!(text.contains("2. second"), "got: {text}");
        assert!(text.contains("3. third"), "got: {text}");
    }

    #[test]
    fn test_unordered_list() {
        let html = "<ul><li>alpha</li><li>beta</li></ul>";
        let text = html_to_markdown(html);
        assert!(text.contains("* alpha"), "got: {text}");
        assert!(text.contains("* beta"), "got: {text}");
    }

    #[test]
    fn test_hr_produces_separator() {
        let html = "<p>before</p><hr/><p>after</p>";
        let text = html_to_markdown(html);
        assert!(text.contains("* * *"), "missing HR, got: {text}");
        assert!(text.contains("before"), "got: {text}");
        assert!(text.contains("after"), "got: {text}");
    }

    #[test]
    fn test_self_closing_br() {
        let html = "line one<br/>line two";
        let text = html_to_markdown(html);
        assert!(text.contains("line one"), "got: {text}");
        assert!(text.contains("line two"), "got: {text}");
        // Both parts must be on separate lines (br produces a newline)
        let line_one_pos = text.find("line one").unwrap();
        let line_two_pos = text.find("line two").unwrap();
        assert!(
            text[line_one_pos..line_two_pos].contains('\n'),
            "br should produce a newline between the two lines, got: {text}"
        );
    }

    #[test]
    fn test_para_elements_double_newline() {
        let html = "<p>first</p><p>second</p>";
        let text = html_to_markdown(html);
        assert!(text.contains("first\n\nsecond"), "got: {text}");
    }

    #[test]
    fn test_no_leading_space_from_indented_inline_child() {
        // HTML indentation between a block element and its first inline child
        // must not produce a leading space on the output line.
        let html = "<p>\n  <a href=\"https://example.com\">Unsubscribe</a>\n</p>";
        let text = html_to_markdown(html);
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
    fn test_layout_table_content_preserved_without_pipe_escaping() {
        // HTML emails use <table> for layout. Content must be extracted as
        // plain paragraphs; pipe characters must never be escaped as &#124;.
        let html = r#"
            <table>
              <tr>
                <td><p>Hello from cell one</p></td>
                <td><p>Cell two | with | pipes</p></td>
              </tr>
              <tr>
                <td><p>Row two content</p></td>
                <td></td>
              </tr>
            </table>
        "#;
        let text = html_to_markdown(html);
        assert!(text.contains("Hello from cell one"), "got: {text}");
        assert!(
            text.contains("Cell two | with | pipes"),
            "pipe characters should be unescaped, got: {text}"
        );
        assert!(text.contains("Row two content"), "got: {text}");
        assert!(
            !text.contains("&#124;"),
            "pipe should not be escaped as &#124;, got: {text}"
        );
        // No Markdown table syntax
        assert!(!text.contains("| ---"), "no table divider expected, got: {text}");
    }

    #[test]
    fn test_inline_spaces_within_paragraph_preserved() {
        let html = "<p>hello world and more</p>";
        let text = html_to_markdown(html);
        assert!(
            text.contains("hello world and more"),
            "word spacing lost, got: {text}"
        );
    }
}
