## Context

The typed web server atomically writes a document and then calls `notify_documents_changed`. The private MCP control handler synchronously runs `load_schematic_graph` under a mutex and does not answer until a full replacement graph, all Markdown embeddings, and all indexes are ready. The browser persistence queue therefore treats graph-build latency as save latency, and tab close waits for that queue before disposing the modeler. A roughly 30-second rebuild becomes a roughly 30-second close.

The active `GraphSnapshot` is already behind an `Arc<RwLock<Arc<_>>>`, which provides a good atomic publication boundary. The change must preserve complete-snapshot query semantics, project confinement, deterministic revisions, query-only public MCP tools, and last-known-good behavior.

## Goals / Non-Goals

**Goals:**

- Remove graph compilation, embedding, and indexing from the durable-save and tab-close critical path.
- Tell the MCP exactly which canonical document changed and whether it was replaced, created, renamed, or deleted.
- Recompute only affected graph fragments and Markdown embeddings when the impact is safely bounded.
- Coalesce rapid saves while guaranteeing eventual processing of the newest durable version of every changed path.
- Keep every query on one complete validated revision and surface asynchronous failures to the editor.
- Retain a safe background full-rebuild path and prove incremental/full result equivalence.

**Non-Goals:**

- Exposing graph mutation or refresh as a public MCP tool.
- Making document persistence asynchronous or weakening atomic file replacement.
- Publishing partial graphs or allowing readers to observe mixed revisions.
- Incrementally updating package/process rename operations in the first implementation; these may use the full-rebuild fallback.
- Persisting the compiled graph across MCP restarts.

## Decisions

### 1. Split durable save acknowledgement from graph publication

After atomic file replacement, the web server sends a bounded private notification containing the project credentials, normalized project-relative path, change kind, and a monotonic notification ID. The MCP validates and enqueues it, then immediately returns `queued` (or `notRunning`/`rejected`). It does not await parsing, embeddings, indexes, or publication.

The save response continues to prove durable persistence. Graph state becomes a separate lifecycle with `queued`, `processing`, `updated`, `unchanged`, `failed`, and `notRunning` states. The web server exposes a read-only project-local graph-refresh status endpoint and the browser polls it only while work is pending. This preserves actionable diagnostics without holding a save request open.

Alternative considered: make no protocol changes and merely spawn the existing rebuild. That fixes close latency, but loses the changed path, prevents targeted work, and gives the editor no reliable completion or failure state.

### 2. Use one project-scoped background scheduler with path coalescing

`SchematicMcp` owns a refresh coordinator and one worker. Pending work is a map keyed by normalized relative path; a later notification replaces the earlier entry for that path. Notifications arriving during a build remain pending for the next pass. The worker drains a batch, reads current durable files, builds off-lock, validates, and swaps the active snapshot under the short write lock.

This is a latest-state queue, not an event log: intermediate auto-saves may be skipped, but once input quiesces a published revision must represent the newest accepted durable state. Serial execution avoids competing embedding/index builds and preserves deterministic publication order.

Alternative considered: one build task per save. It is simple but multiplies CPU/memory pressure and can publish stale work after newer saves.

### 3. Build from cached source fragments and use conservative invalidation

Extend snapshot build state with confined source manifests, content hashes, resolved composition dependencies, reverse dependencies, parsed diagram fragments, Markdown chunks, and embeddings. For a Markdown-only change whose owner still exists, only that owner's document fragments, hashes, chunks, embeddings, and relevant index entries are regenerated. For diagram XML, reparse that diagram and invalidate its reverse-dependent composition closure because reachability and semantic owners can change.

If the change affects root reachability, a semantic owner, a composition target, a rename/delete, an unsupported path, or dependency analysis is uncertain, the worker performs the existing complete loader in the background. Unchanged content-addressed embeddings are reused in either path. Incremental output must pass the same validation and deterministic-revision calculation as a clean full build.

Alternative considered: mutate the published Grafeo database in place. That gives narrower writes but cannot guarantee that concurrent readers see a complete single revision or that failures leave no partial mutations.

### 4. Publish only complete snapshots and verify incremental equivalence

All work produces a staging `GraphSnapshot`. Publication remains one `Arc` swap after parsing, relationship resolution, limits, embeddings, indexes, and deterministic revision validation succeed. On failure the prior snapshot stays active and refresh status records the failed notification ID, prior active revision, affected paths, and diagnostic.

Tests compare the normalized entities, relationships, citations, chunks, search behavior, and revision from an incremental build with a clean full build of the same files. A mismatch is a correctness defect; production may conservatively fall back to the full builder before publication.

### 5. Bound the editor's wait to persistence and enqueueing

The close operation still flushes pending diagram XML to durable storage before destroying the modeler. Its network wait ends after the server has atomically persisted the file and made one bounded enqueue attempt. Acceptance latency receives a regression budget independent of graph size; automated tests use an intentionally slow graph builder and assert that save/close complete without waiting for it.

## Risks / Trade-offs

- **[Incremental invalidation misses a dependency]** → Start conservatively, fall back on structural ambiguity, retain full-build equivalence tests, and never publish before common validation passes.
- **[Background work hides failures]** → Maintain queryable refresh state with notification IDs, revisions, paths, timestamps, and diagnostics; have the browser poll while pending and display failure separately from save success.
- **[Continuous edits starve publication]** → Drain finite batches and leave arrivals for the next pass; coalesce per path but do not reset an unbounded global debounce timer.
- **[Snapshot caches increase memory]** → Store immutable compact fragments and content-addressed embeddings, enforce existing resource limits, and release replaced snapshot state with its `Arc`.
- **[MCP exits after accepting work]** → Do not imply publication in the save response; after restart the initial complete load uses durable files, and the editor reports `notRunning` when status cannot be reached.
- **[Grafeo cannot cheaply patch indexes]** → Reuse parsing and embeddings first, rebuild only the staging index if required, and keep it off the interaction path; profile before adding backend-specific mutation.

## Migration Plan

1. Add the versioned private request/response and refresh-status types while retaining the current full loader.
2. Introduce the coordinator and route every save-triggered refresh through the background full builder; update the browser state model and latency tests.
3. Add source manifests, dependency metadata, and content-addressed embedding reuse.
4. Enable targeted Markdown refresh, then diagram/dependency-closure refresh, guarded by full-build equivalence tests and conservative fallback.
5. Remove the synchronous control path after all callers and tests use enqueue/status semantics.

Rollback can disable incremental selection and keep the background coordinator on the complete loader. If asynchronous behavior itself must be rolled back, the versioned private protocol permits restoring the old handler and web server together without changing public MCP tools or source formats.

## Open Questions

- Establish the concrete enqueue latency budget from CI stability; the behavioral requirement is that it remains independent of graph-build duration.
- Measure whether Grafeo index reconstruction or embedding generation dominates representative projects to prioritize native index patching versus embedding reuse.
