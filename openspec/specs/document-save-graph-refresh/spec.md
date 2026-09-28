# document-save-graph-refresh Specification

## Purpose
Coordinate durable browser and approved-agent document saves with non-blocking, atomic publication of the current project graph.
## Requirements
### Requirement: Successful web save refreshes the running graph
After the Software Schematic daemon successfully persists complete CMMN XML, BPMN XML, or connected Markdown from the browser or an approved agent proposal, it SHALL enqueue that exact canonical durable document in the daemon's project-scoped refresh coordinator. Refresh correctness SHALL NOT depend on a separate MCP process being alive.

#### Scenario: Human saves a corrected diagram
- **WHEN** the browser successfully persists dirty diagram XML containing an evaluated contract correction
- **THEN** the daemon queues that document and subsequent UI and MCP queries observe its published graph revision

#### Scenario: Approved agent proposal saves connected Markdown
- **WHEN** the browser applies an approved agent proposal and conditionally persists connected Markdown
- **THEN** the same coordinator rebuilds the affected chunks, embeddings, indexes, and owning graph entity from durable content

#### Scenario: Human saves connected Markdown
- **WHEN** the browser successfully persists diagram-level or element-ID-bound Markdown
- **THEN** the daemon rebuilds its document chunks, embeddings, indexes, and owning graph entity from current durable content

#### Scenario: Document save fails
- **WHEN** complete conditional document replacement fails
- **THEN** the system reports the save or conflict and does not enqueue rejected content

### Requirement: Complete atomic replacement
Each document-change notification SHALL cause or coalesce into a validated graph refresh for the newest durable authoritative content. The system SHALL atomically publish a complete text-searchable snapshot and SHALL retain the last valid snapshot when parsing, validation, text indexing, or another authoritative build step fails. Embedding artifact updates SHALL trigger or coalesce into a subsequent atomic vector-index refresh only when the artifact still matches that authoritative content.

#### Scenario: Saved documents produce a valid graph
- **WHEN** the notified authoritative document set compiles successfully
- **THEN** all MCP readers switch from the prior complete revision to the replacement complete text-searchable revision without waiting for pending embeddings

#### Scenario: Structural impact is uncertain
- **WHEN** a change can alter root reachability, semantic identity, composition dependencies, rename or deletion effects, or otherwise cannot be safely bounded
- **THEN** the worker runs a complete rebuild in the background before publishing

#### Scenario: Saved documents are temporarily invalid
- **WHEN** the notified authoritative document set cannot produce a complete valid graph
- **THEN** MCP queries continue using the prior revision and the web application receives an actionable graph-refresh diagnostic

#### Scenario: Saves occur during a rebuild
- **WHEN** one or more successful saves arrive while embedding derivation or graph refresh is active
- **THEN** work is coalesced by owner and at least one subsequent serialized publication represents the newest durable documents and only matching embedding artifacts

#### Scenario: Matching artifact becomes ready
- **WHEN** asynchronous derivation publishes a valid artifact for the current Markdown hash
- **THEN** the MCP atomically enables its vectors without rebuilding unrelated embeddings

### Requirement: Private one-way project notification
Graph-refresh ingestion SHALL remain a private daemon operation scoped to one canonical project and SHALL accept only durable, revision-validated document identities from successful typed persistence or constrained synchronization requests. It SHALL NOT accept caller-supplied graph content, arbitrary graph commands, public network requests, or direct graph mutation.

#### Scenario: Project identity does not match
- **WHEN** a synchronization request does not authenticate as the same canonical project as the daemon
- **THEN** the daemon rejects it without rebuilding or changing the published revision

#### Scenario: Changed path escapes the project
- **WHEN** a notification names an absolute, parent-traversing, symlink-escaping, or unsupported document path
- **THEN** the MCP rejects it without reading the path or changing refresh state

#### Scenario: MCP is not running
- **WHEN** a web document save succeeds while no MCP proxy process is running
- **THEN** the daemon still refreshes the graph and retains the resulting receipt for a later client

#### Scenario: No MCP client is connected
- **WHEN** a web document save succeeds while no MCP proxy is connected
- **THEN** the daemon still refreshes the graph and retains the resulting receipt for a later client

### Requirement: Observable asynchronous refresh lifecycle
The system SHALL maintain project-scoped refresh state containing the latest notification ID, lifecycle state, affected paths, prior and active revisions, timestamps, and any diagnostic. The typed web server SHALL provide this state to its own browser client without exposing a graph mutation operation.

#### Scenario: Accepted refresh is still running
- **WHEN** the browser requests refresh status after an accepted notification has been queued or started
- **THEN** it receives `queued` or `processing` state while MCP queries continue against the prior complete revision

#### Scenario: Background refresh completes
- **WHEN** the worker publishes or determines no change to a complete candidate
- **THEN** refresh status becomes `updated` or `unchanged` and identifies the active revision

