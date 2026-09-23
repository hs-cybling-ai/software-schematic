## MODIFIED Requirements

### Requirement: Successful web save refreshes the running graph
After the Software Schematic web application's typed server successfully persists complete CMMN XML, BPMN XML, or connected Markdown, the system SHALL submit a path-aware document-change notification for that exact canonical project to the running MCP process. The notification SHALL identify the normalized project-relative document path and change kind, SHALL NOT be sent before persistence succeeds or for discarded/debounced intermediate content, and SHALL be acknowledged without waiting for graph compilation, embedding, indexing, or publication.

#### Scenario: Human saves a corrected diagram
- **WHEN** the web application successfully persists dirty diagram XML containing an evaluated contract correction
- **THEN** the running project MCP accepts the diagram path for background refresh and the durable save completes without waiting for the graph build

#### Scenario: Human saves connected Markdown
- **WHEN** the web application successfully persists diagram-level or element-ID-bound Markdown
- **THEN** the running project MCP accepts that Markdown path for targeted background regeneration of its owning entity's document chunks, embeddings, and indexes

#### Scenario: Document save fails
- **WHEN** complete document replacement fails
- **THEN** the system reports the save failure and does not request a graph refresh

### Requirement: Complete atomic replacement
Each accepted document-change notification SHALL be processed by a serialized background worker using targeted invalidation when dependency impact is safely known and a complete rebuild fallback otherwise. The system SHALL validate a complete candidate snapshot, atomically publish it, and retain the last valid snapshot when parsing, validation, embedding, indexing, or another build step fails.

#### Scenario: Saved documents produce a valid targeted update
- **WHEN** all effects of a changed document can be safely identified and the candidate compiles successfully
- **THEN** only affected source fragments and derived artifacts are regenerated before all MCP readers atomically switch to the complete replacement revision

#### Scenario: Structural impact is uncertain
- **WHEN** a change can alter root reachability, semantic identity, composition dependencies, rename or deletion effects, or otherwise cannot be safely bounded
- **THEN** the worker runs a complete rebuild in the background before publishing

#### Scenario: Saved documents are temporarily invalid
- **WHEN** the notified document set cannot produce a complete valid graph
- **THEN** MCP queries continue using the prior revision and refresh status records an actionable diagnostic independently of document-save success

#### Scenario: Saves occur during a rebuild
- **WHEN** one or more successful saves arrive while graph rebuilding is active
- **THEN** notifications are coalesced by path and at least one subsequent serialized build includes the newest durable content for every accepted path

### Requirement: Private one-way project notification
The graph-refresh signal SHALL be a private local operation scoped to one canonical project and SHALL accept only a fixed, versioned document-changed notification from that project's web persistence service. It SHALL validate the normalized confined path and change kind before enqueueing work and SHALL NOT be exposed as an MCP tool, public endpoint, arbitrary graph command, or graph mutation interface.

#### Scenario: Project identity does not match
- **WHEN** a notification does not authenticate as the same canonical project as the running MCP
- **THEN** the MCP rejects it without enqueueing work, rebuilding, or changing the published revision

#### Scenario: Changed path escapes the project
- **WHEN** a notification names an absolute, parent-traversing, symlink-escaping, or unsupported document path
- **THEN** the MCP rejects it without reading the path or changing refresh state

#### Scenario: MCP is not running
- **WHEN** a web document save succeeds without a live matching MCP process
- **THEN** the document remains saved and the web application reports that the graph was not refreshed

## ADDED Requirements

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

