# User Story: Newsletter Detection (`is_newsletter`)

## Summary

As a developer building an LLM email pipeline with langmail, I want `preprocess()` to detect whether an email is a newsletter or bulk marketing message, so I can filter, label, or route it without re-parsing the raw email myself.

## Background

Newsletters and marketing emails require different handling than personal correspondence in LLM pipelines — they're typically filtered out of reply-generation contexts, bucketed separately in inbox digests, or summarised differently. The information needed to make this determination is entirely present in the email headers: standards-compliant bulk senders are required by Gmail and Yahoo (since 2024) to include `List-Unsubscribe` headers. Combined with ESP-specific fingerprints in `Return-Path`, DKIM signing domains, and proprietary `X-*` headers, header-based detection is both highly accurate and zero-cost to compute alongside the existing `mail-parser` pass.

The naive approach of checking inclusion signals alone produces significant false positives: platform notification emails (GitHub, Linear, Jira) include `List-Unsubscribe` and `Precedence: list` by convention but are not newsletters. Detection therefore uses a two-pass architecture — exclusion before inclusion — so that platform notifications are never misclassified regardless of which inclusion signals they carry.

The detection belongs in `preprocess()` — not `to_llm_context()` — because it is a factual property of the email, not a rendering decision.

## Acceptance Criteria

### 1. New field on `ProcessedEmail`

```rust
/// Whether this email was identified as a newsletter or bulk marketing message.
/// Detection is based on email headers only (`List-Unsubscribe`, `List-Id`,
/// `Precedence`, ESP fingerprints); no body content is inspected.
/// Platform notification emails (GitHub, Linear, Jira, etc.) are explicitly
/// excluded even when they carry List-Unsubscribe or Precedence: list headers.
/// Note: this is NOT a spam signal. Newsletters are legitimate opt-in content.
pub is_newsletter: bool,
```

The field is non-optional. Emails that cannot be classified as newsletters return `false`.

---

### 2. Detection logic — two-pass architecture

Newsletter detection runs as a pure header analysis step inside `preprocess()`. The two passes run in order; a match in Pass 1 short-circuits the entire function and returns `false` without evaluating Pass 2.

```rust
fn is_newsletter(headers: &Headers) -> bool {
    // Pass 1: exclusion (any match → false, short-circuit)
    if is_excluded(headers) {
        return false;
    }

    // Pass 2: inclusion (any match → true)
    is_included(headers)
}
```

---

### 3. Pass 1 — Exclusion signals

Any one of the following causes `is_newsletter` to return `false` immediately:

**Threading:**
- `In-Reply-To` header is present — newsletters are one-way broadcasts and are never replies to an existing thread. This catches the majority of GitHub/Linear/Jira notifications, which thread into existing conversations.

**RFC mailing list headers:**
- `List-Post` header is present — defined in RFC 2369 to indicate that subscribers can post back to the list. This is the defining characteristic of a mailing list (open-source project lists, internal company lists), not a newsletter broadcast. GitHub PR/issue notifications carry this header; newsletter platforms do not.

**Platform-specific `X-*` headers:**

The presence of any header matching the following prefixes is an unambiguous exclusion signal. No newsletter platform uses these headers.

```
X-GitHub-
X-GitLab-
X-Linear-
X-Jira-
X-Atlassian-
X-PagerDuty-
X-Sentry-
X-Slack-
X-Notion-
X-Asana-
```

Match against lowercased header names using `starts_with`. No regex needed.

**`Auto-Submitted` value:**
- `Auto-Submitted: auto-notified` → exclusion (notification systems: calendaring, ticketing)
- `Auto-Submitted: auto-generated` → not an exclusion signal; also removed from inclusion signals (see §4)

**Known notification-only sending domains (checked against `Return-Path`):**

```
github.com
gitlab.com
linear.app
atlassian.net
jira.com
pagerduty.com
sentry.io
slack.com
notion.so
asana.com
trello.com
```

These domains are used exclusively for platform notifications and never for newsletter broadcasts. Match against the registered domain extracted from the `Return-Path` address (e.g. `noreply@github.com` → `github.com`).

---

### 4. Pass 2 — Inclusion signals

If not excluded, any one of the following causes `is_newsletter` to return `true`:

**Primary signals (either one is sufficient):**
- `List-Unsubscribe` header is present
- `List-Id` header is present (RFC 2919 — strong positive marker of opt-in broadcast email, independent of `List-Unsubscribe`)

