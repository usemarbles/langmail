use pyo3::prelude::*;

/// A primary call-to-action link extracted from an HTML email.
#[pyclass(get_all)]
#[derive(Clone)]
pub struct CallToAction {
    /// The URL the action points to.
    pub url: String,
    /// Human-readable label for the action.
    pub text: String,
    /// Confidence score between 0.0 and 1.0.
    pub confidence: f64,
}

#[pymethods]
impl CallToAction {
    fn __repr__(&self) -> String {
        format!(
            "CallToAction(url='{}', text='{}', confidence={:.2})",
            self.url, self.text, self.confidence
        )
    }
}

/// An email address with optional display name.
#[pyclass(get_all)]
#[derive(Clone)]
pub struct Address {
    /// Display name (e.g. "Alice").
    pub name: Option<String>,
    /// Email address (e.g. "alice@example.com").
    pub email: String,
}

#[pymethods]
impl Address {
    fn __repr__(&self) -> String {
        match &self.name {
            Some(name) => format!("Address(name='{}', email='{}')", name, self.email),
            None => format!("Address(email='{}')", self.email),
        }
    }
}

/// A single message extracted from a quoted reply chain.
#[pyclass(get_all)]
#[derive(Clone)]
pub struct ThreadMessage {
    /// The sender attribution (e.g. "Max Mustermann <test@example.com>").
    pub sender: String,
    /// ISO 8601 timestamp, if parseable from the attribution.
    pub timestamp: Option<String>,
    /// The message body (cleaned, no nested quotes).
    pub body: String,
}

#[pymethods]
impl ThreadMessage {
    fn __repr__(&self) -> String {
        format!(
            "ThreadMessage(sender='{}', timestamp={:?}, body=({} chars))",
            self.sender,
            self.timestamp,
            self.body.len()
        )
    }
}

/// Controls how `to_llm_context_with_options` renders the email body.
#[pyclass(eq, eq_int)]
#[derive(Clone, Debug, PartialEq)]
pub enum RenderMode {
    /// Strip all quoted content — only the latest message is rendered.
    LatestOnly = 0,
    /// Render quoted replies as a chronological transcript below the main content.
    ThreadHistory = 1,
}

/// Options for `to_llm_context_with_options`.
#[pyclass(get_all, set_all)]
#[derive(Clone)]
pub struct LlmContextOptions {
    /// How to render the email body. Default: RenderMode.LatestOnly.
    pub render_mode: RenderMode,
}

#[pymethods]
impl LlmContextOptions {
    #[new]
    #[pyo3(signature = (render_mode=RenderMode::LatestOnly))]
    fn new(render_mode: RenderMode) -> Self {
        Self { render_mode }
    }

    fn __repr__(&self) -> String {
        format!("LlmContextOptions(render_mode={:?})", self.render_mode)
    }
}

/// Preprocessed email output optimized for LLM consumption.
#[pyclass(get_all)]
#[derive(Clone)]
pub struct ProcessedEmail {
    /// The cleaned email body text, with quotes and signature removed.
    pub body: String,
    /// Email subject line.
    pub subject: Option<String>,
    /// Sender address.
    pub from_address: Option<Address>,
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
    /// Thread messages extracted from quoted reply blocks (oldest first).
    pub thread_messages: Vec<ThreadMessage>,
}

#[pymethods]
impl ProcessedEmail {
    fn __repr__(&self) -> String {
        format!(
            "ProcessedEmail(subject='{}', body=({} chars))",
            self.subject.as_deref().unwrap_or(""),
            self.clean_body_length
        )
    }
}

/// Options for customizing preprocessing behavior.
#[pyclass(get_all, set_all)]
#[derive(Clone)]
pub struct PreprocessOptions {
    /// Whether to strip quoted reply content. Default: True.
    pub strip_quotes: bool,
    /// Whether to strip email signatures. Default: True.
    pub strip_signature: bool,
    /// Maximum body length in characters. 0 = no limit. Default: 0.
    pub max_body_length: usize,
}

#[pymethods]
impl PreprocessOptions {
    #[new]
    #[pyo3(signature = (strip_quotes=true, strip_signature=true, max_body_length=0))]
    fn new(strip_quotes: bool, strip_signature: bool, max_body_length: usize) -> Self {
        Self {
            strip_quotes,
            strip_signature,
            max_body_length,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "PreprocessOptions(strip_quotes={}, strip_signature={}, max_body_length={})",
            self.strip_quotes, self.strip_signature, self.max_body_length
        )
    }
}

pyo3::create_exception!(langmail, ParseError, pyo3::exceptions::PyValueError);

/// Preprocess a raw email into an LLM-ready structure.
///
/// Accepts raw email bytes (RFC 5322 / EML format) and returns a structured
/// object with clean body text and metadata.
#[pyfunction]
fn preprocess(raw: &[u8]) -> PyResult<ProcessedEmail> {
    let result = ::langmail::preprocess(raw).map_err(|e| ParseError::new_err(e.to_string()))?;
    Ok(to_py_output(result))
}

/// Preprocess a raw email string (convenience wrapper).
///
/// Same as `preprocess` but accepts a string instead of bytes.
#[pyfunction]
fn preprocess_string(raw: &str) -> PyResult<ProcessedEmail> {
    preprocess(raw.as_bytes())
}

