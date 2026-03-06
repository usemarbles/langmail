use once_cell::sync::Lazy;
use regex::Regex;
use scraper::{Element, ElementRef, Html, Selector};

use crate::types::CallToAction;

// ---------------------------------------------------------------------------
// Static selectors and regexes
// ---------------------------------------------------------------------------

static SEL_JSON_LD: Lazy<Selector> =
    Lazy::new(|| Selector::parse(r#"script[type="application/ld+json"]"#).unwrap());

static SEL_LINKS: Lazy<Selector> = Lazy::new(|| Selector::parse("a[href]").unwrap());

/// Blacklisted patterns in text or href – unsubscribe, legal, privacy.
static RE_BLACKLIST: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)unsubscribe|datenschutz|impressum|privacy|terms|abmelden").unwrap()
});

/// Bare homepage: https://example.com or https://www.example.com/ (no real path).
static RE_BARE_HOMEPAGE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)^https?://(www\.)?[^/]+/?$").unwrap());

/// Class names that indicate a button.
static RE_BTN_CLASS: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\b(btn|button|cta)\b").unwrap());

/// Action keywords (DE multi-word first, then EN/DE single words with \b).
static RE_ACTION_KW: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)geht\s+es\s+zum|hier\s+klicken|mehr\s+erfahren|\b(view|read|open|see|check|reply|continue|go|access|confirm|download|get|start|register|subscribe|buy|order|shop|learn|discover|explore|join)\b",
    )
    .unwrap()
});

/// Action verbs for aria-label scoring.
static RE_ARIA_ACTION: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(view|read|open|see|check|reply|continue|go|access|confirm)\b").unwrap()
});

/// Checks `text-align: center` in a style attribute.
static RE_TEXT_ALIGN_CENTER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"text-align\s*:\s*center").unwrap());

/// Checks `padding: <2+ digit number>` (shorthand only) in a style attribute.
static RE_PADDING_BIG: Lazy<Regex> = Lazy::new(|| Regex::new(r"padding\s*:\s*\d{2,}").unwrap());

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Extract the primary call-to-action link from a decoded HTML email body.
///
/// Tries JSON-LD `potentialAction` first (confidence 1.0), then falls back
/// to heuristic link scoring.  Returns `None` if no suitable link is found.
pub fn extract_cta(html: &str) -> Option<CallToAction> {
    let doc = Html::parse_document(html);

    // Fast path: structured JSON-LD markup
    if let Some(cta) = extract_json_ld(&doc) {
        return Some(cta);
    }

    // Heuristic path: score every <a href="…"> element
    extract_heuristic(&doc)
}

// ---------------------------------------------------------------------------
// JSON-LD path
// ---------------------------------------------------------------------------

