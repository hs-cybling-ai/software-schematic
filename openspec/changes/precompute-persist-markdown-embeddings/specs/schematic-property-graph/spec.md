## MODIFIED Requirements

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