**Supporting signals:**
- `List-Unsubscribe-Post` header is present
- `Precedence` header equals `bulk` (case-insensitive)
  - Note: `Precedence: list` is intentionally absent here. It is an RFC mailing list signal, and mailing lists are handled by the `List-Post` exclusion in Pass 1. Keeping it out of Pass 2 avoids any mailing list that arrives without a `List-Post` header from being misclassified as a newsletter.

**Corroboration note:**
When both `List-Subscribe` and `List-Unsubscribe` are present together, this is a stronger signal than `List-Unsubscribe` alone — it indicates a genuine subscription relationship rather than a boilerplate unsubscribe link added for Gmail/Yahoo compliance. Both headers are covered by the primary `List-Unsubscribe` check, so no additional logic is required; this is documented here for maintainers and for future tuning if the heuristic is ever revisited.

**ESP fingerprint signals:**
- `Return-Path` domain matches a known ESP sending domain (see §5)
- Any DKIM `d=` signing domain matches a known ESP domain (see §5)
- Any header with prefix `X-Mailgun-` is present
- Any header with prefix `X-SES-` is present
- `X-Feedback-Id` header is present
- Any header with prefix `X-Campaign-` or `X-Batch-` is present

> **Note on `Auto-Submitted: auto-generated`:** Removed from inclusion signals relative to the initial design. It adds almost nothing that `List-Unsubscribe` doesn't already cover, and it incorrectly classifies some transactional emails (password resets, order confirmations) that use this header without being newsletters.

---

### 5. Known ESP sending domain list (initial set)

These domains appearing in `Return-Path` or DKIM `d=` indicate bulk newsletter/marketing sending infrastructure:

```
mailgun.org
sendgrid.net
mailchimp.com
mandrillapp.com
amazonses.com
klaviyo.com
brevo.com          // formerly sendinblue
customer.io
postmarkapp.com
sparkpostmail.com
ghost.ghost.org    // Ghost newsletter platform via Mailgun
beehiiv.com
substack.com
convertkit.com
loops.so
buttondown.email
```

This list lives in a private constant in the detection module. It is not part of the public API surface and will be extended over time. Because the exclusion pass runs first, transactional emails from notification platforms that happen to route through SES or Mailgun are already excluded before this check runs.

---

### 6. What does NOT trigger `is_newsletter = true`

- `From` address or display name containing words like "newsletter" or "noreply" — **not a signal**
- Subject line content — **not a signal**
- Body content of any kind — **not inspected**
- `Precedence: list` — absent from inclusion signals; mailing lists are not newsletters
- `Auto-Submitted: auto-generated` — absent from inclusion signals

---

### 7. TDD — test fixtures

Follow TDD: write tests first, then implement to green.

**Fixture 1 — true positive:** `tests/fixtures/saving-your-subscribers.eml`

Copy `__Saving_your_subscribers.eml` from the design session into `tests/fixtures/` under this name. This is a Ghost newsletter sent via Mailgun. Relevant signals:
- `List-Unsubscribe` + `List-Unsubscribe-Post` ✓
- `Return-Path` domain `ghost.ghost.org` ✓
- Dual DKIM: `d=ghost.ghost.org` and `d=mailgun.org` ✓
- `X-Mailgun-*` headers (multiple) ✓
- `X-Feedback-Id` with `mailgun` suffix ✓
- No `In-Reply-To`, no `List-Post`, no platform `X-*` headers → passes exclusion ✓

**Fixture 2 — true negative:** `tests/fixtures/pr-notification-github.eml`

Copy `_usemarbles_langmail__chore__release_v0_10_1__PR__42_.eml` from the design session into `tests/fixtures/` under this name. This is a GitHub PR review comment. It carries three inclusion signals that would misclassify it without the exclusion pass:
- `List-Unsubscribe` + `List-Unsubscribe-Post` ← must be overridden by exclusion
- `Precedence: list` ← must be overridden by exclusion

Exclusion signals that must fire:
- `In-Reply-To: <usemarbles/langmail/pull/42@github.com>` ← threading
- `List-Post` header present ← mailing list signal
- `X-GitHub-Sender`, `X-GitHub-Recipient`, `X-GitHub-Reason` ← platform headers
- `Return-Path: noreply@github.com` → `github.com` notification domain

This fixture is the canonical regression test for the exclusion pass. If the two-pass logic is ever refactored, this test must continue to pass.

**Required test cases:**

