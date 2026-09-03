## Why

Diagram nodes currently capture topology but cannot carry the detailed, sectioned knowledge needed to explain intent, constraints, and code-generation guidance. Adding durable Markdown context and embedding-ready section records turns the diagram into a searchable development knowledge source while preserving a direct authoring experience inside the editor.

## What Changes

- Add a right-side Markdown inspector that opens for the selected diagram node and supports editing and rendered preview through a maintained Markdown library.
- Give every supported node its own Markdown document, including stable section identities and metadata suitable for incremental embedding.
- Add a sibling `.context.sqlite` database that stores node ownership, canonical Markdown sections, embedding vectors, embedding-model metadata, content hashes, and capture revisions together.
- Extend the data-graph file format with a versioned reference to its paired SQLite database and captured revision rather than embedding vector payloads directly in topology JSON.
- Standardize the diagram extension as `.dgraph`, replacing `.dgraph.json`, and pair `name.dgraph` with `name.context.sqlite`.
- Make Capture save content and current section embeddings in one atomic SQLite transaction, without recomputing unchanged sections or reporting success when embeddings are unavailable or stale.
- Add graph-aware context lookup that combines semantic section search with node identity and graph-neighborhood traversal so a future Codex tool can retrieve relevant Markdown from the diagram.
- Define validation, migration, export, and deletion behavior for node context and its derived embeddings.

## Capabilities

### New Capabilities

- `node-markdown-context`: Per-node Markdown authoring, sectioning, atomic content-and-embedding capture in SQLite, and graph-aware context retrieval.

### Modified Capabilities

- `data-graph-diagram-editing`: Extend node selection and the persisted data-graph contract to reference a versioned context collection while preserving topology-only editing behavior.
- `diagram-file-workspace`: Discover `.dgraph` diagrams, migrate the former `.dgraph.json` naming, treat each context database as a hidden authorized companion, and coordinate capture without losing the last valid topology or context revision.

## Impact

- Affects the web editor's node-selection flow, right-side inspector layout, document model, serializers, validators, dirty-state handling, undo/redo integration, and tests.
- Adds a Markdown editor/rendering dependency and a section parsing/sanitization boundary.
- Extends the local diagram storage schema with a context-store reference, adds a sibling SQLite database with a versioned relational schema, and requires coordinated-save, migration, and compatibility handling.
- Introduces SQLite vector-blob storage, an embedding-provider abstraction, and deterministic local retrieval APIs intended for later exposure through a Codex tool; production embedding credentials and the Codex tool surface itself are not required by this change.
