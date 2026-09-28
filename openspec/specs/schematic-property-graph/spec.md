# schematic-property-graph Specification

## Purpose
Compile saved schematic structure and owned Markdown into a deterministic, searchable project graph with stable identity and provenance.
## Requirements
### Requirement: Root-reachable schematic loading
The system SHALL load a schematic graph from the confined project `schematics/main.cmmn` anchor and SHALL recursively follow supported authored composition Names to existing CMMN and BPMN diagrams using the established Name-to-path rules. It SHALL visit each normalized diagram path at most once and SHALL NOT include unreferenced diagrams as authoritative model content.

#### Scenario: Nested compositions are reachable
- **WHEN** the root CMMN links to a BPMN process whose nodes link to further existing compositions
- **THEN** the loader includes each reachable diagram exactly once and records each authored composition relationship

#### Scenario: Composition cycle is encountered
- **WHEN** two reachable diagrams refer to one another
- **THEN** the loader terminates without duplicate diagrams and preserves both composition relationships

#### Scenario: Orphan diagram exists
- **WHEN** a valid diagram exists under `schematics/` but no reachable composition refers to it
- **THEN** the graph excludes it from authoritative model content and the load summary reports it as skipped or orphaned

### Requirement: Complete property-graph entities
The system SHALL represent every loaded diagram, diagram node, authored diagram edge, and derived Markdown chunk with a stable project-scoped SSW URN in its universal `id` property. A schematic element URN SHALL derive from project identity, its diagram's resolved semantic owner Name, and its exact unique diagram element `sourceId`; the terminal component SHALL be the diagram ID and SHALL NOT use the element's optional Name, Label, file, or folder path. Diagram URNs SHALL use the resolved package/process Name, with a reserved `root` owner for the project anchor. Chunk URNs SHALL derive from their owner URN, content hash, and ordinal. The graph SHALL preserve source kind, type, Label, resolved Name, normalized Implementation Status, and semantic ownership and SHALL represent containment, authored edge endpoints, and composition as typed relationships.

#### Scenario: Authored edge is loaded
- **WHEN** a BPMN sequence flow or supported CMMN connection has an ID, source, target, Name, Label, or Documentation
- **THEN** the graph contains an edge entity with those properties plus typed relationships to its source, target, and owning diagram

#### Scenario: Same XML ID appears in two diagrams
- **WHEN** two reachable diagrams both contain element ID `Task_1`
- **THEN** both entities have `sourceId` equal to `Task_1` and distinct universal URN IDs derived from their different semantic owner Names

#### Scenario: Unchanged entity is reloaded
- **WHEN** the same project, semantic owner Name, entity kind, and source ID are loaded again
- **THEN** the entity receives the same universal URN ID

### Requirement: Source map separated from compiled graph
The loader SHALL retain a snapshot-scoped source map from graph URNs to confined source file and XML identity for diagnostics, citations, and future source operations. File and folder paths SHALL NOT be graph identity components or semantic entity properties, and graph queries SHALL operate on URNs, Names, IDs, relationships, status, and Markdown rather than workspace layout.

#### Scenario: Entity source is requested
- **WHEN** an MCP response requires a human-readable source citation for a graph entity
- **THEN** the service resolves it through the source map without adding the file path to the compiled entity model

#### Scenario: Composition files move without semantic rename
- **WHEN** source files move while project identity, semantic owner Name, and element source IDs remain unchanged and composition resolution remains valid
- **THEN** the recompiled entities retain their URNs while source-map locations change

### Requirement: Implementation Status in graph
The loader SHALL normalize each eligible diagram element's persisted Implementation Status to exactly `new`, `modify`, `locked`, or `open`, store it as `implementationStatus`, and derive `developmentScopeEligible=true` only for `new` and `modify`. It SHALL NOT infer status from color, Markdown, proposal text, or repository state.

#### Scenario: Green and orange elements are loaded
- **WHEN** source elements persist `new` and `modify` statuses
- **THEN** their graph entities retain those exact statuses and are eligible for development scope

#### Scenario: Status is absent or invalid
- **WHEN** an eligible source element has no recognized persisted status
- **THEN** the graph normalizes it to `open` and excludes it from development scope

### Requirement: Markdown ownership and provenance
The system SHALL read `main.md` for each loaded diagram and `docs/<element-id>.md` for each loaded node or edge when present as authored logical documentation. For a loaded edge or event, it SHALL also read generated `docs/<element-id>-contract.md` when present as implementation-contract evidence, retain its plan ID and semantic version metadata, and relate it to the same owning entity without replacing or merging the authored logical Markdown. The graph SHALL associate each document and content hash with its role and owner and SHALL retain confined file provenance only in the snapshot source map.

#### Scenario: Diagram and element documentation are present
- **WHEN** a loaded diagram has `main.md` and one of its elements has an ID-bound Markdown file
- **THEN** both documents are retrievable from their respective owners with distinct source paths and hashes

#### Scenario: Logical and implementation contracts are present
- **WHEN** a loaded edge or event has both `<element-id>.md` and `<element-id>-contract.md`
- **THEN** graph queries can retrieve both separately as logical intent and actual implementation evidence for the same entity

