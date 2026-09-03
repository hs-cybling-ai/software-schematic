# document-save-graph-refresh Specification

## Purpose
TBD - created by archiving change refresh-graph-after-web-save. Update Purpose after archive.
## Requirements
### Requirement: Successful web save refreshes the running graph
After the Software Schematic web application's typed server successfully persists complete CMMN XML, BPMN XML, or connected Markdown, the system SHALL notify the running MCP process for that exact canonical project to rebuild its derived graph. It SHALL NOT notify before persistence succeeds or for discarded/debounced intermediate content.

#### Scenario: Human saves a corrected diagram
- **WHEN** the web application successfully persists dirty diagram XML containing an evaluated contract correction
- **THEN** the running project MCP rebuilds from the current root-reachable documents and subsequent queries observe the correction under a new revision

#### Scenario: Human saves connected Markdown
- **WHEN** the web application successfully persists diagram-level or element-ID-bound Markdown
- **THEN** the running project MCP rebuilds its document chunks, embeddings, indexes, and owning graph entity from current documents

#### Scenario: Document save fails
- **WHEN** complete document replacement fails
- **THEN** the system reports the save failure and does not request a graph refresh

### Requirement: Complete atomic replacement
Each document-change notification SHALL cause or coalesce into a complete validated graph rebuild. The system SHALL atomically publish the completed snapshot and SHALL retain the last valid snapshot when parsing, validation, embedding, indexing, or another build step fails.

#### Scenario: Saved documents produce a valid graph
- **WHEN** the notified document set compiles successfully
- **THEN** all MCP readers switch from the prior complete revision to the replacement complete revision

#### Scenario: Saved documents are temporarily invalid
- **WHEN** the notified document set cannot produce a complete valid graph
- **THEN** MCP queries continue using the prior revision and the web application receives an actionable graph-refresh diagnostic

#### Scenario: Saves occur during a rebuild
- **WHEN** one or more successful saves arrive while graph rebuilding is active
- **THEN** notifications are coalesced and at least one subsequent serialized rebuild includes the newest durable documents

### Requirement: Private one-way project notification
The graph-refresh signal SHALL be a private local operation scoped to one canonical project and SHALL accept only a fixed document-changed notification from that project's web persistence service. It SHALL NOT be exposed as an MCP tool, public endpoint, arbitrary graph command, or graph mutation interface.

#### Scenario: Project identity does not match
- **WHEN** a notification does not authenticate as the same canonical project as the running MCP
- **THEN** the MCP rejects it without rebuilding or changing the published revision

#### Scenario: MCP is not running
- **WHEN** a web document save succeeds without a live matching MCP process
- **THEN** the document remains saved and the web application reports that the graph was not refreshed

### Requirement: Query-only LLM graph access
The MCP server SHALL expose only read/query operations for project overview, entity lookup, scope resolution, neighborhood traversal, and search. It SHALL NOT expose `reload_model` or any tool that writes, refreshes, or mutates the graph, diagrams, Markdown, status, or source code.

#### Scenario: Codex discovers project MCP tools
- **WHEN** an LLM client completes MCP tool discovery
- **THEN** every advertised Software Schematic tool is query-only and no reload or mutation tool is present

#### Scenario: Codex finds a missing contract detail
- **WHEN** graph context is incomplete or incorrect for implementation
- **THEN** project guidance requires Codex to stop and ask the user to evaluate and save the document change in the web application