```rust
// 1. Ghost/Mailgun newsletter → true
#[test]
fn test_newsletter_ghost_mailgun() {
    let raw = include_bytes!("fixtures/saving-your-subscribers.eml");
    let result = preprocess(raw).unwrap();
    assert!(result.is_newsletter);
}

// 2. GitHub PR notification → false (exclusion overrides inclusion signals)
#[test]
fn test_newsletter_github_pr_notification() {
    let raw = include_bytes!("fixtures/pr-notification-github.eml");
    let result = preprocess(raw).unwrap();
    assert!(!result.is_newsletter);
}

// 3. Personal email (no bulk headers) → false
// Use the existing eventspace-booking.eml fixture.
#[test]
fn test_newsletter_personal_email() {
    let raw = include_bytes!("fixtures/eventspace-booking.eml");
    let result = preprocess(raw).unwrap();
    assert!(!result.is_newsletter);
}

// 4. List-Unsubscribe alone → true (minimal hand-crafted EML)
#[test]
fn test_newsletter_list_unsubscribe_only() { ... }

// 4b. List-Id alone → true (minimal hand-crafted EML, no List-Unsubscribe)
#[test]
fn test_newsletter_list_id_only() { ... }

// 5. In-Reply-To overrides List-Unsubscribe → false
#[test]
fn test_newsletter_in_reply_to_overrides_list_unsubscribe() { ... }

// 6. List-Post overrides Precedence: list → false
#[test]
fn test_newsletter_list_post_overrides_precedence() { ... }

// 7. X-GitHub-* header alone → false
#[test]
fn test_newsletter_xgithub_header_excluded() { ... }

// 8. Precedence: bulk → true
#[test]
fn test_newsletter_precedence_bulk() { ... }

// 9. DKIM d= matching ESP domain → true
#[test]
fn test_newsletter_dkim_esp_domain() { ... }

// 10. X-Mailgun-* header → true
#[test]
fn test_newsletter_xheader_mailgun() { ... }
```

---

### 8. No changes to `to_llm_context()` in this story

`is_newsletter` is available to callers who want to gate or label output, but `to_llm_context()` does not read or act on this field. Render-mode behaviour for newsletter emails is a separate story.

---

### 9. Node.js binding

Expose `is_newsletter` in the `ProcessedEmail` TypeScript type generated by napi-rs:

```typescript
export interface ProcessedEmail {
  // ...existing fields...
  isNewsletter: boolean;
}
```

The field name follows the existing camelCase convention of the Node.js bindings.

---

## Known Limitations

Header-based newsletter detection is a deterministic heuristic, not a classifier. Two grey zones are known up front and are intentionally not solved by this story:

**Transactional-marketing hybrid emails.** Some emails straddle the line between transactional (account-related, user-triggered) and marketing (editorial, promotional). Examples: Stripe's monthly billing summaries, Amazon's "items you might like" recommendations, Apple's receipt emails with cross-promotion, GitHub's billing digest. These carry `List-Unsubscribe` (because Gmail/Yahoo require it for anything resembling bulk) and will be classified as newsletters by this heuristic. Whether that's the right answer is genuinely context-dependent — no header signal can resolve it cleanly. Callers who need finer distinction should layer their own logic on top.

**Platform-sent actual newsletters.** GitHub Discussions weekly digests, Linear's weekly team summary, etc. are real digest-style newsletter emails sent from platform notification domains. Because the exclusion pass short-circuits on the notification domain, these will be classified as `false` (not a newsletter) even though they arguably are. Accepted tradeoff: the exclusion pass is designed to be high-precision for the common case (per-event notifications), at the cost of missing rare digest-style exceptions.

**Not a spam signal.** `is_newsletter = true` says nothing about whether an email is wanted or unwanted. Newsletters are legitimate opt-in content from publishers, creators, and brands. Downstream consumers should not treat this field as a filter-to-spam indicator.

---

## Out of Scope

- ML or LLM-based classification
- Body content inspection of any kind
- Sender address allowlists/denylists
- A configurable sensitivity level (always-strict detection for now)
- `to_llm_context()` rendering changes based on `is_newsletter`

## Implementation Notes

- All detection logic lives in a dedicated `newsletter` module (`langmail-core/src/newsletter.rs`) and is called from `preprocess()`
- The exclusion and inclusion passes are separate private functions (`is_excluded`, `is_included`) for readability and independent testability
- The ESP domain list and the notification domain list are each a `phf::Set` or a sorted `&[&str]` with binary search — no heap allocation at detection time
- Platform header prefix matching (`X-GitHub-*` etc.) uses `starts_with` on lowercased header names, not regex
- Zero new dependencies are expected; `mail-parser` already surfaces all headers needed
