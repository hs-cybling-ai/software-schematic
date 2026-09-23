## Purpose

Precompute and persist trustworthy Markdown vectors beside their schematic owners so graph startup can reuse derived search data without delaying availability.

## ADDED Requirements

### Requirement: Versioned owner-bound embedding artifacts
For every non-empty connected Markdown document, the system SHALL accept and persist a canonical reserved SSW embedding header in that Markdown file. The header SHALL contain the diagram-root, node, or edge owner identity, authored-body content hash, deterministic chunk identities and hashes, chunking-format version, embedding model identity and version, vector dimensions, and finite vectors. The Markdown body SHALL remain the authored source of truth, and derived embedding metadata SHALL NOT change graph identity or implementation authority.

#### Scenario: Node documentation is embedded
- **WHEN** `docs/<element-id>.md` is durably saved for a current node or edge
- **THEN** its validated embedding header identifies that exact diagram element and records the source and derivation identities needed to detect staleness

#### Scenario: Diagram documentation is embedded
- **WHEN** a diagram's `main.md` is durably saved
- **THEN** its embedding header identifies that diagram's root owner rather than an arbitrary child element

#### Scenario: External generator emits a complete Markdown file
- **WHEN** a conforming tool writes an authored Markdown body plus a valid canonical SSW embedding header
- **THEN** graph construction can consume the vectors without that tool or the system modifying diagram XML

#### Scenario: User views or edits Markdown
- **WHEN** the editor renders a connected Markdown document or shows its source-editing surface
- **THEN** the reserved SSW header is omitted and only the authored Markdown body is visible and editable

### Requirement: Asynchronous latest-content derivation
The system SHALL schedule embedding derivation after durable Markdown persistence, SHALL coalesce pending work by canonical document owner, and SHALL publish a result only when its source hash still matches the newest durable Markdown. Authored Markdown persistence and editor interaction SHALL NOT wait for model loading, chunk embedding, or artifact publication.

#### Scenario: Markdown changes during derivation
- **WHEN** a newer durable version of the same document appears while an older version is being embedded
- **THEN** the older result is discarded and the worker eventually derives and publishes an artifact for the newest durable content

#### Scenario: Derivation fails
- **WHEN** model loading or vector generation fails after Markdown is saved
- **THEN** the Markdown remains saved, the prior valid artifact is not represented as current, and status reports an actionable derivation failure

### Requirement: Conflict-safe derived metadata persistence
Embedding header publication SHALL preserve the exact current authored Markdown body, SHALL be atomic, and SHALL not overwrite a newer body revision. Rename and delete operations SHALL move or remove headers with their Markdown files consistently, and headers whose owners no longer exist SHALL NOT be treated as current.

#### Scenario: Markdown changes while an embedding is computed
- **WHEN** the authored Markdown body is saved after derivation begins but before its header is published
- **THEN** publication proceeds only if the current body hash and owner still match, without reverting the newer authored change

#### Scenario: Document owner is deleted
- **WHEN** an element and its connected Markdown are removed
- **THEN** its embedding artifact is removed or ignored and cannot appear in search results

### Requirement: Backfill and compatibility
The system SHALL accept initialized projects that do not yet contain embedding artifacts, SHALL provide an idempotent project-scoped backfill path, and SHALL replace artifacts whose content, chunking, model, dimension, or format identity is stale. Unknown future artifact versions SHALL be ignored with a diagnostic rather than preventing authored diagrams from loading.

#### Scenario: Existing project has no artifacts
- **WHEN** an existing initialized project is opened or its MCP starts after upgrade
- **THEN** graph access becomes available without synchronous embedding and missing artifacts are queued for asynchronous backfill

#### Scenario: Embedding model changes
- **WHEN** persisted artifacts were produced by a different configured model identity or dimension
- **THEN** they are excluded from vector retrieval and queued for regeneration while authored content and text retrieval remain available
