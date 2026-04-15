//! Provider-specific adapters that normalize third-party email payloads
//! into langmail's cleaning pipeline.
//!
//! Each adapter consumes a provider's native shape (e.g. Gmail's
//! `gmail_v1.Schema$Message` JSON) and produces the same
//! [`crate::ProcessedEmail`] output as the MIME entry point — the shared
//! post-parse pipeline guarantees byte-identical output across input paths.
//!
//! Adapters live in Rust so every binding (Node, Python, …) inherits them
//! without re-implementing provider logic.

pub mod gmail;

pub use gmail::{preprocess_gmail, preprocess_gmail_with_options};
