---
name: anonymize-fixture
description: Anonymize a raw .eml email file for use as a test fixture. Scrubs PII from headers, URLs, and body content while preserving MIME structure, HTML layout, and encoding.
argument-hint: <input.eml> [output.eml]
disable-model-invocation: true
---

# Anonymize Email Fixture

Anonymize the raw email file at `$0` for use as a test fixture. Write the result to `$1` (default: replace the input file in-place).

Read the input file, then systematically apply the rules below. Write the anonymized result to the output path.

See [reference.md](reference.md) for the complete anonymization rules.

## Process

1. **Read** the full .eml file
2. **Identify PII** — sender/recipient names, email addresses, tracking URLs, IPs, message IDs, auth tokens
3. **Classify the sender** — personal name vs. organizational (see reference.md for heuristics)
4. **Apply replacements** following the rules in reference.md, working section by section: headers first, then each MIME part's body
5. **Preserve structure** — MIME boundaries, Content-Transfer-Encoding, quoted-printable soft breaks (`=\r\n`), base64 blocks, HTML tag structure, CSS properties
6. **Write** the anonymized file
7. **Verify** by scanning the output for any remaining PII (real domains, names, IPs, tokens). Report what was changed and flag anything uncertain.
