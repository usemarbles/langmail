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

    /// Message-ID header value.
    pub message_id: Option<String>,

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
    let result = langmail_core::preprocess(&raw)
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
    let core_options = langmail_core::PreprocessOptions {
        strip_quotes: options.strip_quotes.unwrap_or(true),
        strip_signature: options.strip_signature.unwrap_or(true),
        max_body_length: options.max_body_length.unwrap_or(0) as usize,
    };

    let result = langmail_core::preprocess_with_options(&raw, &core_options)
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

// ---------------------------------------------------------------------------
// Internal conversion
// ---------------------------------------------------------------------------

fn to_napi_output(result: langmail_core::ProcessedEmail) -> ProcessedEmail {
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
        message_id: result.message_id,
        in_reply_to: result.in_reply_to,
        references: result.references,
        signature: result.signature,
        raw_body_length: result.raw_body_length as u32,
        clean_body_length: result.clean_body_length as u32,
    }
}
