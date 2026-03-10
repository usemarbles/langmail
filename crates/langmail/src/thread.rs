use once_cell::sync::Lazy;
use regex::Regex;
use scraper::{Html, Node, Selector};

use crate::html::html_to_markdown;
use crate::signature;
use crate::types::ThreadMessage;
use crate::{collapse_empty_lines, trim_whitespace_lines};

/// Patterns matching quote attribution lines, capturing timestamp and sender.
static ATTRIBUTION_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    [
        // German: "Am <date> um <time> Uhr schrieb <sender>:"
        r"Am\s+.+?(\d{1,2})\.\s*(\w+)\.?\s+(\d{4})\s+um\s+(\d{1,2}):(\d{2})\s*Uhr\s+schrieb\s+(.+?):",
        // English: "On <day>, <date> at <time>, <sender> wrote:"
        r"On\s+\w+,\s+(\d{1,2})\s+(\w+)\s+(\d{4})\s+at\s+(\d{1,2}):(\d{2}),\s*(.+?)\s+wrote:",
        // French: "Le <date> à <time>, <sender> a écrit :"
        r"Le\s+.+?(\d{1,2})\s+(\w+)\.?\s+(\d{4})\s+à\s+(\d{1,2}):(\d{2}),?\s*(.+?)\s+a\s+écrit\s*:",
        // Spanish: "El <date> a las <time>, <sender> escribió:"
        r"El\s+.+?(\d{1,2})\s+de\s+(\w+)\.?\s+de\s+(\d{4})\s+a\s+las\s+(\d{1,2}):(\d{2}),?\s*(.+?)\s+escribió\s*:",
    ]
    .iter()
    .filter_map(|p| Regex::new(p).ok())
    .collect()
});

fn month_to_number(month: &str) -> Option<u32> {
    let m = month.to_lowercase();
    match m.as_str() {
        "jan" | "january" | "januar" | "janvier" | "enero" => Some(1),
        "feb" | "february" | "februar" | "février" | "febrero" => Some(2),
        "mar" | "march" | "märz" | "mär" | "mars" | "marzo" => Some(3),
        "apr" | "april" | "avril" | "abril" => Some(4),
        "may" | "mai" | "mayo" => Some(5),
        "jun" | "june" | "juni" | "juin" | "junio" => Some(6),
        "jul" | "july" | "juli" | "juillet" | "julio" => Some(7),
        "aug" | "august" | "août" | "agosto" => Some(8),
        "sep" | "september" | "septembre" | "septiembre" => Some(9),
        "oct" | "october" | "okt" | "oktober" | "octobre" | "octubre" => Some(10),
        "nov" | "november" | "novembre" | "noviembre" => Some(11),
        "dec" | "december" | "dez" | "dezember" | "décembre" | "diciembre" => Some(12),
        _ => None,
    }
}

fn parse_timestamp(day: &str, month: &str, year: &str, hour: &str, minute: &str) -> Option<String> {
    let d: u32 = day.parse().ok()?;
    let m = month_to_number(month)?;
    let y: u32 = year.parse().ok()?;
    let h: u32 = hour.parse().ok()?;
    let min: u32 = minute.parse().ok()?;
    Some(format!("{y:04}-{m:02}-{d:02}T{h:02}:{min:02}:00"))
}

