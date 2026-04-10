Findings

  100% consistent across all 5 sampled notes:

  ┌────────────────────────────────────────┬───────────────────────────────────┬──────────────────────────────────────────────────────┐
  │                 Signal                 │               Value               │                     Reliability                      │
  ├────────────────────────────────────────┼───────────────────────────────────┼──────────────────────────────────────────────────────┤
  │ X-Uniform-Type-Identifier header       │ com.apple.mail-note               │ Best single signal - unique to Apple Notes           │
  ├────────────────────────────────────────┼───────────────────────────────────┼──────────────────────────────────────────────────────┤
  │ Gmail label                            │ Label_6 (name: Notes, type: user) │ Reliable but label ID varies per account             │
  ├────────────────────────────────────────┼───────────────────────────────────┼──────────────────────────────────────────────────────┤
  │ No To header                           │ Always absent                     │ Already detected but imprecise (also catches drafts) │
  ├────────────────────────────────────────┼───────────────────────────────────┼──────────────────────────────────────────────────────┤
  │ From = own email                       │ Account owner's address           │ Necessary but not sufficient alone                   │
  ├────────────────────────────────────────┼───────────────────────────────────┼──────────────────────────────────────────────────────┤
  │ X-Mail-Created-Date header             │ Present on all notes              │ Apple-specific header                                │
  ├────────────────────────────────────────┼───────────────────────────────────┼──────────────────────────────────────────────────────┤
  │ X-Universally-Unique-Identifier header │ UUID format                       │ Apple-specific header                                │
  ├────────────────────────────────────────┼───────────────────────────────────┼──────────────────────────────────────────────────────┤
  │ MIME type                              │ text/html (simple, no multipart)  │ Consistent but not unique to notes                   │
  ├────────────────────────────────────────┼───────────────────────────────────┼──────────────────────────────────────────────────────┤
  │ Mime-Version                           │ Contains dataaccessd/1.0          │ Apple sync daemon signature                          │
  └────────────────────────────────────────┴───────────────────────────────────┴──────────────────────────────────────────────────────┘

  The most reliable detection approach (in order of preference):

  1. Header-based: Check for X-Uniform-Type-Identifier: com.apple.mail-note -- this is the definitive signal
  2. Label-based: Check if message has a label named Notes -- fast, doesn't require raw message parsing
  3. Combined with existing: No To recipients + From is own email (current heuristic, but less precise)

  The X-Uniform-Type-Identifier header is the gold standard since it's set by Apple's IMAP sync daemon (dataaccessd) and uniquely identifies the message as a note, not a regular
  email.
