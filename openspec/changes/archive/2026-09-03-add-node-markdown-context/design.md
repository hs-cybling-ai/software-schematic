## Context

The Data Graph editor is a vanilla JavaScript application built on `diagram-js`. Its versioned JSON document currently normalizes topology into `nodes`, `edges`, and `properties`, and its command stack provides dirty-state and undo/redo signaling. The graph has no durable prose layer, no right-side inspector, and no retrieval contract that an external development tool can consume.

This change crosses the editor UI, command model, serializer, document validation, derived-data lifecycle, and search surface. Markdown is authored content and therefore the source of truth; parsed sections and embeddings are derived indexes that must never prevent a user from recovering or editing that source.

## Goals / Non-Goals

**Goals:**

- Let users select any persisted Data Graph node and edit or preview its Markdown in a right-side inspector.
- Persist captured context and embeddings in a sibling `.context.sqlite` database with a documented relational schema, referenced by the Data Graph file.
- Derive addressable Markdown sections and retain embeddings by section with enough provenance to detect stale vectors.
- Avoid embedding unchanged sections and make partial or failed embedding refresh explicit and recoverable.
- Provide deterministic node, section, semantic, and graph-neighborhood lookup primitives suitable for a future Codex tool.
- Preserve safe coordinated save, undo/redo, deletion, and compatibility behavior.

**Non-Goals:**

- Registering or distributing the Codex tool itself.
- Bundling credentials or requiring a network embedding service to open, edit, or save a diagram.
- Supporting Markdown on BPMN elements, free-floating edge labels, topology edges, or property records in this change.
- Treating rendered HTML or embeddings as authoritative content.
- Building a general-purpose vector database outside the diagram file.

## Decisions

### Store context and vectors in a sibling SQLite database

Version 2 uses the `.dgraph` extension, retains normalized `nodes`, `edges`, and `properties`, and adds a `contextStore` manifest containing a workspace-relative SQLite file URI, schema version, stable diagram ID, and last captured revision. The sibling for `name.dgraph` is `name.context.sqlite`. Its versioned `metadata`, `capture_revisions`, and `context_sections` tables hold one active row per section per revision. Section columns include `diagram_id`, `context_id`, `node_id`, `node_type`, `node_label`, `section_id`, `heading_path_json`, `ordinal`, `markdown`, `content_hash`, `embedding_blob`, `embedding_provider`, `embedding_model`, `embedding_dimensions`, `captured_at`, `capture_revision`, and `is_tombstone`.

SQLite is selected over embedding vectors directly in JSON because it is built into macOS, provides atomic transactions and schema versioning, stores text, metadata, and packed Float32 vectors together, and gives a future Codex tool a universally readable local file to query. Lance was considered but lacks a production-supported Swift SDK; Arrow IPC lacks transactional updates and indexing. The topology file remains reviewable and small, while a workspace-relative manifest keeps the pair discoverable and movable together.

### Keep editor drafts separate from captured context

On an edit command, the sectioner parses Markdown headings and creates normalized draft sections. It reuses a prior section ID first by exact content hash and then by unique heading-path/ordinal match; new or ambiguous content receives a new ID. A normalized SHA-256 hash covers the text used for embedding. Drafts participate in undo/redo and dirty state but do not replace the last captured SQLite revision.

The explicit Capture action generates embeddings for every changed section, reuses unchanged compatible rows, and commits the complete active section set in one SQLite transaction. Capture succeeds only when each non-empty active section has content and a compatible embedding in the committed revision. Provider failure leaves the draft and prior captured revision intact and visibly reports that capture did not complete.

### Use CodeMirror 6 for editing and markdown-it plus DOMPurify for preview

CodeMirror 6 fits the existing framework-free application, supports Markdown syntax and accessible editing, and does not require adopting React. `markdown-it` provides deterministic Markdown rendering; DOMPurify sanitizes the rendered result before insertion. Raw HTML is disabled in the renderer as defense in depth. A heavier WYSIWYG framework was rejected because canonical Markdown and predictable section boundaries are more important than rich-text abstraction.

### Integrate edits with the diagram command stack

