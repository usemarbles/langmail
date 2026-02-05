const { describe, it } = require("node:test");
const assert = require("node:assert/strict");
const { preprocess, preprocessString, preprocessWithOptions } = require("../index.js");

const SIMPLE_EMAIL = Buffer.from(
  [
    "From: Alice <alice@example.com>",
    "To: Bob <bob@example.com>",
    "Subject: Hello Bob",
    "Date: Thu, 05 Feb 2026 10:00:00 +0000",
    "Message-ID: <abc123@example.com>",
    "Content-Type: text/plain; charset=utf-8",
    "",
    "Hey Bob,",
    "",
    "Just wanted to say hi!",
    "",
    "Best,",
    "Alice",
  ].join("\r\n")
);

const REPLY_EMAIL = Buffer.from(
  [
    "From: Bob <bob@example.com>",
    "To: Alice <alice@example.com>",
    "Subject: Re: Hello Bob",
    "Date: Thu, 05 Feb 2026 11:00:00 +0000",
    "Message-ID: <def456@example.com>",
    "In-Reply-To: <abc123@example.com>",
    "References: <abc123@example.com>",
    "Content-Type: text/plain; charset=utf-8",
    "",
    "Hi Alice!",
    "",
    "Great to hear from you.",
    "",
    "On Thu, 05 Feb 2026 at 10:00, Alice <alice@example.com> wrote:",
    "> Hey Bob,",
    ">",
    "> Just wanted to say hi!",
    ">",
    "> Best,",
    "> Alice",
  ].join("\r\n")
);

describe("langmail", () => {
  describe("preprocess", () => {
    it("parses a simple email", () => {
      const result = preprocess(SIMPLE_EMAIL);

      assert.equal(result.subject, "Hello Bob");
      assert.equal(result.from?.email, "alice@example.com");
      assert.equal(result.from?.name, "Alice");
      assert.equal(result.to.length, 1);
      assert.equal(result.to[0].email, "bob@example.com");
      assert.ok(result.body.includes("Just wanted to say hi!"));
    });

    it("strips quoted replies", () => {
      const result = preprocess(REPLY_EMAIL);

      assert.ok(result.body.includes("Great to hear from you."));
      assert.ok(!result.body.includes("Just wanted to say hi!"));
    });

    it("preserves threading metadata", () => {
      const result = preprocess(REPLY_EMAIL);

      assert.deepEqual(result.inReplyTo, ["abc123@example.com"]);
      assert.deepEqual(result.references, ["abc123@example.com"]);
    });

    it("reports body length reduction", () => {
      const result = preprocess(REPLY_EMAIL);

      assert.ok(result.cleanBodyLength < result.rawBodyLength);
    });
  });

  describe("preprocessString", () => {
    it("accepts a string instead of Buffer", () => {
      const raw = SIMPLE_EMAIL.toString("utf-8");
      const result = preprocessString(raw);

      assert.equal(result.subject, "Hello Bob");
      assert.ok(result.body.includes("Just wanted to say hi!"));
    });
  });

  describe("preprocessWithOptions", () => {
    it("can disable quote stripping", () => {
      const result = preprocessWithOptions(REPLY_EMAIL, {
        stripQuotes: false,
      });

      assert.ok(result.body.includes("Just wanted to say hi!"));
    });

    it("can limit body length", () => {
      const result = preprocessWithOptions(SIMPLE_EMAIL, {
        maxBodyLength: 10,
      });

      assert.ok(result.body.length <= 10);
    });
  });
});
