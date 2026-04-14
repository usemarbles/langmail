---
title: Rust API
description: The Rust crate's public surface, with pointers to rustdoc on docs.rs.
---

**Language:** [TypeScript](typescript.md) · [Python](python.md) · Rust — see also [Concepts](../concepts.md)

The Rust API is documented inline via rustdoc. See [**docs.rs/langmail**](https://docs.rs/langmail) for the authoritative, versioned reference — including error variants, method receivers, and `serde` derivations.

## Surface at a glance

Entry points and options. Payload structs (`Address`, `CallToAction`,
`ThreadMessage`) and the provider-adapter entry point (`preprocess_parsed` /
`ParsedInput`) are documented on [docs.rs/langmail](https://docs.rs/langmail).

| Item | Description |
| --- | --- |
| [`preprocess`](https://docs.rs/langmail/latest/langmail/fn.preprocess.html) | `fn preprocess(raw: &[u8]) -> Result<ProcessedEmail, LangmailError>` |
| [`preprocess_with_options`](https://docs.rs/langmail/latest/langmail/fn.preprocess_with_options.html) | Preprocess with a custom `PreprocessOptions` |
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
- There is no Rust equivalent of `preprocessGmail`. Provider adapters are intentionally confined to the Node binding layer — the Rust crate stays provider-agnostic so upstream MIME or JSON sources can be used directly.
