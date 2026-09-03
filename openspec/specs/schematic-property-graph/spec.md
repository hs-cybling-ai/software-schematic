# schematic-property-graph Specification

## Purpose
TBD - created by archiving change add-schematic-graph-mcp. Update Purpose after archive.
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
The system SHALL read `main.md` for each loaded diagram and `docs/<element-id>.md` for each loaded node or edge when present. It SHALL associate authored Markdown and its content hash with the owning compiled entity and SHALL retain confined file provenance only in the snapshot source map.

#### Scenario: Diagram and element documentation are present
- **WHEN** a loaded diagram has `main.md` and one of its elements has an ID-bound Markdown file
- **THEN** both documents are retrievable from their respective owners with distinct source paths and hashes

#### Scenario: Optional Markdown is absent
- **WHEN** a loaded entity has no corresponding Markdown file
- **THEN** graph loading succeeds and reports that entity without documentation chunks

### Requirement: Grafeo-native Markdown embeddings and indexes
The system SHALL split non-empty Markdown into deterministic bounded chunks and SHALL use Grafeo's native local embedding capability to generate and store a finite fixed-dimension vector for each chunk. It SHALL create Grafeo-native vector and text indexes for retrieval and SHALL retain model, dimension, owner, heading, ordinal, hash, and source-path provenance with every indexed chunk. The system SHALL NOT maintain a parallel vector store or independent embedding pipeline.

#### Scenario: Document exceeds one chunk
- **WHEN** normalized Markdown exceeds the configured chunk size
- **THEN** the loader creates ordered bounded chunks whose metadata identifies their common owner and original source

#### Scenario: Embedding model is unavailable
- **WHEN** the configured production embedding model cannot be loaded or returns invalid vectors
- **THEN** initial graph loading fails before a graph snapshot is published and reports an actionable embedding diagnostic

### Requirement: Validated atomic snapshot construction
The system SHALL parse and validate source content, populate a replacement in-memory Grafeo graph, and complete Grafeo embedding and index readiness before publishing it. A reachable malformed diagram, unresolved explicitly authored composition target, duplicate graph identity, path escape, unsupported endpoint, embedding/index failure, or configured resource-limit violation SHALL fail the build with source context and SHALL NOT publish a partial snapshot.

#### Scenario: Reachable diagram is malformed
- **WHEN** a referenced diagram cannot be parsed and no prior snapshot exists
- **THEN** loading fails with its relative path and parse cause and no graph is exposed

#### Scenario: Reload fails
- **WHEN** a service with a valid snapshot attempts to reload invalid project content
- **THEN** the reload reports diagnostics and all readers continue to see the prior complete snapshot

### Requirement: Deterministic graph snapshot identity
The system SHALL compute a deterministic revision fingerprint from the loaded source paths and content hashes and SHALL expose it with entity counts, diagram counts, chunk counts, embedding model identity, load time, and non-fatal diagnostics.

#### Scenario: Unchanged project is loaded twice
- **WHEN** the same confined project content and embedding model are loaded twice
- **THEN** both snapshots have the same revision fingerprint and deterministic entity identities

