const { describe, it } = require("node:test");
const assert = require("node:assert/strict");
const path = require("node:path");
const fs = require("node:fs");

const { preprocessGmail } = require("../index.js");

const FIXTURES_DIR = path.join(__dirname, "fixtures", "gmail");
const loadFixture = (name) =>
  JSON.parse(fs.readFileSync(path.join(FIXTURES_DIR, name), "utf8"));

const SIMPLE = loadFixture("simple.json");
const MULTIPART = loadFixture("multipart-alternative.json");
const THREADED = loadFixture("threaded-reply.json");

describe("preprocessGmail", () => {
  describe("simple text/plain message", () => {
    it("parses headers and body", () => {
      const result = preprocessGmail(SIMPLE);

      assert.equal(result.subject, "Hello Bob");
      assert.equal(result.from?.name, "Alice");
      assert.equal(result.from?.email, "alice@example.com");
      assert.equal(result.to.length, 1);
      assert.equal(result.to[0].email, "bob@example.com");
      assert.equal(result.to[0].name, "Bob");
      assert.ok(result.body.includes("Just wanted to say hi!"));
    });

    it("strips angle brackets from Message-ID", () => {
      const result = preprocessGmail(SIMPLE);
      assert.equal(result.rfcMessageId, "abc123@example.com");
    });

    it("converts the Date header to an ISO 8601 string matching the EML path", () => {
      const result = preprocessGmail(SIMPLE);
      // No fractional seconds — matches the Rust `preprocess` formatter so
      // the `date` field is byte-identical across entry points.
      assert.equal(result.date, "2026-02-05T10:00:00Z");
    });

    it("returns undefined for a malformed Date header", () => {
      const msg = JSON.parse(JSON.stringify(SIMPLE));
      const dateHeader = msg.payload.headers.find((h) => h.name === "Date");
      dateHeader.value = "not a real date";
      const result = preprocessGmail(msg);
      assert.equal(result.date, undefined);
    });

    it("strips the signature by default", () => {
      const result = preprocessGmail(SIMPLE);
      assert.ok(!result.body.includes("Alice\nCEO"));
      // The default signature heuristic folds "Best,\nAlice" out of the body.
      assert.ok(!result.body.trimEnd().endsWith("Alice"));
    });
  });

  describe("multipart/alternative message", () => {
    it("prefers the text/html body over text/plain", () => {
      const result = preprocessGmail(MULTIPART);
      // HTML says "Great to hear from you."; plain text says the same, but
      // the HTML also contains a CTA link "View the update now" — that's
      // the tell that the html part was used.
      assert.equal(result.primaryCta?.url, "https://example.com/view");
      assert.ok(result.primaryCta.text.includes("View"));
    });

    it("parses a quoted display name in the Cc list", () => {
      const result = preprocessGmail(MULTIPART);
      assert.equal(result.cc.length, 2);
      assert.equal(result.cc[0].name, "Carol, Support");
      assert.equal(result.cc[0].email, "carol@example.com");
      assert.equal(result.cc[1].email, "dev@example.com");
      assert.equal(result.cc[1].name, undefined);
    });

    it("renders the HTML body as markdown-like text", () => {
      const result = preprocessGmail(MULTIPART);
      assert.ok(result.body.includes("Hi Alice!"));
      assert.ok(result.body.includes("Great to hear from you."));
      // HTML tags must not leak into the output
      assert.ok(!result.body.includes("<p>"));
      assert.ok(!result.body.includes("</body>"));
    });
  });

  describe("threaded reply", () => {
    it("strips the quoted reply content from the body", () => {
      const result = preprocessGmail(THREADED);
      assert.ok(result.body.includes("Thanks for the update, Alice!"));
      assert.ok(!result.body.includes("can you take a look at the PR"));
    });

    it("extracts thread messages from the blockquote", () => {
      const result = preprocessGmail(THREADED);
      assert.ok(
        result.threadMessages.length > 0,
        `expected threadMessages, got ${JSON.stringify(result.threadMessages)}`,
      );
      const first = result.threadMessages[0];
      assert.ok(first.sender.includes("alice@example.com"));
      assert.ok(first.body.includes("take a look at the PR"));
    });

    it("splits References on whitespace and strips angle brackets", () => {
      const result = preprocessGmail(THREADED);
      assert.deepEqual(result.references, [
        "root-000@example.com",
        "abc123@example.com",
      ]);
    });

    it("wraps In-Reply-To in an array", () => {
      const result = preprocessGmail(THREADED);
      assert.deepEqual(result.inReplyTo, ["abc123@example.com"]);
    });

    it("parses a quoted comma-containing From name", () => {
      const result = preprocessGmail(THREADED);
      assert.equal(result.from?.name, "Lastname, Firstname");
      assert.equal(result.from?.email, "firstname@example.com");
    });
  });

  describe("input forms", () => {
    it("accepts a full googleapis response object with .data", () => {
      const result = preprocessGmail({ data: SIMPLE, status: 200 });
      assert.equal(result.subject, "Hello Bob");
    });

    it("forwards options to the pipeline", () => {
      const result = preprocessGmail(THREADED, { stripQuotes: false });
      assert.ok(result.body.includes("take a look at the PR"));
    });

    it("throws on missing payload", () => {
      assert.throws(
        () => preprocessGmail({ id: "x" }),
        /payload is missing/,
      );
    });

    it("throws on non-object input", () => {
      assert.throws(() => preprocessGmail(null), /expected a Gmail message/);
      assert.throws(() => preprocessGmail("raw"), /expected a Gmail message/);
    });
  });

  describe("body edge cases", () => {
    it("returns an empty body when the payload has no data or parts", () => {
      const msg = {
        id: "empty",
        payload: {
          mimeType: "text/plain",
          headers: [{ name: "Subject", value: "Empty" }],
          body: { size: 0 },
        },
      };
      const result = preprocessGmail(msg);
      assert.equal(result.body, "");
      assert.equal(result.subject, "Empty");
    });

    it("throws a clear error when the main body is attachmentId-only", () => {
      const msg = {
        id: "big",
        payload: {
          mimeType: "text/plain",
          headers: [{ name: "Subject", value: "Large body" }],
          body: { size: 9999999, attachmentId: "ATT_abc123" },
        },
      };
      assert.throws(
        () => preprocessGmail(msg),
        /attachmentId.*attachments\.get/s,
      );
    });

    it("degrades gracefully when optional headers are absent", () => {
      const msg = {
        id: "sparse",
        payload: {
          mimeType: "text/plain",
          // No From, no Date, no Message-ID, no To.
          headers: [{ name: "Subject", value: "Minimal" }],
          body: {
            size: 11,
            data: Buffer.from("Hello world", "utf8").toString("base64url"),
          },
        },
      };
      const result = preprocessGmail(msg);
      assert.equal(result.subject, "Minimal");
      assert.equal(result.from, undefined);
      // `to`/`cc` are always arrays on `ProcessedEmail` — empty when absent.
      assert.deepEqual(result.to, []);
      assert.deepEqual(result.cc, []);
      assert.equal(result.date, undefined);
      assert.equal(result.rfcMessageId, undefined);
      assert.equal(result.inReplyTo, undefined);
      assert.equal(result.references, undefined);
      assert.ok(result.body.includes("Hello world"));
    });
  });

  describe("header parsing edge cases", () => {
    it("parses legacy comment-form addresses `email (Name)`", () => {
      const msg = JSON.parse(JSON.stringify(SIMPLE));
      const from = msg.payload.headers.find((h) => h.name === "From");
      from.value = "alice@example.com (Alice Example)";
      const result = preprocessGmail(msg);
      assert.equal(result.from?.email, "alice@example.com");
      assert.equal(result.from?.name, "Alice Example");
    });

    it("treats a whitespace-only In-Reply-To header as absent", () => {
      const msg = JSON.parse(JSON.stringify(SIMPLE));
      msg.payload.headers.push({ name: "In-Reply-To", value: "   " });
      const result = preprocessGmail(msg);
      assert.equal(result.inReplyTo, undefined);
    });

    it("filters out non-email tokens (e.g. `(comment)` suffixes) in References", () => {
      const msg = JSON.parse(JSON.stringify(SIMPLE));
      msg.payload.headers.push({
        name: "References",
        value: "<root@example.com> (imported from archive) <abc@example.com>",
      });
      const result = preprocessGmail(msg);
      assert.deepEqual(result.references, [
        "root@example.com",
        "abc@example.com",
      ]);
    });
  });

  describe("public surface", () => {
    it("does not re-export preprocessParsed from the package entry", () => {
      const pkg = require("../index.js");
      assert.equal(
        pkg.preprocessParsed,
        undefined,
        "preprocessParsed is an internal hook and must not be on the public API",
      );
    });

    it("exports exactly the documented public API — no accidental additions or removals", () => {
      const pkg = require("../index.js");
      const expected = [
        "preprocess",
        "preprocessWithOptions",
        "preprocessString",
        "toLlmContext",
        "toLlmContextWithOptions",
        "NapiRenderMode",
        "RenderMode",
        "preprocessGmail",
      ].sort();
      const actual = Object.keys(pkg).sort();
      assert.deepEqual(actual, expected);
    });

    it("RenderMode is an alias of NapiRenderMode (same enum object)", () => {
      const pkg = require("../index.js");
      assert.strictEqual(pkg.RenderMode, pkg.NapiRenderMode);
    });
  });
});
