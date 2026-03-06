import langmail
import pytest


SIMPLE_EMAIL = (
    b"From: Alice <alice@example.com>\r\n"
    b"To: Bob <bob@example.com>\r\n"
    b"Subject: Hello Bob\r\n"
    b"Date: Thu, 05 Feb 2026 10:00:00 +0000\r\n"
    b"Message-ID: <abc123@example.com>\r\n"
    b"Content-Type: text/plain; charset=utf-8\r\n"
    b"\r\n"
    b"Hey Bob,\r\n"
    b"\r\n"
    b"Just wanted to say hi!\r\n"
    b"\r\n"
    b"Best,\r\n"
    b"Alice\r\n"
)

REPLY_EMAIL = (
    b"From: Bob <bob@example.com>\r\n"
    b"To: Alice <alice@example.com>\r\n"
    b"Subject: Re: Hello Bob\r\n"
    b"Date: Thu, 05 Feb 2026 11:00:00 +0000\r\n"
    b"Message-ID: <def456@example.com>\r\n"
    b"In-Reply-To: <abc123@example.com>\r\n"
    b"References: <abc123@example.com>\r\n"
    b"Content-Type: text/plain; charset=utf-8\r\n"
    b"\r\n"
    b"Hi Alice!\r\n"
    b"\r\n"
    b"Great to hear from you.\r\n"
    b"\r\n"
    b"On Thu, 05 Feb 2026 at 10:00, Alice <alice@example.com> wrote:\r\n"
    b"> Hey Bob,\r\n"
    b">\r\n"
    b"> Just wanted to say hi!\r\n"
    b">\r\n"
    b"> Best,\r\n"
    b"> Alice\r\n"
)


class TestPreprocess:
    def test_simple_email(self):
        result = langmail.preprocess(SIMPLE_EMAIL)
        assert result.subject == "Hello Bob"
        assert result.from_address.email == "alice@example.com"
        assert result.from_address.name == "Alice"
        assert len(result.to) == 1
        assert result.to[0].email == "bob@example.com"
        assert "Just wanted to say hi!" in result.body

    def test_reply_strips_quotes(self):
        result = langmail.preprocess(REPLY_EMAIL)
        assert "Great to hear from you." in result.body
        assert "Just wanted to say hi!" not in result.body
        assert result.in_reply_to == ["abc123@example.com"]

    def test_clean_body_shorter_than_raw(self):
        result = langmail.preprocess(REPLY_EMAIL)
        assert result.clean_body_length < result.raw_body_length

    def test_date_parsed(self):
        result = langmail.preprocess(SIMPLE_EMAIL)
        assert result.date is not None
        assert "2026-02-05" in result.date

    def test_message_id(self):
        result = langmail.preprocess(SIMPLE_EMAIL)
        assert result.rfc_message_id == "abc123@example.com"


class TestPreprocessString:
    def test_basic(self):
        raw = "From: x@x.com\r\nSubject: Test\r\n\r\nHello world"
        result = langmail.preprocess_string(raw)
        assert result.body == "Hello world"
        assert result.subject == "Test"


class TestPreprocessWithOptions:
    def test_no_strip_quotes(self):
        opts = langmail.PreprocessOptions(strip_quotes=False)
        result = langmail.preprocess_with_options(REPLY_EMAIL, opts)
        assert "Just wanted to say hi!" in result.body
        assert "Great to hear from you." in result.body

    def test_max_body_length(self):
        opts = langmail.PreprocessOptions(max_body_length=5)
        result = langmail.preprocess_with_options(SIMPLE_EMAIL, opts)
        assert len(result.body) <= 5

    def test_default_options_match_preprocess(self):
        a = langmail.preprocess(REPLY_EMAIL)
        opts = langmail.PreprocessOptions()
        b = langmail.preprocess_with_options(REPLY_EMAIL, opts)
        assert a.body == b.body
        assert a.signature == b.signature


class TestToLlmContext:
    def test_format(self):
        result = langmail.preprocess(SIMPLE_EMAIL)
        ctx = langmail.to_llm_context(result)
        assert "FROM: Alice <alice@example.com>" in ctx
        assert "SUBJECT: Hello Bob" in ctx
        assert "CONTENT:" in ctx


class TestParseError:
    def test_invalid_input(self):
        with pytest.raises(langmail.ParseError):
            langmail.preprocess(b"")

    def test_error_is_value_error(self):
        # ParseError is a subclass of ValueError
        with pytest.raises(ValueError):
            langmail.preprocess(b"")


class TestPreprocessOptions:
    def test_defaults(self):
        opts = langmail.PreprocessOptions()
        assert opts.strip_quotes is True
        assert opts.strip_signature is True
        assert opts.max_body_length == 0

    def test_custom(self):
        opts = langmail.PreprocessOptions(
            strip_quotes=False,
            strip_signature=False,
            max_body_length=100,
        )
        assert opts.strip_quotes is False
        assert opts.strip_signature is False
        assert opts.max_body_length == 100

    def test_mutable(self):
        opts = langmail.PreprocessOptions()
        opts.strip_quotes = False
        assert opts.strip_quotes is False
