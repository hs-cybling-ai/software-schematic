## 1. Canonical Markdown Envelope

- [x] 1.1 Define the bounded `ssw.embedding/v1` envelope types, canonical serialization, exact authored-body boundary, and source/embedding revision identities; verify deterministic round-trip and stable hash unit tests.
- [x] 1.2 Benchmark JSON numeric vectors against a compact encoded float payload on representative documents, freeze the v1 encoding with documented limits, and verify fixtures remain portable across supported platforms.
- [x] 1.3 Implement strict leading-envelope parsing and validation for schema version, owner, body/chunk hashes, chunker/model identity, dimensions, finite vectors, and resource limits; verify malformed, oversized, stale, unknown-version, and adversarial fixtures degrade safely.
- [x] 1.4 Add body-only Markdown read/write helpers that hide the envelope, preserve authored bytes, and atomically replace complete files; verify headerless, valid-header, malformed-header, existing-front-matter, Unicode, and line-ending cases.

## 2. Asynchronous Embedding Derivation

- [x] 2.1 Extract chunking and embedding generation into a project-scoped derivation service independent of graph construction; verify equivalent chunk identities and vectors against current deterministic and production-profile tests.
- [x] 2.2 Add per-owner latest-state scheduling after durable Markdown saves with bounded concurrency and coalescing; verify save acknowledgement remains fast while a deliberately blocked model is running.
- [x] 2.3 Implement compare-and-swap envelope publication using current body hash and resolved owner, tag derived writes to prevent scheduling loops, and notify graph refresh after success; verify concurrent newer edits cannot be overwritten.
- [x] 2.4 Handle owner rename/delete and empty Markdown transitions, removing or ignoring obsolete derived data; verify renamed, deleted, orphaned, and emptied documents never contribute stale search results.
- [x] 2.5 Add idempotent project scan/backfill behavior for the editor/server and a CLI backfill command suitable for CI and external generators; verify repeated backfill changes no already-current file.

## 3. Fast Graph and MCP Startup

- [x] 3.1 Change graph construction to build chunks and text indexes from authored bodies, validate/import matching persisted vectors, and never configure or invoke the embedding model; verify a guard/fake model fails the test if touched during graph load.
- [x] 3.2 Add text-only degraded publication for missing/stale/corrupt envelopes and atomic vector-index activation when a current artifact arrives; verify queries remain available throughout and no reader observes a mixed snapshot.
- [x] 3.3 Reuse unaffected imported vectors during document refresh and rebuild only affected index content where supported; verify incremental results are equivalent to a clean load of the same files.
- [x] 3.4 Separate and expose source revision, embedding revision, retrieval mode, and vector readiness in snapshot summaries and existing MCP responses; verify generated-header updates do not alter source/entity/chunk identity.
- [x] 3.5 Remove MCP critical-path model readiness embedding and legacy synchronous generation after equivalence coverage passes; verify cold MCP startup and graph publication meet the selected few-second budget with a large representative fixture and an intentionally unavailable embedding runtime.

## 4. Editor and Status Integration

- [x] 4.1 Update Markdown render, source-edit, assistant-context, and save paths to use the body-only projection and preserve or invalidate derived metadata correctly; verify the reserved header never appears in rendered or editable UI.
- [x] 4.2 Extend typed status data and UI states to report document persistence, embedding queued/processing/current/stale/failed, graph publication, and text-only versus hybrid retrieval independently; verify accessibility text and failure recovery tests.
- [x] 4.3 Ensure a complete Markdown file emitted by a conforming external generator is accepted without any diagram mutation; verify an integration fixture starts with hybrid retrieval and byte-identical BPMN/CMMN files.

## 5. Migration, Performance, and Documentation

- [x] 5.1 Add upgrade fixtures for headerless projects and model/chunker/version changes; verify they become text-searchable immediately and converge to current vectors after asynchronous backfill.
- [x] 5.2 Add end-to-end stress tests for rapid saves, editor and external-writer races, worker/MCP absence or restart, corrupt payloads, and derivation failure/recovery; verify newest durable authored content always wins.
- [x] 5.3 Measure cold/warm MCP startup, graph publication, derivation throughput, Markdown size growth, and text/hybrid search equivalence on representative projects; record budgets and verify regressions in automated performance coverage.
- [x] 5.4 Update README, MCP/editor documentation, generator-facing envelope schema, runtime prerequisites, `.gitignore`/diff guidance if needed, and migration instructions; verify documented examples validate with the production parser and backfill command.