/// Preprocess a raw email with custom options.
#[pyfunction]
fn preprocess_with_options(raw: &[u8], options: &PreprocessOptions) -> PyResult<ProcessedEmail> {
    let core_options = ::langmail::PreprocessOptions {
        strip_quotes: options.strip_quotes,
        strip_signature: options.strip_signature,
        max_body_length: options.max_body_length,
    };

    let result = ::langmail::preprocess_with_options(raw, &core_options)
        .map_err(|e| ParseError::new_err(e.to_string()))?;

    Ok(to_py_output(result))
}

/// Format a preprocessed email as an LLM-ready context string.
///
/// Takes a `ProcessedEmail` and returns a deterministic plain-text
/// representation with header lines followed by a CONTENT section.
#[pyfunction]
fn to_llm_context(email: &ProcessedEmail) -> String {
    let core_email = to_core_email(email);
    core_email.to_llm_context()
}

/// Format a preprocessed email as an LLM-ready context string with options.
///
/// Same as `to_llm_context` but accepts options to control rendering, e.g.
/// `LlmContextOptions(render_mode=RenderMode.ThreadHistory)` to include
/// quoted reply history.
#[pyfunction]
fn to_llm_context_with_options(email: &ProcessedEmail, options: &LlmContextOptions) -> String {
    let core_email = to_core_email(email);
    let core_options = ::langmail::LlmContextOptions {
        render_mode: match options.render_mode {
            RenderMode::ThreadHistory => ::langmail::RenderMode::ThreadHistory,
            RenderMode::LatestOnly => ::langmail::RenderMode::LatestOnly,
        },
    };
    core_email.to_llm_context_with_options(&core_options)
}

// ---------------------------------------------------------------------------
// Internal conversion
// Uses `::langmail::` (absolute path) to avoid collision with the #[pymodule]
// also named `langmail`.
// ---------------------------------------------------------------------------

fn to_core_address(addr: &Address) -> ::langmail::Address {
    ::langmail::Address {
        name: addr.name.clone(),
        email: addr.email.clone(),
    }
}

fn to_py_address(addr: ::langmail::Address) -> Address {
    Address {
        name: addr.name,
        email: addr.email,
    }
}

fn to_core_email(email: &ProcessedEmail) -> ::langmail::ProcessedEmail {
    ::langmail::ProcessedEmail {
        body: email.body.clone(),
        subject: email.subject.clone(),
        from: email.from_address.as_ref().map(to_core_address),
        to: email.to.iter().map(to_core_address).collect(),
        cc: email.cc.iter().map(to_core_address).collect(),
        date: email.date.clone(),
        rfc_message_id: email.rfc_message_id.clone(),
        in_reply_to: email.in_reply_to.clone(),
        references: email.references.clone(),
        signature: email.signature.clone(),
        raw_body_length: email.raw_body_length,
        clean_body_length: email.clean_body_length,
        primary_cta: email
            .primary_cta
            .as_ref()
            .map(|c| ::langmail::CallToAction {
                url: c.url.clone(),
                text: c.text.clone(),
                confidence: c.confidence,
            }),
        thread_messages: email
            .thread_messages
            .iter()
            .map(|m| ::langmail::ThreadMessage {
                sender: m.sender.clone(),
                timestamp: m.timestamp.clone(),
                body: m.body.clone(),
            })
            .collect(),
    }
}

fn to_py_output(result: ::langmail::ProcessedEmail) -> ProcessedEmail {
    ProcessedEmail {
        body: result.body,
        subject: result.subject,
        from_address: result.from.map(to_py_address),
        to: result.to.into_iter().map(to_py_address).collect(),
        cc: result.cc.into_iter().map(to_py_address).collect(),
        date: result.date,
        rfc_message_id: result.rfc_message_id,
        in_reply_to: result.in_reply_to,
        references: result.references,
        signature: result.signature,
        raw_body_length: result.raw_body_length,
        clean_body_length: result.clean_body_length,
        primary_cta: result.primary_cta.map(|c| CallToAction {
            url: c.url,
            text: c.text,
            confidence: c.confidence,
        }),
        thread_messages: result
            .thread_messages
            .into_iter()
            .map(|m| ThreadMessage {
                sender: m.sender,
                timestamp: m.timestamp,
                body: m.body,
            })
            .collect(),
    }
}

/// Email preprocessing for LLMs.
#[pymodule]
fn langmail(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ProcessedEmail>()?;
    m.add_class::<Address>()?;
    m.add_class::<CallToAction>()?;
    m.add_class::<ThreadMessage>()?;
    m.add_class::<RenderMode>()?;
    m.add_class::<LlmContextOptions>()?;
    m.add_class::<PreprocessOptions>()?;
    m.add("ParseError", m.py().get_type::<ParseError>())?;
    m.add_function(wrap_pyfunction!(preprocess, m)?)?;
    m.add_function(wrap_pyfunction!(preprocess_string, m)?)?;
    m.add_function(wrap_pyfunction!(preprocess_with_options, m)?)?;
    m.add_function(wrap_pyfunction!(to_llm_context, m)?)?;
    m.add_function(wrap_pyfunction!(to_llm_context_with_options, m)?)?;
    Ok(())
}
