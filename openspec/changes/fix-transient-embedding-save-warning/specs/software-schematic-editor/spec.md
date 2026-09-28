## MODIFIED Requirements

### Requirement: Document and graph save status
The web application SHALL distinguish durable document persistence, asynchronous embedding derivation, and derived graph refresh. After a successful diagram or connected Markdown save, it SHALL report whether relevant embeddings are current, queued, processing, stale, or failed and whether graph refresh is queued, processing, published, failed, or unavailable because the MCP is not running. Expected missing or stale embedding metadata while current derivation is queued or processing SHALL be presented as normal background progress and SHALL NOT produce a warning or error toast. Embedding derivation failure or graph failure SHALL remain actionable, SHALL NOT be represented as a document-save failure, and SHALL NOT roll back authored files.

#### Scenario: Markdown save queues derivation
- **WHEN** connected Markdown saves and its content requires a new embedding artifact
- **THEN** the application reports the document as saved and embedding derivation as queued without waiting for vector generation

#### Scenario: Graph is usable while vectors are pending
- **WHEN** the current source revision is published while embedding derivation remains queued or processing
- **THEN** the application reports graph text retrieval as ready and vector retrieval as pending or degraded

#### Scenario: Pending header does not produce a false warning
- **WHEN** the text-ready refresh for newly saved Markdown reports that its current embedding header is not yet available and derivation is queued or processing
- **THEN** the application continues to show background embedding progress without displaying a graph-warning or error toast

#### Scenario: Save and graph refresh succeed
- **WHEN** current embedding artifacts are published and the running MCP imports them
- **THEN** the application reports current vector readiness and the updated graph artifact identity

#### Scenario: Save succeeds without MCP
- **WHEN** the document saves but no matching MCP process is running
- **THEN** the document remains saved, embedding derivation may continue independently, and the application reports that the graph was not refreshed

#### Scenario: Embedding derivation fails
- **WHEN** Markdown saves but asynchronous embedding generation fails
- **THEN** the application reports the document as saved, vector retrieval as degraded, and an actionable derivation diagnostic

#### Scenario: Save succeeds but graph rebuild fails
- **WHEN** the graph retains its prior source revision after a refresh failure
- **THEN** the application reports the document as saved, graph refresh as failed, and an actionable diagnostic