#### Scenario: Background refresh fails
- **WHEN** the worker cannot build a valid candidate
- **THEN** refresh status becomes `failed`, includes an actionable diagnostic, and identifies the retained active revision

### Requirement: Non-blocking refresh scheduling
Graph-build duration SHALL NOT be included in the latency of a successful document save or permitted diagram-tab close. The enqueue acknowledgement SHALL be bounded independently of project graph size and refresh work SHALL execute outside the request's interaction-critical path.

#### Scenario: Graph construction is slow
- **WHEN** a test graph builder is held for longer than the save and close latency budget
- **THEN** the document save and permitted tab close complete after durable persistence and enqueue acknowledgement while refresh remains processing

### Requirement: Query-only LLM graph access
The MCP surface SHALL permit authenticated project-local clients to request synchronization only for canonical durable diagram or Markdown identities and current strong revisions, inspect bounded operation status, and receive publication receipts. It SHALL keep authoritative file reading, dependency selection, graph compilation, validation, and publication inside the daemon.

#### Scenario: Skill submits one changed document
- **WHEN** the graph skill submits one canonical Markdown identity with its current durable revision
- **THEN** the daemon validates it, queues it through the shared coordinator, and returns a correlated operation identity

#### Scenario: Skill supplies content or a stale revision
- **WHEN** the request includes caller-controlled graph content or a revision that differs from the durable document
- **THEN** the daemon rejects it and preserves the last-known-good graph

#### Scenario: Codex discovers project MCP tools
- **WHEN** an LLM client completes MCP tool discovery
- **THEN** every advertised Software Schematic tool is bounded to query, typed proposal workflow, durable-document synchronization, receipt, or revision-bound implementation context and no arbitrary mutation tool is present

#### Scenario: Codex finds a missing contract detail
- **WHEN** graph context is incomplete or incorrect for implementation
- **THEN** managed guidance directs Codex to use the design interview and approval-gated proposal workflow rather than changing source contracts directly

### Requirement: Content-addressed incremental refresh
Graph refresh SHALL compare strong durable content revisions against the active source manifest and rebuild only affected parsed fragments, annotations, embeddings, relationships, and indexes when the dependency impact is safely bounded. Modification times SHALL NOT determine correctness. Unsupported changes, rename or delete ambiguity, reachability changes, or failed incremental validation SHALL use the complete rebuild path before publication.

#### Scenario: Markdown body changes for one owner
- **WHEN** one connected Markdown document has a new content revision and its ownership remains valid
- **THEN** refresh replaces that owner's annotation fragments and dependent search entries while reusing unaffected graph material

#### Scenario: Timestamp changes without content change
- **WHEN** a document modification time changes but its strong content revision is unchanged
- **THEN** refresh treats the document as unchanged and does not rebuild its graph fragment

#### Scenario: Composition reachability changes
- **WHEN** a diagram edit changes composition reachability and bounded dependency invalidation cannot be proven complete
- **THEN** refresh uses the complete builder and atomically publishes only the validated result

### Requirement: Unified change ingestion
Durable changes produced by the web editor or an approved agent proposal SHALL enter the same project-scoped coalescing refresh coordinator and SHALL produce the same status and publication semantics. The coordinator SHALL guarantee eventual processing of the newest accepted content revision for every changed path while allowing superseded intermediate revisions to be skipped.

#### Scenario: UI and skill save the same document in sequence
- **WHEN** two conditionally accepted saves produce successive durable revisions before graph processing completes
- **THEN** the coordinator may coalesce the first but eventually publishes a snapshot containing the second

#### Scenario: A conditional write conflicts
- **WHEN** an editor or skill write is rejected because its expected revision is stale
- **THEN** no refresh is scheduled for the rejected content

### Requirement: Plan-declared implementation-contract publication
Generated edge and event implementation contracts accepted through an active build plan SHALL enter the same durable conditional-save, embedding, refresh, and atomic-publication pipeline as connected authored Markdown. The plan SHALL retain its immutable base graph revision and record the graph revision produced by its own declared generated-contract outputs as its completion revision; only unrelated authoritative changes SHALL stale the plan during this transition.

#### Scenario: Build publishes one declared edge contract
- **WHEN** a building plan conditionally saves an authorized `docs/<edge-id>-contract.md`
- **THEN** the coordinator publishes it as implementation evidence and the plan records the resulting graph revision without becoming stale from that save

#### Scenario: Unrelated diagram changes during contract publication
- **WHEN** another save changes authoritative content outside the plan's declared contract-output set before completion
- **THEN** the plan becomes stale and cannot complete against the substituted graph revision

#### Scenario: Generated contract save conflicts
- **WHEN** an existing generated contract changed after the build captured its expected revision
- **THEN** the conditional save is rejected, no caller content is published, and the plan remains resumable with a conflict diagnostic
