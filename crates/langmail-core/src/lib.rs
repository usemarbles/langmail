mod html;
mod quotes;
mod signature;
mod types;

pub use types::*;

use mail_parser::MessageParser;

/// Preprocess a raw email (RFC 5322 / EML format) into an LLM-ready structure.
///
/// This is the primary entry point for langmail. It takes raw email bytes and
/// returns a structured [`ProcessedEmail`] with clean body text, metadata, and
/// thread information — optimized for feeding into language models.
///
/// # Example
/// ```
/// let raw = b"From: alice@example.com\r\nTo: bob@example.com\r\nSubject: Hello\r\n\r\nHi Bob!";
/// let output = langmail_core::preprocess(raw).unwrap();
/// assert!(output.body.contains("Hi Bob!"));
/// ```
pub fn preprocess(raw: &[u8]) -> Result<ProcessedEmail, LangmailError> {
    let message = MessageParser::default()
        .parse(raw)
        .ok_or(LangmailError::ParseFailed)?;

    let subject = message.subject().map(|s| s.to_string());

    let from = message
        .from()
        .and_then(|addrs| addrs.first())
        .map(|addr| Address {
            name: addr.name().map(|n| n.to_string()),
            email: addr.address().map(|a| a.to_string()).unwrap_or_default(),
        });

    let to = extract_addresses(message.to());
    let cc = extract_addresses(message.cc());

    let date = message.date().map(|d| {
        // mail-parser DateTime -> ISO 8601 string
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            d.year, d.month, d.day, d.hour, d.minute, d.second
        )
    });

    let message_id = message.message_id().map(|id| id.to_string());

    let in_reply_to = message
        .in_reply_to()
        .as_text_list()
        .map(|list| list.iter().map(|s| s.to_string()).collect());

    let references = message
        .references()
        .as_text_list()
        .map(|list| list.iter().map(|s| s.to_string()).collect());

    // Extract body: prefer plain text, fall back to HTML with conversion
    let raw_body = extract_body(&message);

    // Strip quoted replies
    let body_without_quotes = quotes::strip_quotes(&raw_body);

    // Strip signature
    let (clean_body, signature) = signature::extract_signature(&body_without_quotes);

    Ok(ProcessedEmail {
        body: clean_body.trim().to_string(),
        subject,
        from,
        to,
        cc,
        date,
        message_id,
        in_reply_to,
        references,
        signature,
        raw_body_length: raw_body.len(),
        clean_body_length: clean_body.trim().len(),
    })
}

/// Preprocess with custom options.
pub fn preprocess_with_options(
    raw: &[u8],
    options: &PreprocessOptions,
) -> Result<ProcessedEmail, LangmailError> {
    let mut output = preprocess(raw)?;

    if !options.strip_quotes {
        // Re-extract without quote stripping
        let message = MessageParser::default()
            .parse(raw)
            .ok_or(LangmailError::ParseFailed)?;
        let raw_body = extract_body(&message);
        let (clean_body, sig) = if options.strip_signature {
            signature::extract_signature(&raw_body)
        } else {
            (raw_body.clone(), None)
        };
        output.body = clean_body.trim().to_string();
        output.signature = sig;
        output.clean_body_length = output.body.len();
    } else if !options.strip_signature {
        // Re-run with quotes stripped but keep signature
        let message = MessageParser::default()
            .parse(raw)
            .ok_or(LangmailError::ParseFailed)?;
        let raw_body = extract_body(&message);
        let body_without_quotes = quotes::strip_quotes(&raw_body);
        output.body = body_without_quotes.trim().to_string();
        output.signature = None;
        output.clean_body_length = output.body.len();
    }

    if options.max_body_length > 0 && output.body.len() > options.max_body_length {
        output.body = output.body[..options.max_body_length].to_string();
        output.clean_body_length = output.body.len();
    }

    Ok(output)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn extract_body(message: &mail_parser::Message) -> String {
    // Prefer plain text body
    if let Some(text) = message.body_text(0) {
        return text.to_string();
    }

    // Fall back to HTML body, converting to plain text
    if let Some(html_body) = message.body_html(0) {
        return html::html_to_clean_text(&html_body);
    }

    String::new()
}

fn extract_addresses(address_opt: Option<&mail_parser::Address>) -> Vec<Address> {
    match address_opt {
        Some(addresses) => addresses
            .iter()
            .map(|addr| Address {
                name: addr.name().map(|n| n.to_string()),
                email: addr.address().map(|a| a.to_string()).unwrap_or_default(),
            })
            .collect(),
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_email() -> Vec<u8> {
        concat!(
            "From: Alice <alice@example.com>\r\n",
            "To: Bob <bob@example.com>\r\n",
            "Subject: Hello Bob\r\n",
            "Date: Thu, 05 Feb 2026 10:00:00 +0000\r\n",
            "Message-ID: <abc123@example.com>\r\n",
            "Content-Type: text/plain; charset=utf-8\r\n",
            "\r\n",
            "Hey Bob,\r\n",
            "\r\n",
            "Just wanted to say hi!\r\n",
            "\r\n",
            "Best,\r\n",
            "Alice\r\n",
        )
        .as_bytes()
        .to_vec()
    }

    fn reply_email() -> Vec<u8> {
        concat!(
            "From: Bob <bob@example.com>\r\n",
            "To: Alice <alice@example.com>\r\n",
            "Subject: Re: Hello Bob\r\n",
            "Date: Thu, 05 Feb 2026 11:00:00 +0000\r\n",
            "Message-ID: <def456@example.com>\r\n",
            "In-Reply-To: <abc123@example.com>\r\n",
            "References: <abc123@example.com>\r\n",
            "Content-Type: text/plain; charset=utf-8\r\n",
            "\r\n",
            "Hi Alice!\r\n",
            "\r\n",
            "Great to hear from you.\r\n",
            "\r\n",
            "On Thu, 05 Feb 2026 at 10:00, Alice <alice@example.com> wrote:\r\n",
            "> Hey Bob,\r\n",
            ">\r\n",
            "> Just wanted to say hi!\r\n",
            ">\r\n",
            "> Best,\r\n",
            "> Alice\r\n",
        )
        .as_bytes()
        .to_vec()
    }

    #[test]
    fn test_simple_email() {
        let output = preprocess(&simple_email()).unwrap();
        assert_eq!(output.subject.as_deref(), Some("Hello Bob"));
        assert_eq!(output.from.as_ref().unwrap().email, "alice@example.com");
        assert_eq!(output.from.as_ref().unwrap().name.as_deref(), Some("Alice"));
        assert_eq!(output.to.len(), 1);
        assert_eq!(output.to[0].email, "bob@example.com");
        assert!(output.body.contains("Just wanted to say hi!"));
    }

    #[test]
    fn test_reply_strips_quotes() {
        let output = preprocess(&reply_email()).unwrap();
        assert!(output.body.contains("Great to hear from you."));
        assert!(!output.body.contains("Just wanted to say hi!"));
        assert_eq!(
            output.in_reply_to.as_ref().unwrap(),
            &["abc123@example.com"]
        );
    }

    #[test]
    fn test_clean_body_shorter_than_raw() {
        let output = preprocess(&reply_email()).unwrap();
        assert!(output.clean_body_length < output.raw_body_length);
    }
}
