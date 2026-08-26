## 1. SQLite Storage Foundation

- [x] 1.1 Integrate macOS SQLite through a Swift system-library target, document the relational schema and SQLite dependency, and add a smoke test that creates, reopens, and queries a temporary database.
- [x] 1.2 Implement the versioned SQLite row model for diagram, node, section, Markdown, embedding provenance/Float32 blob, capture revision, and tombstone fields with strict type and size validation.
- [x] 1.3 Implement the workspace-scoped SQLite context store for opening a referenced revision, committing a complete revision transactionally, detecting unreferenced revisions, and compacting recoverable history.
- [x] 1.4 Add deterministic SQLite fixtures containing multiple nodes, sections, embeddings, and a tombstone, plus invalid-schema and incomplete-revision fixtures.

## 2. Data Graph Manifest and Migration

- [x] 2.1 Upgrade the Data Graph document model to version 2 with a validated `contextStore` manifest containing stable diagram ID, workspace-relative `.context.sqlite` URI, schema version, and authoritative SQLite revision.
- [x] 2.2 Implement in-memory migration from valid topology-only version-1 documents and the former `.dgraph.json` naming to `.dgraph` with a default sibling `<diagram-name>.context.sqlite` reference without marking the tab dirty.
- [x] 2.3 Resolve context database paths through the authorized workspace service and reject absolute, symlink-escaping, missing-revision, or mismatched-diagram references.
- [x] 2.4 Update deterministic serialization, starter/valid/invalid fixtures, compatibility documentation, and JS/Swift tests for version-2 manifests while keeping vector payloads out of topology JSON.

## 3. Markdown Draft and Section Model

- [x] 3.1 Implement canonical Markdown normalization, heading-based section parsing including preambles, SHA-256 content hashing, and conservative stable section-ID reconciliation.
- [x] 3.2 Add per-node draft context state and command-stack handlers so debounced edits, empty-context removal, undo, and redo restore Markdown and derived sections together.
- [x] 3.3 Integrate node deletion and restoration with staged context tombstones so capture removes deleted-node rows and undo restores the draft before capture.
- [x] 3.4 Add unit tests for nested headings, duplicate/reordered headings, unchanged-section reuse, edited hashes, empty context, node deletion, and command-stack behavior.

## 4. Markdown Inspector Experience

- [x] 4.1 Add pinned CodeMirror 6, Markdown language, markdown-it, and DOMPurify dependencies and configure raw-HTML-disabled, sanitized preview rendering.
- [x] 4.2 Build the responsive right-side inspector with selected-node identity, source/preview controls, Capture status/action, accessible labels, keyboard navigation, and focus return to the diagram.
- [x] 4.3 Connect Data Graph selection changes to the inspector and synchronously flush pending draft input on selection change, blur, capture, export, and teardown.
- [x] 4.4 Add DOM and interaction tests for supported node types, viewport resizing, node switching, debounce boundaries, keyboard operation, and malicious Markdown/link sanitization.

## 5. Embedding and Atomic Capture

- [x] 5.1 Define the embedding-provider interface and implement deterministic test and unavailable-provider adapters with provider, model, dimension, and source-hash provenance.
- [x] 5.2 Implement capture planning that reuses unchanged compatible embeddings, requests embeddings only for changed sections, rejects late hash-mismatched results, and requires content plus a current vector for every active non-empty section.
- [x] 5.3 Extend the native/web bridge with correlated draft-load, capture, progress, success, and failure messages without exposing provider credentials to document parsing or rendered content.
- [x] 5.4 Implement the two-phase capture protocol: commit a complete SQLite revision, atomically replace the diagram manifest reference, retain the prior revision on failure, and surface retry/cleanup for unreferenced revisions.
- [x] 5.5 Add integration tests for successful capture, unchanged embedding reuse, provider failure, partial batch failure, concurrent edit/late result, SQLite commit failure, manifest-write failure, and external diagram/database conflicts.

## 6. Graph-Aware Context Retrieval

- [x] 6.1 Implement direct SQLite lookup by diagram, node, context, and section IDs and return Markdown with identity, heading path, revision, and embedding provenance.
- [x] 6.2 Implement compatible-vector similarity search plus lexical matching with explicit score components, configurable result limits, and stable node/section tie-breaking.
- [x] 6.3 Implement bounded, cycle-safe graph-neighborhood expansion across stored topology and object links without materializing inferred graph content.
- [x] 6.4 Define and document the host-facing retrieval contract intended for a later Codex tool, including query inputs, results, semantic-coverage reporting, limits, and actionable errors.
- [x] 6.5 Add retrieval tests for exact IDs, semantic ranking, incompatible/missing embeddings, lexical fallback, graph distance, cycles, limits, deterministic ordering, and deleted contexts.

## 7. Verification and Delivery

- [x] 7.1 Run web lint, unit tests, compatibility tests, and production asset build; copy verified assets into the native application bundle.
- [x] 7.2 Run Swift unit/integration tests and add a UI workflow covering node selection, Markdown editing/preview, successful Capture, reopen, and restored content.
- [x] 7.3 Verify generated context data with independent `sqlite3` inspection to confirm each active captured row contains section Markdown and a finite Float32 embedding of the declared dimension.
- [x] 7.4 Update the README and compatibility guide with logical file layout, Capture semantics, backup/move guidance, dataset limits, failure recovery, and the future Codex retrieval boundary.
