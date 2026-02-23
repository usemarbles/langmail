use serde::{Deserialize, Serialize};
use std::fmt;

/// The primary output of langmail preprocessing.
///
/// Contains a cleaned email body optimized for LLM consumption,
/// along with structured metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessedEmail {
    /// The cleaned email body text, with quotes and signature removed.
    pub body: String,

    /// Email subject line.
    pub subject: Option<String>,

    /// Sender address.
    pub from: Option<Address>,

    /// Recipient addresses.
    pub to: Vec<Address>,

    /// CC addresses.
    pub cc: Vec<Address>,

    /// Date as ISO 8601 string.
    pub date: Option<String>,

    /// RFC 2822 Message-ID header value.
    pub rfc_message_id: Option<String>,

    /// In-Reply-To header values (for threading).
    pub in_reply_to: Option<Vec<String>>,

    /// References header values (for threading).
    pub references: Option<Vec<String>>,

    /// Extracted signature, if found.
    pub signature: Option<String>,

    /// Length of the original body before cleaning.
    pub raw_body_length: usize,

    /// Length of the cleaned body.
    pub clean_body_length: usize,
}

/// An email address with optional display name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    /// Display name (e.g. "Alice").
    pub name: Option<String>,

    /// Email address (e.g. "alice@example.com").
    pub email: String,
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{} <{}>", name, self.email),
            None => write!(f, "{}", self.email),
        }
    }
}

impl ProcessedEmail {
    /// Format the email as a plain-text string suitable for LLM prompts.
    ///
    /// Produces a deterministic representation with header lines (FROM, TO,
    /// SUBJECT, DATE) followed by a CONTENT section.  Missing fields are
    /// omitted; the CONTENT line is always present.
    pub fn to_llm_context(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        if let Some(from) = &self.from {
            parts.push(format!("FROM: {}", from));
        }
        if !self.to.is_empty() {
            let to_str: Vec<String> = self.to.iter().map(|a| a.to_string()).collect();
            parts.push(format!("TO: {}", to_str.join(", ")));
        }
        if let Some(subject) = &self.subject {
            parts.push(format!("SUBJECT: {}", subject));
        }
        if let Some(date) = &self.date {
            parts.push(format!("DATE: {}", date));
        }
        parts.push("CONTENT:".to_string());
        parts.push(self.body.clone());

        parts.join("\n")
    }
}

/// Options for customizing preprocessing behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreprocessOptions {
    /// Whether to strip quoted reply content. Default: true.
    #[serde(default = "default_true")]
    pub strip_quotes: bool,

    /// Whether to strip email signatures. Default: true.
    #[serde(default = "default_true")]
    pub strip_signature: bool,

    /// Maximum body length in characters. 0 = no limit. Default: 0.
    #[serde(default)]
    pub max_body_length: usize,
}

impl Default for PreprocessOptions {
    fn default() -> Self {
        Self {
            strip_quotes: true,
            strip_signature: true,
            max_body_length: 0,
        }
    }
}

fn default_true() -> bool {
    true
}

/// Errors that can occur during preprocessing.
#[derive(Debug)]
pub enum LangmailError {
    /// The raw input could not be parsed as a valid email message.
    ParseFailed,
}

impl fmt::Display for LangmailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LangmailError::ParseFailed => write!(f, "Failed to parse email message"),
        }
    }
}

impl std::error::Error for LangmailError {}
