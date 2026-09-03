## 1. One-Way Refresh Boundary

- [x] 1.1 Extract the MCP snapshot build-then-swap operation into serialized/coalescing server state while preserving complete-build validation and rollback.
- [x] 1.2 Add the private project-bound document-change listener and ephemeral authenticated endpoint metadata beneath `.ss/run/`.
- [x] 1.3 Notify the matching running MCP only after successful typed CMMN, BPMN, diagram Markdown, or element Markdown persistence.

## 2. Query-Only MCP

- [x] 2.1 Remove `reload_model` from MCP tool discovery, schemas, handlers, documentation, and managed agent guidance.
- [x] 2.2 Add guidance requiring Codex to stop and ask the user to evaluate and save missing or incorrect contracts in the web application rather than mutating or refreshing project documents or graph state.

## 3. Save and Refresh Feedback

- [x] 3.1 Extend typed write responses with separate durable-save and graph-refresh outcomes, including published/prior revision and bounded diagnostics.
- [x] 3.2 Update web save-state presentation for `saved and graph updated`, `saved and MCP not running`, and `saved but graph update failed` without rolling back authored documents.

## 4. Verification

- [x] 4.1 Add Rust tests for post-save-only notification, exact project binding, no-running-MCP behavior, atomic replacement, failed-build rollback, coalesced saves, and stale endpoint metadata.
- [x] 4.2 Add MCP tests proving every advertised tool is query-only and `reload_model` is unavailable.
- [x] 4.3 Add web tests for the three save/refresh states and preservation of successful saves when graph refresh fails.
- [x] 4.4 Run formatting, linting, Rust/web suites, strict OpenSpec validation, and an end-to-end human web-save-to-MCP-query smoke test.
