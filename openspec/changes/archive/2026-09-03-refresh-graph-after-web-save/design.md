## Context

The web application persists complete diagram XML and connected Markdown through typed Rust endpoints. Separately, a project-bound stdio MCP process holds a derived in-memory Grafeo snapshot. It currently exposes `reload_model`, which lets an LLM request snapshot replacement, while a human web save does not notify it.

The product boundary is intentionally one-way: human-reviewed diagram and Documentation changes flow into a compiled graph, and Codex reads that graph to build code. Codex must not mutate source documents or control what graph revision becomes authoritative.

## Goals / Non-Goals

**Goals:**

- Refresh the running project MCP graph after each successful web save of diagram XML or connected Markdown.
- Publish only a complete valid replacement and preserve the last valid snapshot after refresh failure.
- Make document-save and graph-refresh state understandable in the web UI.
- Keep all LLM-facing MCP operations query-only.

**Non-Goals:**

- Shell update/reload commands, file watching, incremental graph patching, node locking, or agent-authored document changes.
- Allowing Codex to invoke graph refresh or write graph, diagram, Markdown, or Implementation Status data.

## Decisions

### Trigger only after the durable document save

The Rust typed write handler performs its existing complete-file replacement first. Only after that succeeds does it send an internal `documents_changed` notification for the canonical project. Failed or debounced-away writes never trigger refresh. Diagram XML and connected Markdown use the same post-save path.

This trigger lives at the server persistence boundary rather than in browser JavaScript, so a dropped browser response cannot falsely refresh before durable save and every supported web save behaves consistently.

### Rebuild the complete derived snapshot

The notification asks the running MCP process to execute its existing build-then-swap operation. Although one document caused the update, the first version recompiles the complete root-reachable schematic because it is simple, deterministic, and correctly handles changed edges, composition, IDs, Markdown chunks, embeddings, and removed entities. It does not attempt in-place graph mutation.

Refreshes are coalesced and serialized. If saves arrive during a build, one subsequent build covers the newest durable state. A successful build atomically publishes one revision; a failed build leaves the prior revision available.

### Use a private project-local notification channel

The MCP process advertises an ephemeral loopback notification endpoint beneath `.ss/run/`, bound to its canonical project identity and protected by a per-process token. The web server reads only its own project's endpoint metadata and sends only the fixed `documents_changed` operation. The channel is not exposed as MCP, a public HTTP API, or a user command.

If no MCP is running, the document save still succeeds and the UI reports that the graph was not refreshed; the next MCP start compiles current documents normally.

### Remove model-controlled reload

Remove `reload_model` from MCP tool discovery and handling. `get_project_model`, entity lookup, scope resolution, neighborhood traversal, and search remain read-only views. Managed guidance explicitly forbids source or graph mutation from Codex and tells agents to ask the user to evaluate changes in the web application.

### Distinguish save success from refresh success

The write response includes document-save success and graph-refresh outcome/revision as separate fields. The UI may say `Saved; graph updated`, `Saved; MCP not running`, or `Saved; graph update failed`. A graph failure never claims that the authored save failed and never rolls the document back.

## Risks / Trade-offs

- [Rapid autosaves cause redundant embedding work] → Debounce/coalesce notifications in the MCP process and run at most one follow-up build after an active build.
- [A person saves an temporarily incomplete document] → Preserve the last valid snapshot and show the validation failure in the web save state.
- [MCP is not running] → Preserve the document and show a non-blocking stale/not-running state; startup later compiles current source.
- [Private endpoint metadata becomes stale] → Validate token and project identity, fail closed, and replace stale metadata on MCP startup.

## Migration Plan

Ship the post-save notification, MCP listener, read-only tool surface, and UI status in the next project runtime update. Existing diagrams and Markdown require no migration. Older running MCP processes simply cannot receive notifications and are reported as not refreshed until restarted with the current runtime.

## Open Questions

None for the initial one-way document-to-code refresh.
