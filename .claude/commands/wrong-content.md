Look at the uncommitted email message in the `/crates/langmail/tests/fixtures` folder.

It's content doesn't get extracted correctly. The part before the quoted text which says
"<Somebdoy> wrote on <date>:"
is part of the content, but shouldn't.

Do the following in a TDD manner:
1. Anonymize the email message
2. Create a new integration test
3. Fix
