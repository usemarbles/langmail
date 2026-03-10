use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Preprocessed email output optimized for LLM consumption.
#[napi(object)]
pub struct ProcessedEmail {
    /// The cleaned email body text, with quotes and signature removed.
    pub body: String,

    /// Email subject line.
    pub subject: Option<String>,

    /// Sender address.
    pub from: Option<NapiAddress>,

    /// Recipient addresses.
    pub to: Vec<NapiAddress>,

    /// CC addresses.
    pub cc: Vec<NapiAddress>,

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
    pub raw_body_length: u32,

    /// Length of the cleaned body.
    pub clean_body_length: u32,

    /// Primary call-to-action link extracted from the HTML body, if any.
    pub primary_cta: Option<NapiCallToAction>,

    /// Thread messages extracted from quoted reply blocks (oldest first).
    pub thread_messages: Vec<NapiThreadMessage>,
}

/// A single message extracted from a quoted reply chain.
#[napi(object)]
pub struct NapiThreadMessage {
    /// The sender attribution (e.g. "Max Mustermann <test@example.com>").
    pub sender: String,
    /// ISO 8601 timestamp, if parseable from the attribution.
    pub timestamp: Option<String>,
    /// The message body (cleaned, no nested quotes).
    pub body: String,
}

/// Controls how `toLlmContextWithOptions` renders the email body.
#[napi(string_enum)]
pub enum NapiRenderMode {
    /// Strip all quoted content — only the latest message is rendered.
    LatestOnly,
    /// Render quoted replies as a chronological transcript below the main content.
    ThreadHistory,
}

/// Options for `toLlmContextWithOptions`.
#[napi(object)]
pub struct NapiLlmContextOptions {
    /// How to render the email body. Default: "LatestOnly".
    pub render_mode: Option<NapiRenderMode>,
}

/// A primary call-to-action link extracted from an HTML email.
#[napi(object)]
pub struct NapiCallToAction {
    /// The URL the action points to.
    pub url: String,
    /// Human-readable label for the action.
    pub text: String,
    /// Confidence score between 0.0 and 1.0.
    pub confidence: f64,
}

/// An email address with optional display name.
#[napi(object)]
pub struct NapiAddress {
    /// Display name (e.g. "Alice").
    pub name: Option<String>,

    /// Email address (e.g. "alice@example.com").
    pub email: String,
}

/// Options for customizing preprocessing behavior.
#[napi(object)]
pub struct PreprocessOptions {
    /// Whether to strip quoted reply content. Default: true.
    pub strip_quotes: Option<bool>,

    /// Whether to strip email signatures. Default: true.
    pub strip_signature: Option<bool>,

    /// Maximum body length in characters. 0 = no limit. Default: 0.
    pub max_body_length: Option<u32>,
}

/// Preprocess a raw email into an LLM-ready structure.
///
/// Accepts raw email bytes (RFC 5322 / EML format) and returns a structured
/// object with clean body text and metadata.
///
/// @param raw - Raw email as a Buffer or Uint8Array
/// @returns Preprocessed email output
#[napi]
pub fn preprocess(raw: Buffer) -> Result<ProcessedEmail> {
    let result = langmail::preprocess(&raw)
        .map_err(|e| Error::new(Status::GenericFailure, format!("{}", e)))?;

    Ok(to_napi_output(result))
}

/// Preprocess a raw email with custom options.
///
/// @param raw - Raw email as a Buffer or Uint8Array
/// @param options - Preprocessing options
/// @returns Preprocessed email output
#[napi]
pub fn preprocess_with_options(raw: Buffer, options: PreprocessOptions) -> Result<ProcessedEmail> {
    let core_options = langmail::PreprocessOptions {
        strip_quotes: options.strip_quotes.unwrap_or(true),
        strip_signature: options.strip_signature.unwrap_or(true),
        max_body_length: options.max_body_length.unwrap_or(0) as usize,
    };

    let result = langmail::preprocess_with_options(&raw, &core_options)
        .map_err(|e| Error::new(Status::GenericFailure, format!("{}", e)))?;

    Ok(to_napi_output(result))
}

