const { describe, it } = require("node:test");
const assert = require("node:assert/strict");

const { preprocessGmail } = require("../index.js");

// Detailed behavioral coverage for `preprocessGmail` lives in Rust at
// `crates/langmail/tests/gmail_adapter.rs` — the logic that used to be
// exercised here now lives in `crates/langmail/src/adapters/gmail.rs` and
// is tested directly against the core API. This file only verifies that
// the Node binding is wired up correctly:
//   - a well-formed message round-trips through the binding,
//   - options reach the pipeline, and
//   - JS-side and Rust-side errors surface with useful messages.

// Minimal Gmail message; `SGVsbG8gd29ybGQ=` is base64url("Hello world").
const SMOKE_MESSAGE = {
  id: "smoke",
  payload: {
    mimeType: "text/plain",
    headers: [
      { name: "Subject", value: "Smoke test" },
      { name: "From", value: "Alice <alice@example.com>" },
      { name: "To", value: "bob@example.com" },
      {
        name: "Message-ID",
        value: "<smoke-001@example.com>",
      },
    ],
    body: {
      size: 11,
      data: Buffer.from("Hello world", "utf8").toString("base64url"),
    },
  },
};

describe("preprocessGmail (Node binding smoke tests)", () => {
  it("round-trips a well-formed Gmail message through the binding", () => {
    const result = preprocessGmail(SMOKE_MESSAGE);

    assert.equal(result.subject, "Smoke test");
    assert.equal(result.from?.email, "alice@example.com");
    assert.equal(result.from?.name, "Alice");
    assert.deepEqual(
      result.to.map((a) => a.email),
      ["bob@example.com"],
    );
    assert.equal(result.rfcMessageId, "smoke-001@example.com");
    assert.ok(result.body.includes("Hello world"));
  });

  it("forwards options to the Rust pipeline", () => {
    // `maxBodyLength: 5` must truncate the cleaned body; a no-op binding
    // would ignore the option and return the full text.
    const result = preprocessGmail(SMOKE_MESSAGE, { maxBodyLength: 5 });
    assert.ok(
      result.body.length <= 5,
      `expected body ≤ 5 chars, got ${result.body.length}: ${JSON.stringify(result.body)}`,
    );
  });

  it("throws a TypeError from JS when input is not an object", () => {
    assert.throws(
      () => preprocessGmail(null),
      (err) =>
        err instanceof TypeError && /expected a Gmail message/.test(err.message),
    );
    assert.throws(
      () => preprocessGmail("raw string"),
      (err) =>
        err instanceof TypeError && /expected a Gmail message/.test(err.message),
    );
  });

  it("surfaces Rust-side errors (missing payload) with an actionable message", () => {
    assert.throws(
      () => preprocessGmail({ id: "x" }),
      /payload is missing/,
    );
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
