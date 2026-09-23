## 1. Refresh Protocol and State

- [x] 1.1 Replace the fixed no-payload control request with a versioned, authenticated request carrying notification ID, normalized project-relative path, and change kind, including confinement and supported-extension validation tests.
- [x] 1.2 Expand graph-refresh outcomes and status data to represent queued, processing, updated, unchanged, failed, rejected/not-running states with affected paths, revisions, timestamps, and diagnostics.
- [x] 1.3 Add a read-only typed-server graph-refresh status endpoint that queries the matching project MCP without exposing refresh or mutation through public MCP tools.

## 2. Background Refresh Coordinator

- [x] 2.1 Add one project-scoped refresh coordinator that acknowledges accepted work before graph construction begins and runs builds outside the save request path.
- [x] 2.2 Implement path-keyed latest-state coalescing, finite batch draining, serialized builds, and follow-up processing for notifications accepted during an active build.
- [x] 2.3 Preserve last-known-good snapshots and record actionable asynchronous failure state when a candidate build fails.
- [x] 2.4 Add concurrency tests proving rapid and overlapping notifications converge on the newest durable content without concurrent builds or stale publication.

## 3. Incremental Graph Construction

- [x] 3.1 Add snapshot-scoped source manifests, content hashes, parsed diagram fragments, composition dependency/reverse-dependency metadata, and content-addressed embedding metadata.
- [x] 3.2 Implement targeted Markdown invalidation that regenerates only the owning document chunks, embeddings, and relevant index content while reusing unaffected artifacts.
- [x] 3.3 Implement diagram invalidation across the safely determined reverse-dependency closure, including removal of obsolete entities, relationships, citations, chunks, and index content.
- [x] 3.4 Route root-reachability, identity, rename, delete, unsupported, and uncertain dependency changes to the existing complete loader in the background.
- [x] 3.5 Validate all incremental candidates before the atomic `Arc` swap and fall back to a complete build when incremental safety or equivalence cannot be established.

## 4. Save and Editor Integration

- [x] 4.1 Update atomic document writes to send the saved path and change kind, return after bounded enqueue acknowledgement, and keep persistence failure distinct from refresh scheduling state.
- [x] 4.2 Update browser persistence state and messaging for queued, processing, updated, unchanged, failed, and not-running refresh outcomes, polling only while work is pending.
- [x] 4.3 Update tab disposal so it flushes durable diagram persistence but does not wait for background graph processing before selecting the next tab and releasing modeler resources.
- [x] 4.4 Add browser/server tests with a deliberately blocked graph builder proving save and permitted tab close complete within a graph-size-independent latency budget.

## 5. Correctness, Performance, and Documentation

- [x] 5.1 Add equivalence tests comparing incremental and clean full builds for revisions, entities, relationships, citations, chunks, lookup, traversal, scope resolution, and search behavior.
- [x] 5.2 Add failure and recovery tests for malformed saves, embedding/index errors, MCP absence/restart, invalid paths, and successful refresh after a prior failure.
- [x] 5.3 Benchmark representative diagram and Markdown edits to verify unaffected parsing and embeddings are reused and capture whether remaining time is dominated by Grafeo index reconstruction.
- [x] 5.4 Update repository and MCP documentation to describe asynchronous save-triggered refresh, transient stale-query behavior, lifecycle status, diagnostics, coalescing, and full-rebuild fallback.
- [x] 5.5 Run the focused Rust and web test suites plus the complete project quality checks and record any platform-specific timing thresholds.
