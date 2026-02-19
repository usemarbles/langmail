# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**langmail** is a Rust library with Node.js bindings for preprocessing email content for LLM consumption. It strips HTML noise, quoted replies, signatures, and excessive whitespace while preserving structured metadata (headers, threading info, etc.).

## Architecture

### Rust Core (`crates/langmail-core/src/`)

The core processing pipeline (in `lib.rs`, `html.rs`, `quotes.rs`, `signature.rs`, `types.rs`):

1. **MIME Parsing** - Uses `mail-parser` crate (v0.11+) to parse raw RFC 5322 email bytes
2. **Body Extraction** - Prefers plain text, falls back to HTML with conversion
3. **Quote Stripping** (`quotes.rs`) - Regex-based removal of Gmail/Outlook/Apple Mail quoted replies. Supports multilingual patterns (English, German, French, Spanish)
4. **Signature Extraction** (`signature.rs`) - Two-strategy approach:
   - Explicit delimiters (`-- `, "Sent from my iPhone", etc.)
   - Heuristic detection of sign-off patterns near end of message
5. **HTML Cleaning** (`html.rs`) - Custom HTML→text converter that strips tags/scripts/styles while preserving structure

### Node.js Bindings (`crates/langmail-node/src/`)

Built with NAPI-RS (v2.16) for native Node.js bindings. The `lib.rs` file wraps `langmail-core` functions and converts between Rust and N-API types.

### Node.js Package (`packages/langmail/`)

Three main exports:
- `preprocess(Buffer)` - Main API, takes raw email bytes
- `preprocessString(string)` - Convenience wrapper for string input
- `preprocessWithOptions(Buffer, options)` - Configurable version

### Type System (`types.rs`)

- `ProcessedEmail` - Main output structure with cleaned body + metadata
- `PreprocessOptions` - Configuration: `stripQuotes`, `stripSignature`, `maxBodyLength`
- `Address` - Email address with optional display name
- `LangmailError` - Only error type: `ParseFailed`

## Development Commands

### Rust

```bash
# Run Rust tests
cargo test --workspace

# Format code
cargo fmt --all

# Lint with clippy
cargo clippy --workspace -- -D warnings
```

### Node.js

Navigate to the package directory first:

```bash
cd packages/langmail

# Build native module (debug)
npm run build:debug

# Build native module (release)
npm run build

# Run Node.js tests
npm test
# Or directly:
node --test test/preprocess.test.js
```

### CI

GitHub Actions runs three jobs:
1. `test-rust` - Cargo test on Ubuntu
2. `test-node` - Matrix build/test on Ubuntu/macOS/Windows
3. `lint` - Cargo fmt + clippy checks

The CI workflows are located in `.github/workflows/` and test across multiple platforms.

## Key Implementation Notes

### Quote Detection Patterns

The `quotes.rs` module uses regex patterns to detect various quote header styles. When adding support for new email clients or languages:
- Add patterns to the `QUOTE_HEADERS` static
- Ensure patterns use `(?m)` for multiline mode
- Test against actual email samples

### Signature Heuristics

Signature detection combines:
- Explicit delimiters (RFC 3676 `-- ` delimiter)
- "Sent from" patterns (mobile clients)
- Sign-off phrases ("Best regards", "Mit freundlichen Grüßen", etc.)
- Length constraints (max 10 lines)

The `looks_like_signature()` heuristic validates that remaining content after sign-off is short-form (not paragraphs).

### HTML Processing

The HTML→text converter is intentionally simple and doesn't use a full HTML parser. It:
- Manually tracks tag state
- Strips `<script>` and `<style>` blocks entirely
- Converts block elements to newlines
- Handles common HTML entities
- Collapses excessive whitespace

This approach is fast and sufficient for email content.

## Testing

- Rust tests are inline with `#[cfg(test)]` modules in each source file under `crates/langmail-core/src/`
- Node.js tests are in `packages/langmail/test/` and use Node's built-in test runner (requires Node 18+)
- Test fixtures are defined inline as strings/bytes (no external `.eml` files needed)

## mail-parser API Notes

The project uses `mail-parser` v0.11+. Key API details:
- `message.to()` and `message.cc()` return `Option<&Address>` (not `HeaderValue`)
- Use `.iter()` on `Address` to iterate over individual `Addr` items
- Each `Addr` has `name()` and `address()` methods returning `Option<&str>`
