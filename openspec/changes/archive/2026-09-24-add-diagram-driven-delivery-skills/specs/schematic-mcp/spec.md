## ADDED Requirements

### Requirement: Revision-bound document synchronization
The MCP service SHALL expose a bounded graph-synchronization capability for authenticated project-local skill clients that accepts canonical project-relative document identities, strong content revisions, change kinds, and an optional previously observed source-manifest revision. It SHALL read authoritative durable content itself, reject documents outside the project, coalesce duplicate revisions, and SHALL NOT accept caller-supplied graph data, arbitrary paths, or graph mutation commands.

#### Scenario: Skill synchronizes changed documents
- **WHEN** the graph skill submits two valid changed-document identities and their durable revisions
- **THEN** the service validates the project and revisions, queues only those changes, and returns a synchronization operation identity

#### Scenario: Caller submits stale or invented content
- **WHEN** a submitted revision does not match the current durable document or a path is not a canonical project document
- **THEN** the service rejects that item without publishing caller-controlled or partial graph state

### Requirement: Source manifest and publication receipts
Project overview and synchronization status SHALL expose a bounded source manifest of canonical documents and strong content revisions, plus publication receipts that correlate an accepted change set to a complete active graph revision. A successful receipt SHALL identify whether processing was incremental, unchanged, or a full-rebuild fallback and SHALL be returned only after atomic publication.

#### Scenario: Incremental publication completes
- **WHEN** all accepted changes are incorporated into a validated snapshot
- **THEN** status returns a receipt containing the operation identity, included source-manifest revision, active graph revision, and incremental outcome

#### Scenario: Publication fails
- **WHEN** parsing, validation, indexing, or embedding work fails
- **THEN** status reports the diagnostic and prior active revision and issues no successful receipt for the rejected source manifest

### Requirement: Revision-bound implementation context
The MCP service SHALL allow a client to request bounded implementation context for an explicitly resolved authorized entity set and graph revision. The response SHALL include relevant annotations, relationships, citations, status, and source-manifest revision and SHALL reject a request whose graph revision is no longer available or whose entities are not eligible implementation targets.

#### Scenario: Build requests current context
- **WHEN** the build skill supplies a current graph revision and eligible resolved entities
- **THEN** the service returns a deterministic bounded implementation context for that exact revision

#### Scenario: Requested revision is obsolete
- **WHEN** the build skill supplies a graph revision that can no longer validate the entities' contracts
- **THEN** the service returns a revalidation-required result instead of silently substituting current context

