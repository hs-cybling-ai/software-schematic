## Why

Every successful diagram or Markdown save currently waits for the project MCP to rebuild the entire derived graph, including embeddings and indexes. That work can take about 30 seconds and is on the critical path for closing a diagram, making normal editing feel stalled.

## What Changes

- Include the canonical changed document path and change kind in the private save notification so the MCP can invalidate only affected derived content.
- Acknowledge a durable save after the MCP accepts the refresh request, without waiting for graph compilation, embedding, indexing, or publication.
- Process accepted refreshes in a serialized background worker that coalesces overlapping changes and atomically publishes only complete, validated revisions.
- Incrementally replace the affected diagram, entities, relationships, source-map entries, Markdown chunks, embeddings, and indexes when dependency impact can be determined safely.
- Fall back to a background full rebuild for structural changes whose reachability, identity, rename, deletion, or dependency impact cannot be updated safely in isolation.
- Expose queued, processing, updated, unchanged, and failed refresh state separately from durable document-save success so the editor remains responsive without hiding refresh failures.
- Add performance and concurrency coverage proving that save and tab close do not wait for graph rebuild duration and that rapid saves converge on the newest durable content.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `document-save-graph-refresh`: Replace synchronous complete rebuild-on-save behavior with path-aware, asynchronous refresh scheduling, targeted updates where safe, and background full-rebuild fallback.
- `schematic-property-graph`: Permit validated incremental snapshot derivation while preserving deterministic identity, complete query consistency, and atomic publication.
- `software-schematic-editor`: Report durable save independently from asynchronous graph-refresh progress and ensure tab close is not blocked by graph processing.

## Impact

- `software-schematic-cli/src/lib.rs`: save response and refresh notification payload.
- `software-schematic-cli/src/schematic_mcp.rs`: private control protocol, refresh queue/worker, outcomes, coalescing, and publication lifecycle.
- `software-schematic-cli/src/schematic_graph.rs`: dependency metadata and incremental graph/document rebuild support.
- `software-schematic-web/src/main.js`: persistence state, graph-refresh status handling, and close behavior.
- Existing CLI, MCP, web, and graph tests plus new latency, coalescing, failure, and incremental-equivalence tests.
- The private local notification schema changes; public query-only MCP tools remain unchanged.
