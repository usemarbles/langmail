---
title: langmail
description: Email preprocessing for LLM consumption. Written in Rust, available for Node.js and Python.
---

# Email preprocessing for LLM consumption

Parse, clean, and structure raw emails into formats optimized for language
models. Written in Rust. Available for Node.js and Python.

[Get Started](introduction.md){ .md-button .md-button--primary }
[View on GitHub](https://github.com/usemarbles/langmail){ .md-button }
[llms.txt](llms.txt){ .md-button title="Site summary for LLM clients" }

## What it does

- **HTML to Markdown** — converts HTML email bodies to clean Markdown,
  preserving semantic structure and stripping tracking pixels.
- **Reply detection** — identifies and removes quoted content across Gmail,
  Outlook, Apple Mail, and non-standard clients.
- **Signature stripping** — removes email signatures using heuristic pattern
  matching. No ML, no training data required.
- **CTA extraction** — surfaces calls-to-action by position and structure,
  returning structured data your LLM can act on.

## Quick install

=== "Node.js"

    ```sh
    npm install langmail
    ```

=== "Python"

    ```sh
    pip install langmail
    ```

=== "Rust"

    ```sh
    cargo add langmail
    ```
