## MODIFIED Requirements

### Requirement: Validated atomic snapshot construction
The system SHALL construct a complete candidate snapshot either from a clean project load or from a prior snapshot plus safely invalidated source fragments. It SHALL parse and validate affected source content, populate a staging Grafeo graph, and complete required embedding and index readiness before publication. A reachable malformed diagram, unresolved explicitly authored composition target, duplicate graph identity, path escape, unsupported endpoint, embedding/index failure, configured resource-limit violation, or uncertain incremental dependency impact SHALL prevent incremental publication; the system SHALL either use a complete-build fallback or retain the prior snapshot with a diagnostic.

#### Scenario: Reachable diagram is malformed
- **WHEN** a referenced diagram cannot be parsed and no prior snapshot exists
- **THEN** loading fails with its relative path and parse cause and no graph is exposed

#### Scenario: Incremental refresh succeeds
- **WHEN** the changed paths and their dependency closure produce a valid complete candidate snapshot
- **THEN** the system atomically publishes that snapshot and no reader observes a mixture of old and new graph content

#### Scenario: Incremental impact is uncertain
- **WHEN** the loader cannot prove that its invalidation set covers every entity, relationship, source-map entry, chunk, embedding, and index entry affected by a change
- **THEN** it uses the complete loader rather than publishing the incremental candidate

#### Scenario: Reload fails
- **WHEN** a service with a valid snapshot attempts to refresh invalid project content
- **THEN** the refresh reports diagnostics and all readers continue to see the prior complete snapshot

### Requirement: Deterministic graph snapshot identity
The system SHALL compute a deterministic revision fingerprint from the complete loaded source paths and content hashes and SHALL expose it with entity counts, diagram counts, chunk counts, embedding model identity, load time, and non-fatal diagnostics. A targeted refresh and a clean complete load of identical durable project content SHALL produce the same revision fingerprint and semantically equivalent graph results.

#### Scenario: Unchanged project is loaded twice
- **WHEN** the same confined project content and embedding model are loaded twice
- **THEN** both snapshots have the same revision fingerprint and deterministic entity identities

#### Scenario: Incremental and complete builds use the same content
- **WHEN** one snapshot is produced by targeted refresh and another by a clean complete load of identical durable files
- **THEN** their revisions, entities, relationships, citations, chunks, and query behavior are equivalent

## ADDED Requirements

### Requirement: Content-aware derived artifact reuse
The graph builder SHALL retain enough snapshot-scoped source manifest and dependency metadata to identify affected diagram and Markdown fragments. It SHALL reuse unchanged parsed fragments and finite embeddings only when their content hashes, owner identities, embedding model, and dimensions still match.

#### Scenario: One Markdown document changes
- **WHEN** an owner's Markdown content changes without changing its owner identity or graph dependencies
- **THEN** the builder regenerates that document's chunks, embeddings, and index content while reusing unaffected source fragments and embeddings

#### Scenario: Diagram changes composition dependencies
- **WHEN** a diagram edit adds, removes, or retargets a composition relationship
- **THEN** the builder invalidates the affected dependency closure or selects the complete-build fallback before publication

