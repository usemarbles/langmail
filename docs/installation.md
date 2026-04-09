---
title: Installation
description: Install langmail for your language of choice. The API is consistent across all three runtimes.
---

## Node.js

Requires Node.js 16 or later. Pre-compiled native binaries are included — no Rust toolchain needed.

```sh
npm install langmail
```

```sh
pnpm add langmail
# or
yarn add langmail
```

## Python

Requires Python 3.8 or later. Wheels are pre-compiled for macOS, Linux, and Windows.

```sh
pip install langmail
```

## Rust

Add `langmail` as a dependency in your `Cargo.toml`.

```sh
cargo add langmail
```

```toml
[dependencies]
langmail = "0.1"
```

!!! note
    Building from source requires Rust 1.70 or later. Pre-compiled binaries for Node.js and Python cover macOS (arm64, x64), Linux (x64, arm64), and Windows (x64).
