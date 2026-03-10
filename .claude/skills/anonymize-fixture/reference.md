# Anonymization Reference

## Replacement Values

Use these canonical placeholders consistently:

| PII type | Replacement |
|---|---|
| Personal email | `test@example.com` |
| Noreply/system email | `noreply@example.com` |
| Bounce address | `bounce@example.com` |
| Person's first name | `Max` |
| Person's last name | `Mustermann` |
| Full name | `Max Mustermann` |
| Organization name | Pick a generic name like `ACME Corp` or use `Example <Type>` (e.g. `Example Events`) — keep it recognizable as a business |
| Brand/product name in URLs | Use `example-<brand>.com` (e.g. `example-venue.com`) to keep URLs structurally distinct |
| Profile image URL | `https://example.com/profile.jpg` |
| Generic URL | `https://example.com/` |
| Tracking pixel URL | `https://example.com/tracking.gif` |
| Unsubscribe URL | `https://example.com/unsubscribe` |
| Help URL | `https://example.com/help` |
| Message-ID | `<test-message-id@example.com>` |
| IP address | `192.0.2.1` (TEST-NET per RFC 5737) |
| Phone number | `+49 123 4567890` or `(555) 123-4567` |
| Street address | `Musterstraße 1, 12345 Berlin` |

## Header Rules

### Strip entirely

Remove these headers completely (routing/auth/tracking noise):

- `Received`, `X-Received`
- `Return-Path`
- `DKIM-Signature`, `X-Google-DKIM-Signature`
- `ARC-Seal`, `ARC-Message-Signature`, `ARC-Authentication-Results`
- `Authentication-Results`, `Received-SPF`
- `X-Google-SMTP-Source`, `X-Gm-Message-State`, `X-Gm-Gg`, `X-Gm-Features`
- `Feedback-ID`
- `Require-Recipient-Valid-Since`
- `Sender`, `X-Google-Sender-Delegation`, `X-Google-Sender-Auth`
- `X-LinkedIn-FBL`, `X-LinkedIn-Id`
- Any other header that contains auth tokens, routing info, or tracking IDs

### Anonymize values

| Header | Action |
|---|---|
| `From` | Replace email address with `noreply@example.com`. Replace personal names with `Max Mustermann`. For organizational senders, keep the org display name or use a generic replacement. |
| `To`, `Cc`, `Bcc`, `Delivered-To` | Replace all email addresses with `test@example.com`. Replace display names with `Max Mustermann`. |
| `Message-ID` | Replace with `<test-message-id@example.com>` |
| `References`, `In-Reply-To` | Keep structure but OK to leave as-is (they're message IDs, not PII) — or replace if they contain real domains |
| `Subject` | Replace any personal names. Keep the rest intact (subject line structure matters for testing). |
| `List-Unsubscribe` | Replace URL with `<https://example.com/unsubscribe>` |

### Keep as-is

- `MIME-Version`, `Content-Type`, `Content-Transfer-Encoding`, `Content-Disposition`
- `Date` (usually fine for fixtures)
- Boundary parameters in Content-Type

## Sender Classification

Before doing name-based body replacement, determine if the sender is a person or an organization:

**Organizational signals (skip name replacement in body):**
- **Role address**: local part is one of: `noreply`, `no-reply`, `events`, `info`, `support`, `team`, `hello`, `contact`, `office`, `booking`, `sales`, `admin`, `press`, `marketing`, `newsletter`, `notifications`, `service`
- **Business words** in display name: `Events`, `Team`, `Support`, `GmbH`, `Inc`, `LLC`, `Ltd`, `Corp`, `Foundation`, `Service(s)`, `Newsletter`, `News`, `Notifications`, `Alerts`
- **Non-name patterns**: all-caps with digits (e.g. "WERK1"), single word that's clearly a brand

When the sender is organizational, do NOT search-and-replace the display name parts through the body — this causes corruption (e.g. "WERK1 Events" → replacing "Events" throughout the file breaks "Events Team", URLs containing "events", etc.).

Instead, only anonymize the From header's email address and leave the display name as-is (or replace with a fitting generic org name).

## Body / URL Rules

### URLs to anonymize

Replace these with appropriate `example.com` variants:

- **LinkedIn**: messaging threads, tracking pixels (`/emimp/`), CDN images (`media.licdn.com`), static assets (`static.licdn.com`), profile URLs (`/in/username`), unsubscribe (`/psettings/`), help, any remaining `linkedin.com/*`
- **Google user content**: `lh*.googleusercontent.com/*` (signature images)
- **Google review**: `g.page/*`
- **Meetup**: `meetup.com/*`
- **Monday.com**: `monday.com/*`
- **Any other third-party URL** that isn't a standard namespace (like `w3.org`) or already an `example*.com` domain

### URLs to preserve

- `example.com`, `example-*.com` (already anonymized)
- `w3.org` (XML namespaces, standards — structural, not PII)
- `schemas.microsoft.com` and similar standards URIs
- `mailto:` links (anonymize the email address inside, keep the `mailto:` scheme)

### Tracking parameters

Strip or replace values of tracking query parameters:
- `midToken`, `midSig`, `otpToken`, `eid`, `loid`, `lipi`, `trk`, `trkEmail`
- UTM parameters (`utm_source`, `utm_medium`, `utm_campaign`, etc.)
- Any parameter that looks like a session/tracking token

### Email addresses in body

Replace all real email addresses with `test@example.com`.

### Names in body

- For **personal senders**: replace first name, last name, and full name occurrences with `Max` / `Mustermann` / `Max Mustermann`
- For **recipients**: same treatment — extract name from To header (display name or derive from local part), replace in body
- Be careful with **case-sensitive context**: `max-width` in CSS is NOT the name "Max" — don't replace inside CSS properties, HTML attributes, or code-like contexts
- Use word-boundary-aware matching — don't replace substrings inside longer words

### Other PII

- IP addresses → `192.0.2.1`
- Phone numbers → generic placeholder
- Physical addresses → generic placeholder
- Account numbers, policy numbers, reference codes → `REDACTED` or `TEST-123456`

## Encoding Awareness

### Quoted-Printable

QP-encoded parts use `=XX` for non-ASCII bytes and `=\r\n` (soft line break) for line wrapping at 76 chars. When replacing text:
- The replacement may change line lengths — re-wrap lines exceeding 76 chars
- Soft breaks can split a URL or token mid-word — consider the unwrapped text when identifying PII
- After replacement, ensure `=XX` sequences are still valid

### Base64

Base64-encoded parts should generally be left alone unless you can decode, anonymize, and re-encode them. For test fixtures, it's usually acceptable to leave base64 blocks as-is (they often contain images or attachments that don't need anonymization for testing purposes).

## Verification Checklist

After anonymizing, scan the output for:
- [ ] Real email domains (anything not `example*.com`)
- [ ] Real person names (check From/To originals against body)
- [ ] Third-party URLs (anything not `example*.com`, `w3.org`, or standard schemas)
- [ ] IP addresses that aren't `192.0.2.1` or well-known
- [ ] Phone numbers
- [ ] Physical addresses
- [ ] Tracking tokens or session IDs in URLs

Report a summary of changes made and flag anything you're unsure about.