The inspector buffers keystrokes locally and commits a debounced `context.update` command, with an immediate flush on selection change, save/export, or focus loss. Each committed change contains before/after context state so undo and redo restore Markdown and derived section metadata together. Embedding refresh results use a separate guarded command that applies only if the section source hash still matches, preventing late asynchronous results from overwriting newer edits.

### Separate embedding production from storage and search

An `EmbeddingProvider` interface accepts normalized section text and returns provider/model metadata plus fixed-dimension numeric vectors. The editor can draft with no provider, but Capture is unavailable and the prior database revision remains authoritative. Provider-specific credentials and network calls stay outside document parsing.

This boundary allows a host process or later Codex integration to supply embeddings without coupling the portable file format to one vendor. The initial implementation includes a deterministic fake provider for tests and import/export fixtures, not a production credential flow.

### Combine retrieval signals deterministically

The context index supports direct lookup by node or section ID, lexical matching over labels/headings/text, cosine similarity over compatible current embeddings, and bounded graph expansion across stored topology and object links. Results expose node/section IDs, heading path, Markdown, score components, and graph distance. Ranking uses explicit weights, stable tie-breaking by node then section ID, a configurable result limit, and a bounded hop count.

The retrieval API reads stored topology only and does not materialize inferred connections. Direct lookup works without embeddings; semantic search reports incompatible or absent embedding coverage rather than silently treating vectors from different models as comparable.

### Coordinate topology and SQLite revisions safely

The current writer emits version 2. Import accepts the immediately preceding valid topology-only document and upgrades it in memory with a default sibling database reference without marking the tab dirty; older obsolete development shapes remain rejected. The host validates that resolved database paths remain inside the authorized workspace. SQLite schema and row validation rejects duplicate active context/section IDs, multiple active contexts for one node, missing or unsupported node owners, malformed or non-finite vectors, inconsistent dimensions, invalid hashes, and incomplete captured revisions.

Capture first commits a new SQLite revision in one transaction, then atomically updates the topology manifest to that revision. Until the manifest update succeeds, the prior revision remains authoritative; an unreferenced newer revision can be retried or compacted. Deleting a node stages a context tombstone and becomes authoritative on the next successful capture. This is a small two-phase protocol because filesystem replacement cannot atomically cover both the JSON and SQLite files.

## Risks / Trade-offs

- [SQLite history can grow through captured revisions] → Use a predictable sibling name, enforce limits, and compact revisions older than the authoritative recovery window.
- [Section identities can shift after substantial restructuring] → Reuse IDs conservatively and treat IDs as durable retrieval handles, not semantic truth; unmatched sections receive new IDs.
- [Debounced edits can be lost during navigation or export] → Flush synchronously on selection changes, blur, export, and teardown, and test those boundaries.
- [Rendered Markdown can introduce unsafe HTML or links] → Disable raw HTML, sanitize all generated HTML, and add malicious-input tests.
- [Capture can fail after embedding or database commit] → Keep the prior manifest revision authoritative and retry or clean up unreferenced SQLite revisions.
- [Graph expansion can produce noisy context] → Bound traversal depth and result count, expose score components, and use deterministic weighting/tie-breaking.

## Migration Plan

1. Define the SQLite schema, store interface, context-store manifest, and fixture tooling.
2. Add `.dgraph` discovery/naming, version-2 parsing, manifest validation, authorized path resolution, and in-memory upgrade of valid version-1 topology-only `.dgraph.json` documents.
3. Add the draft context model, sectioner, command handlers, deletion staging, and tests before exposing the inspector.
4. Add the Markdown inspector and sanitized preview, then wire selection and draft flushing.
5. Add embedding generation and the atomic Capture protocol, followed by SQLite-backed graph-aware retrieval.
6. Update fixtures and compatibility tests so capture produces a referenced SQLite revision containing both content and embeddings.

Rollback consists of reverting the application release; the SQLite database remains independently readable with standard SQLite tooling, but a prior editor requires a topology-only copy because it intentionally rejects the version-2 manifest rather than discarding context.

## Open Questions

- Which production embedding provider and model should the host or Codex tool use, and where should credentials live?
- What default section-size, vector-dimension, document-size, retrieval-weight, and graph-hop limits best fit real project diagrams?
- What bridge operations should the native SQLite context store expose for draft loading, capture, and search?
