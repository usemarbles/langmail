# Spec 07 — MCP server (`langmail-mcp`)

**Status:** draft
**Priority:** P2

## Problem

langmail's target audience — developers building AI apps with Claude — increasingly works inside MCP-aware clients: Claude Code, Claude Desktop, Cursor, Windsurf, Zed. For that audience, the shortest path to "try it" is not `npm install` — it's adding an MCP server to a config file.

An MCP server also gives langmail zero-install distribution into conversations. A user asking Claude Code "summarize this email thread" could trigger a tool that preprocesses the email *before* it hits the model, automatically.

## Goal

Ship `langmail-mcp` as a tiny package that wraps the core library as an MCP server. Runnable via `npx -y langmail-mcp` with no install step.

## Surface

**Package location**: `packages/langmail-mcp/`
**Binary**: `langmail-mcp` (declared in `package.json` `bin`)
**Invocation**: `npx -y langmail-mcp` (stdio transport, the standard for local MCP clients)

### Tools exposed

1. **`preprocess_email`** — input: raw email string *or* parsed provider payload (discriminated union). Output: `ProcessedEmail` as JSON.
2. **`email_to_llm_context`** — input: raw email string, optional `renderMode`. Output: string (the `toLlmContext` output).
3. **`build_thread_context`** — input: array of raw emails or `ProcessedEmail`s. Output: string thread transcript (see spec 06).
4. **`preprocess_gmail`** — input: Gmail API `Schema$Message` JSON. Output: `ProcessedEmail`.

Each tool has a clear `description` explaining *when* Claude should call it. No resources or prompts in v1 — tools only.

### Example client config

Claude Desktop (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "langmail": {
      "command": "npx",
      "args": ["-y", "langmail-mcp"]
    }
  }
}
```

Claude Code:

```bash
claude mcp add langmail npx -y langmail-mcp
```

## Implementation notes

- Built on `@modelcontextprotocol/sdk` (the official TypeScript SDK).
- Zero dependencies beyond `langmail` and the MCP SDK.
- Stateless — each tool call is independent. No filesystem, no network.
- ~100 lines of code. Thin wrapper; no business logic.
- Ships pure JS, no native binary required in the `mcp` package itself (it depends on `langmail`, which handles binary distribution).

## Documentation

New `docs/mcp.md` with:

- 60-second "add to Claude Desktop / Claude Code / Cursor" quickstart.
- Screenshots of Claude in a client calling the tools on a real email.
- Link to the MCP spec for readers who don't know what MCP is.

README gets a new "Use in Claude Code / Claude Desktop" section near the top with the `claude mcp add` one-liner.

## Scope

**In:**
- The four tools above.
- stdio transport.
- Published as `langmail-mcp` on npm.
- Config snippets for Claude Desktop, Claude Code, Cursor.

**Out:**
- HTTP / SSE transport (stdio is sufficient for local MCP clients; HTTP is a follow-up if demand appears).
- Auth / multi-tenant (the server is single-user by design).
- Any Gmail / provider API calls — the server processes content the user already has.
- Resources or prompts — tools only in v1.

## Success criteria

- A user can add langmail to Claude Desktop in under 60 seconds with a single config paste.
- README's MCP section is reachable above the fold.
- At least one of the four tools shows up in the user's client immediately with a useful description.

## Risk / open questions

- Tool selection: does Claude reliably pick `preprocess_email` over "just read the file"? The tool description needs to be explicit: *"Always call this before summarising or classifying an email — it removes ~70% of irrelevant content and produces structured metadata."*
- Discovery: being in the MCP registry (once it lands) matters. Track that as a follow-up.
- Versioning: `langmail-mcp` version should track `langmail` major version to avoid mismatch confusion.