#### Scenario: Implemented contract differs
- **WHEN** generated contract content differs from logical documentation
- **THEN** graph loading succeeds, preserves both documents, and exposes a contract-drift indicator without treating the generated file as new design authority

#### Scenario: Optional Markdown is absent
- **WHEN** a loaded entity has neither logical nor generated Markdown
- **THEN** graph loading succeeds and reports that entity without documentation chunks

#### Scenario: Generated contract targets an unsupported owner
- **WHEN** a `-contract.md` file does not resolve to a reachable edge or event
- **THEN** the loader excludes it from implementation evidence and reports a bounded diagnostic

### Requirement: Grafeo-native Markdown embeddings and indexes
The system SHALL split non-empty Markdown into deterministic bounded chunks and SHALL validate and import current persisted finite fixed-dimension vectors for those chunks into Grafeo-native vector indexes. It SHALL create Grafeo-native text indexes for all current chunks regardless of vector readiness and SHALL retain model, dimension, owner, heading, ordinal, hash, and source-path provenance with every indexed chunk. Graph construction SHALL NOT load an embedding model or generate vectors, and the system SHALL NOT maintain a parallel query-time vector store.

#### Scenario: Document exceeds one chunk
- **WHEN** normalized Markdown exceeds the configured chunk size
- **THEN** the loader creates ordered bounded chunks whose identities match the persisted artifact and whose metadata identifies their common owner and original source

#### Scenario: Current vectors are available
- **WHEN** every artifact field and chunk hash matches the current source and configured embedding profile
- **THEN** graph construction imports the vectors and enables hybrid text/vector retrieval without invoking the embedding model

#### Scenario: Vector artifact is unavailable or invalid
- **WHEN** an artifact is missing, stale, corrupt, non-finite, has the wrong dimensions, or uses an unsupported version
- **THEN** graph construction excludes those vectors, keeps text retrieval available, reports degraded vector readiness, and does not delay graph publication to regenerate them

#### Scenario: Embedding model is unavailable
- **WHEN** the configured production embedding model cannot be loaded by the asynchronous derivation worker
- **THEN** graph construction remains text-searchable without loading the model and reports vector derivation as degraded with an actionable diagnostic

### Requirement: Validated atomic snapshot construction
The system SHALL parse and validate source content, populate a replacement in-memory Grafeo graph, and complete available text and vector index readiness before publishing it. A reachable malformed diagram, unresolved explicitly authored composition target, duplicate graph identity, path escape, unsupported endpoint, text-index failure, or configured resource-limit violation SHALL fail the build with source context and SHALL NOT publish a partial snapshot. Missing or invalid persisted embeddings SHALL degrade vector readiness but SHALL NOT fail initial graph publication; a later valid artifact update SHALL atomically publish vector-capable retrieval for the affected current revision.

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
- **WHEN** a service with a valid snapshot attempts to reload invalid authoritative project content
- **THEN** the reload reports diagnostics and all readers continue to see the prior complete snapshot

#### Scenario: Embeddings are still pending
- **WHEN** authoritative diagrams and Markdown are valid but one or more current vector artifacts are not ready
- **THEN** the system atomically publishes a text-searchable snapshot within the graph startup budget and later replaces it atomically when valid current vectors become available

### Requirement: Deterministic graph snapshot identity
The system SHALL compute a deterministic source revision fingerprint from loaded authoritative source paths and content hashes and SHALL separately expose embedding readiness and artifact-set identity with entity counts, diagram counts, chunk counts, embedding model identity, load time, and non-fatal diagnostics. Adding a valid derived artifact for unchanged authoritative content SHALL NOT change source identity, but SHALL change the reported vector readiness or artifact-set identity.

#### Scenario: Unchanged project is loaded twice
- **WHEN** the same confined authoritative project content and valid embedding artifacts are loaded twice
- **THEN** both snapshots have the same source revision, artifact-set identity, deterministic entity identities, and retrieval behavior

#### Scenario: Pending artifact becomes ready
- **WHEN** an embedding artifact is published without changing its Markdown source
- **THEN** source revision remains stable while artifact-set identity and vector readiness reflect the newly available vectors

#### Scenario: Incremental and complete builds use the same content
- **WHEN** one snapshot is produced by targeted refresh and another by a clean complete load of identical durable files
- **THEN** their revisions, entities, relationships, citations, chunks, and query behavior are equivalent

### Requirement: Content-aware derived artifact reuse
The graph builder SHALL retain enough snapshot-scoped source manifest and dependency metadata to identify affected diagram and Markdown fragments. It SHALL reuse unchanged parsed fragments and finite embeddings only when their content hashes, owner identities, embedding model, and dimensions still match.

#### Scenario: One Markdown document changes
- **WHEN** an owner's Markdown content changes without changing its owner identity or graph dependencies
- **THEN** the builder regenerates that document's chunks, embeddings, and index content while reusing unaffected source fragments and embeddings

#### Scenario: Diagram changes composition dependencies
- **WHEN** a diagram edit adds, removes, or retargets a composition relationship
- **THEN** the builder invalidates the affected dependency closure or selects the complete-build fallback before publication
