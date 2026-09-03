# schematic-mcp Specification

## Purpose
TBD - created by archiving change add-schematic-graph-mcp. Update Purpose after archive.
## Requirements
### Requirement: Project-local stdio MCP service
The CLI SHALL provide `ss mcp --project <root>` and SHALL fully load the in-memory schematic graph before serving Model Context Protocol requests over standard input and standard output. It SHALL reserve standard output for MCP protocol messages and write human-readable startup or failure diagnostics to standard error.

#### Scenario: MCP starts successfully
- **WHEN** a client launches `ss mcp` for a valid initialized project
- **THEN** the server completes MCP initialization and advertises its read-oriented schematic tools after the graph snapshot is ready

#### Scenario: Initial load fails
- **WHEN** the project graph cannot be built
- **THEN** the process reports actionable diagnostics on standard error, exits unsuccessfully, and emits no partial tool service

### Requirement: Project model overview
The MCP server SHALL expose a tool that returns the root diagram, snapshot revision, embedding model, load timestamp, reachable diagram summaries, entity and chunk counts, and load diagnostics.

#### Scenario: Agent begins project work
- **WHEN** an MCP client requests the project model overview
- **THEN** the response gives the bounded inventory and revision needed to choose subsequent entity, traversal, or search calls

### Requirement: Exact entity lookup
The MCP server SHALL expose a tool that resolves one diagram, node, or edge by universal URN `id`, `{ownerName, sourceId}`, canonical Name, or unambiguous bare source ID and returns its compiled properties, semantic owner, relationships summary, Markdown excerpts, and optional source-map citation. Ambiguous or absent identifiers SHALL return structured candidates or a not-found diagnostic rather than selecting silently.

#### Scenario: Canonical Name is unique
- **WHEN** a client looks up a unique complete Software Schematic Name
- **THEN** the response identifies the matching entity and includes its universal URN ID, exact source ID, semantic owner Name, optional source citation, and snapshot revision

#### Scenario: Short identifier is ambiguous
- **WHEN** a client supplies an XML ID that exists under multiple semantic owners without an owner Name
- **THEN** the tool reports ambiguity and returns bounded candidate identities without choosing one

### Requirement: Proposal-language development-scope resolution
The MCP server SHALL expose `resolve_development_scope` accepting natural proposal language and an optional explicit root universal URN or `{ownerName, sourceId}`. It SHALL use Grafeo hybrid search restricted to `new` and `modify` schematic entities and graph relationships to rank root candidates. It SHALL automatically select a root only when the highest candidate satisfies configured confidence and separation thresholds; otherwise it SHALL return bounded candidates for user selection. A selected root and every authorized implementation target SHALL have status `new` or `modify`; `open` and `locked` neighbors SHALL be returned only as context.

#### Scenario: Natural language has one strong eligible match
- **WHEN** proposal language strongly matches one green or orange node and its documented neighborhood
- **THEN** the tool returns that node's universal URN ID, semantic owner Name, `sourceId`, matching evidence, and reachable `new`/`modify` scope without requiring an ID in the user's prompt

#### Scenario: Multiple eligible roots are plausible
- **WHEN** two or more `new` or `modify` nodes have close proposal-language relevance
- **THEN** the tool returns ranked candidates with statuses, paths, excerpts, and scores and does not silently choose a root

#### Scenario: No eligible node matches
- **WHEN** proposal language matches only `open` or `locked` nodes or no modeled node
- **THEN** the tool returns no authorized scope and instructs the user to mark the intended node `new` or `modify` in the diagram

#### Scenario: Explicit root is provided by a proposal
- **WHEN** a proposal already contains a universal root URN or unambiguous `{ownerName, sourceId}`
- **THEN** the tool validates that root is still `new` or `modify` and returns its current authorized scope without semantic reselection

### Requirement: Project identity verification
The MCP server SHALL be launched with an explicit project root, canonicalize and validate that root as an initialized Software Schematic project, and return `projectName`, deterministic `projectId`, root diagram universal URN plus `diagramPath` and `sourceId`, and snapshot revision in project overview and development-scope responses. It SHALL NOT load a project inferred from an arbitrary caller working directory.

