## Why

Creating the project MCP and its in-memory graph still initializes the embedding model and embeds Markdown, so startup can take minutes even when authored content has not changed. Embeddings are derived from Markdown and should be computed when that Markdown changes, then reused so normal MCP and graph creation completes in a few seconds.

## What Changes

- Move Markdown chunk embedding generation out of MCP/graph construction and into a project-scoped asynchronous derivation worker triggered after a durable Markdown update.
- Define a canonical, versioned embedding header that the editor, CLI, or another Markdown code generator can place directly in each connected Markdown file. Record owner, authored-body hash, chunking version, model identity, dimensions, and vectors, and hide the reserved header from rendered and source-editing UI.
- Make graph construction validate and import current persisted embeddings instead of loading an embedding model or recomputing vectors on its critical path.
- Allow a graph to become queryable with text retrieval and an explicit degraded/stale-vector status when a valid embedding artifact is absent; asynchronously repair the artifact and atomically enable vector retrieval when ready.
- Coalesce repeated Markdown updates by owner, prevent an older embedding result from replacing a newer one, and keep derived-header writes from racing with authored Markdown edits.
- Add migration/backfill support for existing projects plus timing, consistency, corruption, model-version, rename/delete, and concurrent-save coverage.

## Capabilities

### New Capabilities

- `persisted-markdown-embeddings`: Defines versioned, owner-bound embedding artifacts, asynchronous derivation, validation, lifecycle, and backfill behavior.

### Modified Capabilities

- `schematic-property-graph`: Graph construction consumes validated precomputed vectors and remains promptly queryable when vector artifacts are missing or stale.
- `document-save-graph-refresh`: Durable Markdown updates schedule both embedding derivation and graph refresh with content-version ordering and coalescing.
- `software-schematic-editor`: Save/refresh status distinguishes authored persistence, embedding readiness, and graph readiness without blocking editing.

## Impact

- `software-schematic-cli/src/schematic_graph.rs`: remove critical-path embedding generation; validate/import persisted vectors; expose degraded vector readiness.
- `software-schematic-cli/src/schematic_mcp.rs`: startup and refresh lifecycle, atomic vector-index activation, and status reporting.
- `software-schematic-cli/src/lib.rs`: post-save derivation scheduling, backfill command/server behavior, canonical header parsing, UI projection, and conflict-safe Markdown persistence.
- Connected Markdown gains a reserved SSW derived-metadata header that external generators may emit; its authored body remains the source of truth and no diagram rewrite is required.
- Packaged embedding-model/runtime use moves to the derivation worker and migration/backfill path rather than normal MCP startup.
- Existing projects remain readable and receive embeddings asynchronously; query-only public MCP tools remain unchanged apart from readiness/diagnostic fields in existing responses.