/// Preprocess a raw email string (convenience wrapper).
///
/// Same as `preprocess` but accepts a string instead of a Buffer.
///
/// @param raw - Raw email as a string
/// @returns Preprocessed email output
#[napi]
pub fn preprocess_string(raw: String) -> Result<ProcessedEmail> {
    preprocess(Buffer::from(raw.as_bytes().to_vec()))
}

/// Format a preprocessed email as an LLM-ready context string.
///
/// Takes a `ProcessedEmail` (as returned by `preprocess`) and returns a
/// deterministic plain-text representation with header lines followed by a
/// CONTENT section, suitable for pasting into an LLM prompt.
///
/// @param email - A ProcessedEmail object
/// @returns Formatted context string
#[napi]
pub fn to_llm_context(email: ProcessedEmail) -> String {
    let core_email = to_core_email(email);
    core_email.to_llm_context()
}

/// Format a preprocessed email as an LLM-ready context string with options.
///
/// Same as `toLlmContext` but accepts options to control rendering, e.g.
/// `{ renderMode: "ThreadHistory" }` to include quoted reply history.
///
/// @param email - A ProcessedEmail object
/// @param options - LLM context options
/// @returns Formatted context string
#[napi]
pub fn to_llm_context_with_options(
    email: ProcessedEmail,
    options: NapiLlmContextOptions,
) -> String {
    let core_email = to_core_email(email);
    let core_options = langmail::LlmContextOptions {
        render_mode: match options.render_mode {
            Some(NapiRenderMode::ThreadHistory) => langmail::RenderMode::ThreadHistory,
            _ => langmail::RenderMode::LatestOnly,
        },
    };
    core_email.to_llm_context_with_options(&core_options)
}

// ---------------------------------------------------------------------------
// Internal conversion
// ---------------------------------------------------------------------------

fn to_core_email(email: ProcessedEmail) -> langmail::ProcessedEmail {
    langmail::ProcessedEmail {
        body: email.body,
        subject: email.subject,
        from: email.from.map(|a| langmail::Address {
            name: a.name,
            email: a.email,
        }),
        to: email
            .to
            .into_iter()
            .map(|a| langmail::Address {
                name: a.name,
                email: a.email,
            })
            .collect(),
        cc: email
            .cc
            .into_iter()
            .map(|a| langmail::Address {
                name: a.name,
                email: a.email,
            })
            .collect(),
        date: email.date,
        rfc_message_id: email.rfc_message_id,
        in_reply_to: email.in_reply_to,
        references: email.references,
        signature: email.signature,
        raw_body_length: email.raw_body_length as usize,
        clean_body_length: email.clean_body_length as usize,
        primary_cta: email.primary_cta.map(|c| langmail::CallToAction {
            url: c.url,
            text: c.text,
            confidence: c.confidence,
        }),
        thread_messages: email
            .thread_messages
            .into_iter()
            .map(|m| langmail::ThreadMessage {
                sender: m.sender,
                timestamp: m.timestamp,
                body: m.body,
            })
            .collect(),
    }
}

fn to_napi_output(result: langmail::ProcessedEmail) -> ProcessedEmail {
    ProcessedEmail {
        body: result.body,
        subject: result.subject,
        from: result.from.map(|a| NapiAddress {
            name: a.name,
            email: a.email,
        }),
        to: result
            .to
            .into_iter()
            .map(|a| NapiAddress {
                name: a.name,
                email: a.email,
            })
            .collect(),
        cc: result
            .cc
            .into_iter()
            .map(|a| NapiAddress {
                name: a.name,
                email: a.email,
            })
            .collect(),
        date: result.date,
        rfc_message_id: result.rfc_message_id,
        in_reply_to: result.in_reply_to,
        references: result.references,
        signature: result.signature,
        raw_body_length: result.raw_body_length as u32,
        clean_body_length: result.clean_body_length as u32,
        primary_cta: result.primary_cta.map(|c| NapiCallToAction {
            url: c.url,
            text: c.text,
            confidence: c.confidence,
        }),
        thread_messages: result
            .thread_messages
            .into_iter()
            .map(|m| NapiThreadMessage {
                sender: m.sender,
                timestamp: m.timestamp,
                body: m.body,
            })
            .collect(),
    }
}