#### Scenario: Pinned project wrapper launches MCP
- **WHEN** Codex starts the repository's `ssw mcp` wrapper from any working directory
- **THEN** the wrapper derives its own repository root, the server loads that root's `schematics/main.cmmn`, and project overview identifies that same project

#### Scenario: Project root is invalid
- **WHEN** the supplied root lacks the expected initialized runtime or root schematic
- **THEN** MCP startup fails with a project-specific diagnostic before serving tools

#### Scenario: Agent receives a mismatched identity
- **WHEN** project overview does not identify the active Codex repository expected by managed guidance
- **THEN** the agent treats the MCP as misconfigured and does not use it to authorize implementation scope

### Requirement: Node-linked lightweight proposals
Managed agent guidance SHALL require generated code proposals to record the MCP-resolved root identity and require every implementation task to reference at least one authorized diagram entity. It SHALL direct agents to keep detailed requirements and design in diagram structure and Markdown and SHALL NOT require the user to include IDs in natural-language requests.

#### Scenario: Agent creates an OpenSpec proposal
- **WHEN** MCP confidently resolves the user's request to a development root
- **THEN** the proposal remains concise, records the returned root, and records `nodeRefs` on every task instead of duplicating the detailed diagram contract

#### Scenario: Task lacks an in-scope node
- **WHEN** an agent identifies necessary work that cannot be linked to a returned `new` or `modify` entity
- **THEN** the agent stops and asks for the diagram scope to be updated rather than adding an unscoped task

### Requirement: Bounded neighborhood traversal
The MCP server SHALL expose a tool for bounded incoming and outgoing traversal from an entity, with optional relationship-type and direction filters. The server SHALL enforce a conservative maximum hop depth and result count and SHALL return relationship direction, type, endpoint identity, and source provenance.

#### Scenario: Agent asks what a node affects
- **WHEN** a client requests outgoing topology and composition neighbors for a node
- **THEN** the response returns the bounded matching relationships and entities in deterministic order

#### Scenario: Client exceeds traversal limit
- **WHEN** a client requests more hops or results than the configured maximum
- **THEN** the tool rejects or clamps the request explicitly and reports the applied bound

### Requirement: Grafeo-native hybrid schematic search
The MCP server SHALL expose bounded search across entity properties and Markdown using Grafeo's native text, vector, and hybrid-search facilities, with optional entity-kind, diagram, and neighborhood filters. Each result SHALL include its owner entity, source path, excerpt, snapshot revision, Grafeo relevance information, graph-distance context, and deterministic tie-breaking. The MCP layer SHALL NOT reimplement vector similarity or maintain a second search index.

#### Scenario: Concept is described only in Markdown
- **WHEN** a client searches for a concept semantically related to a documented chunk but absent from entity Labels and Names
- **THEN** search can return the owning entity and chunk with vector score and source citation

#### Scenario: Search is scoped to a composition
- **WHEN** a client supplies a reachable diagram filter and result limit
- **THEN** every result belongs to that diagram or its requested bounded neighborhood and the response does not exceed the limit

### Requirement: Explicit snapshot reload
The MCP server SHALL expose a tool that rebuilds the complete graph from current project files and atomically publishes it only on success. The response SHALL identify whether the revision changed and SHALL return the active revision and diagnostics.

#### Scenario: Files changed successfully
- **WHEN** a client requests reload after valid schematic or Markdown edits
- **THEN** subsequent tool calls observe the new complete snapshot and its new revision

#### Scenario: Files changed invalidly
- **WHEN** reload encounters invalid reachable content
- **THEN** the response reports failure and subsequent tool calls continue to use the prior revision

### Requirement: Read-only constrained MCP surface
The first MCP service SHALL NOT expose arbitrary GQL, graph mutation, schematic file mutation, source-code mutation, remote HTTP transport, or authentication. All tool inputs SHALL use explicit schemas and all returned collections and text excerpts SHALL be bounded.

#### Scenario: Client enumerates MCP capabilities
- **WHEN** MCP initialization and tool listing complete
- **THEN** only project overview, entity lookup, proposal-language scope resolution, bounded traversal, bounded search, and snapshot reload capabilities are available

