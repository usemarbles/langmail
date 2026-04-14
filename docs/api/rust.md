---
title: Rust API
description: The Rust crate's public surface, with pointers to rustdoc on docs.rs.
---

The Rust API is documented inline via rustdoc. See [**docs.rs/langmail**](https://docs.rs/langmail) for the authoritative, versioned reference — including error variants, method receivers, and `serde` derivations.

## Surface at a glance

Entry points and options. Payload structs (`Address`, `CallToAction`,
`ThreadMessage`) and the provider-adapter entry point (`preprocess_parsed` /
`ParsedInput`) are documented on [docs.rs/langmail](https://docs.rs/langmail).

| Item | Description |
| --- | --- |
| [`preprocess`](https://docs.rs/langmail/latest/langmail/fn.preprocess.html) | `fn preprocess(raw: &[u8]) -> Result<ProcessedEmail, LangmailError>` |
| [`preprocess_with_options`](https://docs.rs/langmail/latest/langmail/fn.preprocess_with_options.html) | Preprocess with a custom `PreprocessOptions` |
| [`adapters::preprocess_gmail`](https://docs.rs/langmail/latest/langmail/adapters/gmail/fn.preprocess_gmail.html) | `fn preprocess_gmail(msg_json: &str) -> Result<ProcessedEmail, LangmailError>` — normalize a Gmail API `Schema$Message` (JSON) through the shared pipeline |
| [`adapters::preprocess_gmail_with_options`](https://docs.rs/langmail/latest/langmail/adapters/gmail/fn.preprocess_gmail_with_options.html) | As above, with a `PreprocessOptions` override |
| [`ProcessedEmail`](https://docs.rs/langmail/latest/langmail/struct.ProcessedEmail.html) | The parsed output struct |
| [`ProcessedEmail::to_llm_context`](https://docs.rs/langmail/latest/langmail/struct.ProcessedEmail.html#method.to_llm_context) | Method on `ProcessedEmail` returning the LLM-ready string |
| [`ProcessedEmail::to_llm_context_with_options`](https://docs.rs/langmail/latest/langmail/struct.ProcessedEmail.html#method.to_llm_context_with_options) | As above, with a `LlmContextOptions` |
| [`PreprocessOptions`](https://docs.rs/langmail/latest/langmail/struct.PreprocessOptions.html) | `strip_quotes`, `strip_signature`, `max_body_length` |
| [`LlmContextOptions`](https://docs.rs/langmail/latest/langmail/struct.LlmContextOptions.html) | `render_mode: RenderMode` |
| [`RenderMode`](https://docs.rs/langmail/latest/langmail/enum.RenderMode.html) | `LatestOnly` \| `ThreadHistory` |
| [`LangmailError`](https://docs.rs/langmail/latest/langmail/enum.LangmailError.html) | Error variants returned by `preprocess` |

## Notes

- `preprocess` returns a `Result`; unlike the Node and Python bindings, there is no exception channel.
- `to_llm_context` is a method on `ProcessedEmail`, not a free function.
- All data types (`ProcessedEmail`, `Address`, `CallToAction`, `ThreadMessage`, `PreprocessOptions`, `LlmContextOptions`, `RenderMode`) derive `serde::Serialize` / `Deserialize` for JSON round-tripping. `LangmailError` does not.
- Provider adapters live under `langmail::adapters`. They take the provider's JSON as `&str` — serialize a Gmail `users.messages.get` response with `serde_json::to_string` and pass the result. The adapter walks `payload.parts`, base64url-decodes the bodies, and feeds them into the same cleaning pipeline as `preprocess`, so the output is byte-identical to the MIME path. Gmail-specific errors are raised as `LangmailError::InvalidGmailMessage` and `LangmailError::BodyRequiresAttachmentFetch`.