fn parse_sender(raw: &str) -> String {
    raw.trim()
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

/// Parse attribution text to extract sender and timestamp.
fn parse_attribution(text: &str) -> (String, Option<String>) {
    for pattern in ATTRIBUTION_PATTERNS.iter() {
        if let Some(caps) = pattern.captures(text) {
            let day = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let month = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let year = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            let hour = caps.get(4).map(|m| m.as_str()).unwrap_or("");
            let minute = caps.get(5).map(|m| m.as_str()).unwrap_or("");
            let sender_raw = caps.get(6).map(|m| m.as_str()).unwrap_or("");

            let timestamp = parse_timestamp(day, month, year, hour, minute);
            let sender = parse_sender(sender_raw);

            return (sender, timestamp);
        }
    }

    (parse_sender(text.trim().trim_end_matches(':')), None)
}

static BLOCKQUOTE_SEL: Lazy<Selector> = Lazy::new(|| Selector::parse("blockquote").unwrap());
static GMAIL_QUOTE_SEL: Lazy<Selector> =
    Lazy::new(|| Selector::parse("div.gmail_quote, div.gmail_quote_container").unwrap());
static SIGNATURE_SEL: Lazy<Selector> =
    Lazy::new(|| Selector::parse("div.gmail_signature").unwrap());

/// Extract the inner HTML of an element, excluding nested blockquotes,
/// gmail_quote containers, and gmail_signature divs.
fn extract_direct_html(element: &scraper::ElementRef) -> String {
    // Collect node IDs to exclude
    let mut exclude_ids = std::collections::HashSet::new();
    for el in element.select(&BLOCKQUOTE_SEL) {
        exclude_ids.insert(el.id());
    }
    for el in element.select(&GMAIL_QUOTE_SEL) {
        exclude_ids.insert(el.id());
    }
    for el in element.select(&SIGNATURE_SEL) {
        exclude_ids.insert(el.id());
    }

    let mut html = String::new();
    collect_html_excluding(*element, &exclude_ids, &mut html);
    html
}

fn collect_html_excluding(
    node_ref: scraper::ElementRef,
    exclude_ids: &std::collections::HashSet<ego_tree::NodeId>,
    out: &mut String,
) {
    // Iterate over direct children of this element
    for child in node_ref.children() {
        write_node_html(child, exclude_ids, out);
    }
}

fn write_node_html(
    node: ego_tree::NodeRef<Node>,
    exclude_ids: &std::collections::HashSet<ego_tree::NodeId>,
    out: &mut String,
) {
    match node.value() {
        Node::Text(text) => out.push_str(text),
        Node::Element(el) => {
            // Check if this element should be excluded
            if let Some(element_ref) = scraper::ElementRef::wrap(node) {
                if exclude_ids.contains(&element_ref.id()) {
                    return;
                }
            }
            out.push('<');
            out.push_str(&el.name.local);
            for (key, val) in el.attrs() {
                out.push(' ');
                out.push_str(key);
                out.push_str("=\"");
                out.push_str(val);
                out.push('"');
            }
            out.push('>');
            for child in node.children() {
                write_node_html(child, exclude_ids, out);
            }
            out.push_str("</");
            out.push_str(&el.name.local);
            out.push('>');
        }
        _ => {
            for child in node.children() {
                write_node_html(child, exclude_ids, out);
            }
        }
    }
}

/// Extract thread messages from HTML blockquotes.
///
/// Walks the `<blockquote>` nesting, extracting each quoted message's
/// content and attribution. Returns messages in chronological order (oldest first).
pub fn extract_thread_messages(html: &str) -> Vec<ThreadMessage> {
    let document = Html::parse_document(html);

    let mut messages: Vec<ThreadMessage> = Vec::new();
    let mut seen_bodies: std::collections::HashSet<String> = std::collections::HashSet::new();

    for bq in document.select(&BLOCKQUOTE_SEL) {
        let attribution_text = find_attribution_for_blockquote(&document, &bq);

        let (sender, timestamp) = if let Some(attr_text) = &attribution_text {
            parse_attribution(attr_text)
        } else {
            continue;
        };

        let direct_html = extract_direct_html(&bq);
        if direct_html.trim().is_empty() {
            continue;
        }

        let md = html_to_markdown(&direct_html);
        let (body, _sig) = signature::extract_signature(&md);
        let body = collapse_empty_lines(&trim_whitespace_lines(body.trim()));
        let body = body.trim().to_string();

        if body.is_empty() {
            continue;
        }

        // Deduplication
        if !seen_bodies.insert(body.clone()) {
            continue;
        }

        messages.push(ThreadMessage {
            sender,
            timestamp,
            body,
        });
    }

    // Reverse: outermost blockquote = most recent, innermost = oldest
    messages.reverse();
    messages
}

/// Find the attribution text for a blockquote by looking at preceding siblings.
fn find_attribution_for_blockquote(
    document: &Html,
    blockquote: &scraper::ElementRef,
) -> Option<String> {
    let tree_node = document.tree.get(blockquote.id())?;

    // Walk previous siblings looking for a gmail_attr div
    let mut sibling = tree_node.prev_sibling();
    while let Some(sib) = sibling {
        if let Node::Element(el) = sib.value() {
            let classes: Vec<&str> = el.classes().collect();
            if classes.contains(&"gmail_attr") {
                return Some(collect_text_content(sib));
            }
        }
        sibling = sib.prev_sibling();
    }

    // Check parent gmail_quote container for gmail_attr child
    let parent = tree_node.parent()?;
    if let Node::Element(parent_el) = parent.value() {
        let classes: Vec<&str> = parent_el.classes().collect();
        if classes.contains(&"gmail_quote") || classes.contains(&"gmail_quote_container") {
            for child in parent.children() {
                if let Node::Element(el) = child.value() {
                    let child_classes: Vec<&str> = el.classes().collect();
                    if child_classes.contains(&"gmail_attr") {
                        return Some(collect_text_content(child));
                    }
                }
            }
        }
    }

    None
}

fn collect_text_content(node: ego_tree::NodeRef<Node>) -> String {
    let mut text = String::new();
    for desc in node.descendants() {
        if let Node::Text(t) = desc.value() {
            text.push_str(t);
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_german_attribution() {
        let text =
            "Am Fr., 27. Feb. 2026 um 11:53\u{a0}Uhr schrieb Max Mustermann <test@example.com>:";
        let (sender, ts) = parse_attribution(text);
        assert_eq!(sender, "Max Mustermann <test@example.com>");
        assert_eq!(ts, Some("2026-02-27T11:53:00".to_string()));
    }

    #[test]
    fn test_parse_english_attribution() {
        let text = "On Fri, 27 Feb 2026 at 09:16, Max Mustermann <test@example.com> wrote:";
        let (sender, ts) = parse_attribution(text);
        assert_eq!(sender, "Max Mustermann <test@example.com>");
        assert_eq!(ts, Some("2026-02-27T09:16:00".to_string()));
    }

    #[test]
    fn test_month_to_number() {
        assert_eq!(month_to_number("Feb"), Some(2));
        assert_eq!(month_to_number("März"), Some(3));
        assert_eq!(month_to_number("Dez"), Some(12));
    }
}
