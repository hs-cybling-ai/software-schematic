## Why

Every connected Markdown save deliberately publishes a text-searchable graph before asynchronous vector derivation finishes, but the refresh API flattens the expected missing-header diagnostic into a generic warning. The editor consequently shows `Graph rebuilt from main.cmmn; … embedding header is missing; vector retrieval is pending` on every save even though persistence and graph publication succeeded, making normal progress look like an error and obscuring real failures.

## What Changes

- Keep graph-build diagnostics structured so expected vector-readiness conditions are not promoted to graph-refresh warnings.
- Report the published graph's retrieval mode and vector readiness independently from actionable graph diagnostics in refresh status.
- Make the editor present a Markdown save with pending embeddings as normal background progress and avoid warning toasts for that transient state.
- Preserve actionable reporting for malformed/stale artifacts, embedding derivation failures, and authoritative graph-refresh failures.
- Add regression coverage for the complete two-publication sequence: body save produces a text-ready graph with vectors pending, then header publication produces a hybrid-ready graph without a false warning.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `document-save-graph-refresh`: Distinguish successful text-ready publication with pending vectors from actionable graph-refresh warnings and failures in typed refresh state.
- `software-schematic-editor`: Present expected asynchronous embedding work as progress rather than an error toast while continuing to surface real derivation and graph failures.

## Impact

- Affects graph diagnostic classification and refresh outcome serialization in `software-schematic-cli/src/schematic_graph.rs` and `software-schematic-cli/src/schematic_mcp.rs`.
- Affects save-status rendering and toast behavior in `software-schematic-web/src/main.js` and the packaged web assets.
- Extends Rust refresh tests and web status/UI contract tests; no dependency, file-format, MCP mutation-surface, or authored Markdown compatibility change is required.
