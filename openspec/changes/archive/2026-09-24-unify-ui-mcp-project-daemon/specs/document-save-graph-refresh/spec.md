## MODIFIED Requirements

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

### Requirement: Private one-way project notification
Graph-refresh ingestion SHALL remain a private daemon operation scoped to one canonical project and SHALL accept only durable, revision-validated document identities from successful typed persistence or constrained synchronization requests. It SHALL NOT accept caller-supplied graph content, arbitrary graph commands, public network requests, or direct graph mutation.

#### Scenario: Project identity does not match
- **WHEN** a synchronization request does not authenticate as the same canonical project as the daemon
- **THEN** the daemon rejects it without rebuilding or changing the published revision

#### Scenario: Changed path escapes the project
- **WHEN** a synchronization request names an absolute, parent-traversing, symlink-escaping, or unsupported document path
- **THEN** the daemon rejects it without reading the path or changing refresh state

#### Scenario: No MCP client is connected
- **WHEN** a web document save succeeds while no MCP proxy is connected
- **THEN** the daemon still refreshes the graph and retains the resulting receipt for a later client

#### Scenario: MCP is not running
- **WHEN** a web document save succeeds while no MCP proxy process is running
- **THEN** the daemon still refreshes the graph and retains the resulting receipt for a later client

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
