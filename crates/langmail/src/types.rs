use serde::{Deserialize, Serialize};
use std::fmt;

/// Controls how `to_llm_context_with_options` renders the email body.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderMode {
    /// Strip all quoted content — only the latest message is rendered.
    #[default]
    LatestOnly,
    /// Render quoted replies as a chronological transcript below the main content.
    ThreadHistory,
}

/// Options for `to_llm_context_with_options`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmContextOptions {
    /// How to render the email body. Default: `LatestOnly`.
    #[serde(default)]
    pub render_mode: RenderMode,
}

/// A single message extracted from a quoted reply chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThreadMessage {
    /// The sender attribution line (e.g. "Max Mustermann <test@example.com>").
    pub sender: String,
    /// ISO 8601 timestamp, if parseable from the attribution.
    pub timestamp: Option<String>,
    /// The message body (cleaned, markdown-converted, no nested quotes).
    pub body: String,
}

/// A primary call-to-action link extracted from an HTML email.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallToAction {
    /// The URL the action points to.
    pub url: String,
    /// Human-readable label for the action.
    pub text: String,
    /// Confidence score between 0.0 and 1.0.
    pub confidence: f64,
}

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

    /// Primary call-to-action link extracted from the HTML body, if any.
    pub primary_cta: Option<CallToAction>,

    /// Thread messages extracted from quoted reply blocks in the HTML.
    /// Present when the email contains `<blockquote>` quoted replies.
    /// Ordered oldest-first (chronological).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub thread_messages: Vec<ThreadMessage>,
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
        self.to_llm_context_with_options(&LlmContextOptions::default())
    }

    /// Format the email as a plain-text string with rendering options.
    ///
    /// When `options.render_mode` is [`RenderMode::ThreadHistory`], appends a
    /// chronological transcript of quoted reply messages after a `---` separator.
    pub fn to_llm_context_with_options(&self, options: &LlmContextOptions) -> String {
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

        if matches!(options.render_mode, RenderMode::ThreadHistory)
            && !self.thread_messages.is_empty()
        {
            parts.push("\n---\n".to_string());
            parts.push("THREAD HISTORY:\n".to_string());

            for (i, msg) in self.thread_messages.iter().enumerate() {
                if i > 0 {
                    parts.push(String::new()); // blank line between messages
                }
                let header = if let Some(ts) = &msg.timestamp {
                    format!("[{}] {}", ts, msg.sender)
                } else {
                    format!("[?] {}", msg.sender)
                };
                parts.push(header);
                parts.push(msg.body.clone());
            }
        }

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
