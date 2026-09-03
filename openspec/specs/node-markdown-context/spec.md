# node-markdown-context Specification

## Purpose
TBD - created by archiving change add-node-markdown-context. Update Purpose after archive.
## Requirements
### Requirement: Node Markdown inspector
The editor SHALL open a right-side Markdown inspector when a persisted Data Graph node is selected, SHALL identify the selected node, SHALL provide editable source and sanitized rendered-preview modes, and SHALL preserve diagram interaction in the remaining viewport. The editor SHALL use a Markdown-aware editing library, SHALL disable or sanitize executable raw content, and SHALL provide keyboard-accessible controls for editing, previewing, and closing the inspector.

#### Scenario: Node opens its Markdown
- **WHEN** the user selects an object, edge, mutation, or displayed property node
- **THEN** the right-side inspector opens with that node's Markdown source and identity without changing the graph topology

#### Scenario: Preview is rendered safely
- **WHEN** the selected node's Markdown contains formatting and executable HTML or an unsafe URL
- **THEN** the preview renders supported Markdown formatting and removes or disables executable content

#### Scenario: Selection changes with pending input
- **WHEN** the user edits Markdown and then selects another node before the debounce interval ends
- **THEN** the editor commits the first node's pending edit before displaying the second node's Markdown

#### Scenario: Inspector is operated by keyboard
- **WHEN** keyboard focus enters the Markdown inspector
- **THEN** the user can edit, switch source and preview, and close or return focus to the diagram without a pointer

### Requirement: Undoable per-node Markdown authoring
The editor SHALL maintain at most one draft context document per supported node, SHALL create it on the first non-empty edit, SHALL commit Markdown changes through the diagram command stack, and SHALL include pending edits in dirty-state and capture behavior. Undo and redo SHALL restore the draft Markdown and its corresponding derived section state without altering the last successfully captured SQLite revision.

#### Scenario: Markdown edit is committed
- **WHEN** the user changes a node's Markdown and the edit is committed by debounce, blur, selection change, or export
- **THEN** the document becomes dirty and the selected node owns the new draft Markdown awaiting capture

#### Scenario: Markdown edit is undone
- **WHEN** the user undoes and redoes a committed Markdown change
- **THEN** the Markdown and derived sections return to their prior state and then to the edited state

#### Scenario: Empty context is removed
- **WHEN** the user removes all Markdown from a node and commits the edit
- **THEN** the editor removes that node's draft context and stages removal of its captured sections for the next capture

#### Scenario: Node with context is deleted
- **WHEN** the user deletes a node that owns Markdown context
- **THEN** the editor removes the node, stages a context tombstone in the same undoable operation, and removes the captured rows when the deletion is captured

### Requirement: Deterministic Markdown section collection
The system SHALL parse canonical Markdown into an ordered collection of addressable sections using heading boundaries and a document preamble when present. Each section SHALL store a stable identifier, heading path, ordinal, section Markdown, and a normalized content hash; reparsing SHALL reuse an unambiguous prior identifier for unchanged or recognizably corresponding content and SHALL assign a new identifier when safe correspondence cannot be established.

#### Scenario: Headed document is sectioned
- **WHEN** Markdown contains a preamble and nested headings
- **THEN** the context stores ordered preamble and heading sections with heading paths, ordinals, text, and hashes matching the canonical Markdown

#### Scenario: One section changes
- **WHEN** the user edits one section without changing the other sections
- **THEN** unchanged sections retain their identifiers and hashes while the edited section receives a new hash

#### Scenario: Ambiguous section correspondence is encountered
- **WHEN** duplicated headings and reordered content cannot be matched unambiguously to a prior section
- **THEN** the system assigns a new section identifier rather than attaching prior derived data to the wrong content

### Requirement: Atomic SQLite context capture
The system SHALL support embedding generation through a provider-independent interface and SHALL capture sections to a sibling `.context.sqlite` database governed by a versioned relational schema. Every active captured section row SHALL store its Markdown content together with provider, model, dimensions, finite Float32 vector blob, source hash, node metadata, and capture revision. Capture SHALL reuse an embedding only when its source hash and model metadata match, SHALL commit a complete revision atomically, and SHALL report success only after the `.dgraph` manifest references that committed revision.

#### Scenario: Unchanged section is processed
- **WHEN** embedding refresh encounters a section with a current compatible embedding
- **THEN** capture copies or retains that content-and-embedding row without requesting regeneration

#### Scenario: Edited section is processed
- **WHEN** a section's normalized content hash differs from its stored embedding source hash
- **THEN** capture requests an embedding for only that changed section and writes its new content and vector in the new revision

#### Scenario: Provider is unavailable
- **WHEN** Capture is requested while no embedding provider is configured or a generation request fails
- **THEN** the draft remains editable, the prior SQLite revision remains authoritative, and the editor reports that content and embeddings were not captured

#### Scenario: Late embedding result arrives
- **WHEN** an embedding result returns after its section has been edited again
- **THEN** the system rejects the result whose source hash no longer matches and preserves the newer section state

#### Scenario: Dataset commits but manifest update fails
- **WHEN** SQLite commits a new complete revision but the diagram manifest cannot be atomically updated
- **THEN** the prior manifest revision remains authoritative and the system reports a recoverable unreferenced revision without claiming capture success

### Requirement: Graph-aware context retrieval
The system SHALL expose deterministic context lookup by node ID and section ID and SHALL support ranked retrieval combining lexical matching, compatible current embedding similarity, and bounded traversal of stored graph connections. Results SHALL include node and section identity, heading path, Markdown, score components, and graph distance; SHALL enforce configurable hop and result limits; and SHALL use stable tie-breaking.

#### Scenario: Context is found from a node
- **WHEN** a caller requests context for a node with a bounded graph-neighborhood depth
- **THEN** the system returns that node's matching sections and eligible neighboring-node sections with their graph distances without modifying the graph

#### Scenario: Semantic sections are ranked
- **WHEN** a query embedding is compatible with current stored section embeddings
- **THEN** the system ranks sections using their similarity together with configured lexical and graph signals and returns stable results for the same input

#### Scenario: Embeddings are absent or incompatible
- **WHEN** a query has no compatible current section embeddings
- **THEN** retrieval still returns applicable direct, lexical, and graph results and reports that semantic coverage was unavailable

#### Scenario: Traversal encounters a cycle
- **WHEN** graph-neighborhood retrieval encounters cyclic relationships or object links
- **THEN** traversal terminates within the hop limit and visits each graph element at most once per query path policy

### Requirement: SQLite context validation and limits
The system SHALL validate the relational schema and referenced SQLite revision and SHALL reject duplicate active identifiers, more than one active context for a node, missing or unsupported owners, inconsistent canonical sections, malformed hashes, malformed or non-finite Float32 vectors, vector lengths inconsistent with declared dimensions, or rows lacking either section content or a current embedding. It SHALL enforce configured Markdown, section, vector, and database limits and SHALL report actionable failures without changing the authoritative manifest revision.

#### Scenario: Invalid context owner is imported
- **WHEN** a context references a node that is absent or unsupported
- **THEN** import fails with an actionable context-owner error and the source file remains unchanged

#### Scenario: Invalid embedding is imported
- **WHEN** an embedding contains a non-finite value or a vector length different from its declared dimensions
- **THEN** import fails with an actionable embedding validation error and the source file remains unchanged

#### Scenario: Context exceeds a configured limit
- **WHEN** imported or captured context exceeds a configured Markdown, section, vector, or database limit
- **THEN** the system rejects the operation, preserves the last valid state, and identifies the exceeded limit