fn extract_json_ld(doc: &Html) -> Option<CallToAction> {
    for script in doc.select(&SEL_JSON_LD) {
        let text: String = script.text().collect();
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(target) = json
                .get("potentialAction")
                .and_then(|a| a.get("target"))
                .and_then(|t| t.as_str())
            {
                let name = json
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("View")
                    .to_string();
                return Some(CallToAction {
                    url: target.to_string(),
                    text: name,
                    confidence: 1.0,
                });
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Heuristic path
// ---------------------------------------------------------------------------

fn extract_heuristic(doc: &Html) -> Option<CallToAction> {
    let mut best: Option<(i32, CallToAction)> = None;

    for link in doc.select(&SEL_LINKS) {
        let href = match link.value().attr("href") {
            Some(h) => h,
            None => continue,
        };

        // Must be http/https
        if !href.starts_with("http://") && !href.starts_with("https://") {
            continue;
        }

        // Link text: prefer title attribute, fall back to scraped text
        let text = link
            .value()
            .attr("title")
            .map(|t| t.trim().to_string())
            .unwrap_or_else(|| link.text().collect::<String>().trim().to_string());

        // Skip empty text
        if text.is_empty() {
            continue;
        }

        // Blacklist: unsubscribe, privacy, etc.
        if RE_BLACKLIST.is_match(&text) || RE_BLACKLIST.is_match(href) {
            continue;
        }

        // Bare homepage filter
        if RE_BARE_HOMEPAGE.is_match(href) {
            continue;
        }

        // Logo/brand image wrapper
        if wraps_logo_img(&link) {
            continue;
        }

        // Structural exclusion: header / footer / banner / contentinfo
        if in_excluded_ancestor(&link) {
            continue;
        }

        let score = score_link(&link, &text);
        if score < 40 {
            continue;
        }

        if best.as_ref().is_none_or(|(s, _)| score > *s) {
            best = Some((
                score,
                CallToAction {
                    url: href.to_string(),
                    text,
                    confidence: f64::min(score as f64 / 100.0, 1.0),
                },
            ));
        }
    }

    best.map(|(_, cta)| cta)
}

// ---------------------------------------------------------------------------
// Scoring helpers
// ---------------------------------------------------------------------------

fn score_link(link: &ElementRef, text: &str) -> i32 {
    let mut score = 0i32;

    // +30: button-styled element
    if is_button_styled(link) {
        score += 30;
    }

    // +25: action keyword present; +40 bonus when text is also long
    if RE_ACTION_KW.is_match(text) {
        score += 25;
        if text.len() >= 20 {
            score += 40;
        }
    }

    // +20: visually prominent (≥2 of 3 signals)
    if prominent_count(link, text) >= 2 {
        score += 20;
    }

    // +15: aria-label contains an action verb
    if let Some(aria) = link.value().attr("aria-label") {
        if RE_ARIA_ACTION.is_match(aria) {
            score += 15;
        }
    }

    score
}

/// Returns true if the link looks like a button via class, role, or parent TD styling.
fn is_button_styled(link: &ElementRef) -> bool {
    // Own class contains btn/button/cta
    if let Some(cls) = link.value().attr("class") {
        if RE_BTN_CLASS.is_match(cls) {
            return true;
        }
    }
    // role="button"
    if link.value().attr("role") == Some("button") {
        return true;
    }
    // Parent <td> style has padding AND (background | border)
    if let Some(parent) = link.parent_element() {
        if parent.value().name() == "td" {
            if let Some(style) = parent.value().attr("style") {
                let has_padding = style.contains("padding");
                let has_bg_or_border = style.contains("background") || style.contains("border");
                if has_padding && has_bg_or_border {
                    return true;
                }
            }
        }
    }
    false
}

/// Counts how many of the three visual-prominence signals are present.
fn prominent_count(link: &ElementRef, text: &str) -> usize {
    let mut count = 0;
    if let Some(parent) = link.parent_element() {
        if let Some(style) = parent.value().attr("style") {
            if RE_TEXT_ALIGN_CENTER.is_match(style) {
                count += 1;
            }
            if RE_PADDING_BIG.is_match(style) {
                count += 1;
            }
        }
    }
    if text.len() > 10 {
        count += 1;
    }
    count
}

/// Returns true if the link wraps an `<img>` whose alt text contains "logo" or "brand".
fn wraps_logo_img(link: &ElementRef) -> bool {
    static SEL_IMG: Lazy<Selector> = Lazy::new(|| Selector::parse("img").unwrap());
    for img in link.select(&SEL_IMG) {
        if let Some(alt) = img.value().attr("alt") {
            let lower = alt.to_lowercase();
            if lower.contains("logo") || lower.contains("brand") {
                return true;
            }
        }
    }
    false
}

/// Returns true if any ancestor is a `<header>`, `<footer>`, or has
/// `role="banner"` / `role="contentinfo"`.
fn in_excluded_ancestor(link: &ElementRef) -> bool {
    let mut current = link.parent_element();
    while let Some(el) = current {
        let name = el.value().name();
        let role = el.value().attr("role").unwrap_or("");
        if name == "header" || name == "footer" || role == "banner" || role == "contentinfo" {
            return true;
        }
        current = el.parent_element();
    }
    false
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn wrap(link: &str) -> String {
        format!("<html><body>{}</body></html>", link)
    }

    // --- Filter tests ---

    #[test]
    fn test_bare_homepage_filtered() {
        let html = wrap(r#"<a href="https://example.com/">View your account now today</a>"#);
        assert!(extract_cta(&html).is_none());
    }

    #[test]
    fn test_bare_homepage_with_path_not_filtered() {
        // A path after the host is NOT a bare homepage — should pass the filter
        let html = wrap(
            r#"<a href="https://example.com/dashboard" class="btn">Go to your dashboard now</a>"#,
        );
        let cta = extract_cta(&html);
        assert!(
            cta.is_some(),
            "link with path should not be bare-homepage-filtered"
        );
    }

    #[test]
    fn test_unsubscribe_in_text_filtered() {
        let html = wrap(r#"<a href="https://example.com/manage">Unsubscribe from this list</a>"#);
        assert!(extract_cta(&html).is_none());
    }

    #[test]
    fn test_unsubscribe_in_href_filtered() {
        let html = wrap(r#"<a href="https://example.com/unsubscribe">Manage preferences</a>"#);
        assert!(extract_cta(&html).is_none());
    }

    #[test]
    fn test_mailto_filtered() {
        let html = wrap(r#"<a href="mailto:support@example.com">Contact us</a>"#);
        assert!(extract_cta(&html).is_none());
    }

    #[test]
    fn test_low_score_discarded() {
        // "random link" has no action keyword and no button styling → score 0 < 40
        let html = wrap(r#"<a href="https://example.com/page">Random link text here</a>"#);
        assert!(extract_cta(&html).is_none());
    }

    #[test]
    fn test_logo_image_filtered() {
        let html = wrap(
            r#"<a href="https://www.company.com/home"><img src="logo.png" alt="Company Logo" /></a>"#,
        );
        assert!(extract_cta(&html).is_none());
    }

    #[test]
    fn test_brand_image_filtered() {
        let html = wrap(
            r#"<a href="https://example.com/brand"><img src="img.png" alt="Brand Image" />View offer</a>"#,
        );
        assert!(extract_cta(&html).is_none());
    }

    #[test]
    fn test_empty_alt_image_not_logo_filtered() {
        // No alt text → not treated as logo → link passes logo filter
        // But empty-text links still get filtered
        let html = wrap(r#"<a href="https://example.com/view"><img src="img.png" /></a>"#);
        // Link text is empty → filtered by empty-text check, not logo check
        assert!(extract_cta(&html).is_none());
    }

    // --- Scoring tests ---

    #[test]
    fn test_button_class_detected() {
        // +30 (btn class) + 25 ("go" keyword) = 55 ≥ 40 → found
        let html =
            wrap(r#"<a href="https://app.example.com/dash" class="btn">Go to Dashboard</a>"#);
        let cta = extract_cta(&html).expect("should find CTA with btn class");
        assert_eq!(cta.url, "https://app.example.com/dash");
        assert!(cta.confidence >= 0.5);
    }

    #[test]
    fn test_de_keyword_long_text() {
        // +25 (DE keyword "geht es zum") + 40 (len 29 ≥ 20) = 65 ≥ 40 → found
        let html =
            wrap(r#"<a href="https://example.com/postfach">Hier geht es zum TK-Postfach</a>"#);
        let cta = extract_cta(&html).expect("should find CTA with DE keyword");
        assert!(
            cta.confidence > 0.6,
            "confidence should be > 0.6, got {}",
            cta.confidence
        );
        assert_eq!(cta.url, "https://example.com/postfach");
    }

    #[test]
    fn test_json_ld_fast_path() {
        let html = r#"<html><head>
            <script type="application/ld+json">
            {"@type":"EmailMessage","name":"View Invoice","potentialAction":{"@type":"ViewAction","target":"https://example.com/invoice/42"}}
            </script></head><body></body></html>"#;
        let cta = extract_cta(html).expect("should find JSON-LD CTA");
        assert_eq!(cta.url, "https://example.com/invoice/42");
        assert_eq!(cta.text, "View Invoice");
        assert_eq!(cta.confidence, 1.0);
    }

    #[test]
    fn test_json_ld_missing_target_falls_through() {
        // JSON-LD without potentialAction.target → falls through to heuristic
        let html = r#"<html><head>
            <script type="application/ld+json">{"@type":"EmailMessage","name":"Hello"}</script>
            </head><body>
            <a href="https://example.com/view" class="btn">View your report now here</a>
            </body></html>"#;
        let cta = extract_cta(html).expect("should fall through to heuristic");
        assert!(cta.url.contains("example.com/view"));
    }
}
