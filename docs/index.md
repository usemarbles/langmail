---
title: langmail
description: Email preprocessing for LLM consumption. Written in Rust, available for Node.js and Python.
---

# Email preprocessing for LLM consumption

Parse, clean, and structure raw emails into formats optimized for language
models. Written in Rust. Available for Node.js and Python.

[Get Started](introduction.md){ .md-button .md-button--primary }
[View on GitHub](https://github.com/usemarbles/langmail){ .md-button }

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

```sh
# Node.js
npm install langmail

# Python
pip install langmail

# Rust
cargo add langmail
```
