# schematic-mcp Specification

## Purpose
Expose the current project model and focused diagram-driven workflow to local AI clients through a bounded stdio MCP adapter.
## Requirements
### Requirement: Project-local stdio MCP service
`ss mcp --project <root>` SHALL attach standard input/output to the matching project runtime and SHALL NOT load a second graph or workflow owner. If the runtime restarted, the adapter SHALL exit with concise retry guidance; the host can relaunch it.

#### Scenario: MCP starts before the UI
- **WHEN** the AI client launches MCP for an initialized project
- **THEN** the project runtime becomes ready and serves the project tools without opening the browser

#### Scenario: MCP starts successfully
- **WHEN** a client launches `ss mcp` for a valid initialized project
- **THEN** the adapter starts or attaches to the one project runtime and advertises its workflow and graph tools after the graph is ready

#### Scenario: Initial load fails
- **WHEN** the project runtime cannot load the saved model
- **THEN** MCP reports the project-specific problem, exits unsuccessfully, and exposes no partial tool service

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
MCP graph queries SHALL reject raw XML replacement, arbitrary filesystem writes, executable payloads, unrestricted paths, direct graph mutation, and proposal approval. The workflow surface MAY accept compare-and-swap build-plan transitions and generated implementation-contract documents only for edge or event outputs declared by the active plan, at paths derived by the runtime from graph source identity; it SHALL reject caller-selected paths and content that expands beyond authorized plan scope.

#### Scenario: AI needs more contract detail
- **WHEN** implementation context is incomplete in a way that would change product scope or behavior
- **THEN** the workflow returns to the design interview and diagram instead of inventing a parallel prose specification

#### Scenario: Build records actual implementation contract
- **WHEN** the active work item implements a declared edge or event and submits bounded contract content
- **THEN** the runtime derives the `docs/<element-id>-contract.md` path, validates plan authority and revisions, and conditionally persists the generated evidence

#### Scenario: Client enumerates MCP capabilities
- **WHEN** MCP initialization and tool listing complete
- **THEN** every mutation-capable tool is limited to typed proposal workflow, durable-document synchronization, compare-and-swap plan progress, or plan-declared implementation-contract evidence

### Requirement: Guided design workspace entry
The MCP server SHALL expose `open_design_workspace` as the bounded entry operation for the design workflow. The operation SHALL request that the matching project runtime open the modeling workspace when no fresh browser session exists and SHALL return project identity, graph and source-manifest revisions, root-anchor context, fresh active diagram and selection context when present, bounded resumable process candidates, and resumable interview and proposal summaries. Each process candidate SHALL include stable model identity, a human-readable label or Name, its owning business anchor, composition availability, and the information needed by subsequent model queries without requiring the user to handle a path or identifier. The response SHALL report a bounded launch state and actionable fallback diagnostic, but SHALL NOT return the daemon token, an authenticated launch URL, an absolute project path, or unrestricted document content.

#### Scenario: Design begins against a headless runtime
- **WHEN** a client calls `open_design_workspace` and no fresh browser session is registered
- **THEN** the runtime securely requests the system browser to open the matching project workspace and the tool returns whether a browser session connected within the bounded readiness interval

#### Scenario: Modeling workspace is already connected
- **WHEN** a fresh browser session is registered for the matching runtime generation
- **THEN** the tool does not open a duplicate workspace and returns the active diagram and selected entity references from that session

#### Scenario: Existing processes are available
- **WHEN** the current graph contains reachable named process designs or named Process Task anchors
- **THEN** the tool returns a deterministic bounded candidate list that distinguishes existing compositions from anchors whose composition can be created

#### Scenario: Design work can be resumed
- **WHEN** the project has a current durable interview or non-terminal proposal
- **THEN** the response identifies its target and state so the skill can resume it before asking the developer to choose unrelated work

#### Scenario: Browser launch fails
- **WHEN** the runtime cannot request the local system browser or no browser session connects within the readiness interval
- **THEN** the tool preserves all model and workflow state and returns concise guidance to run the project wrapper without disclosing an authenticated URL

#### Scenario: Workspace response is inspected for secrets
- **WHEN** `open_design_workspace` succeeds, is logged, or returns an error
- **THEN** its model-visible result contains no daemon token or authenticated browser URL

### Requirement: Model-driven workflow tools
The MCP surface SHALL expose only capability discovery, guided design workspace entry, interview summary save/resume, typed diagram proposal submit/get/wait/cancel, graph synchronization/query, versioned build-plan list/get/wait/claim/record/complete operations, scope resolution, revision-bound implementation context, and bounded generated implementation-contract publication.

#### Scenario: AI enters the design workflow
- **WHEN** a registered design skill invokes `open_design_workspace`
- **THEN** the operation returns bounded orientation and launch state for the matching project without granting diagram mutation or proposal approval authority

#### Scenario: AI submits a diagram proposal
- **WHEN** registered operations reference current durable diagram revisions
- **THEN** the proposal enters browser review and cannot be self-approved by the AI

#### Scenario: AI waits for Play
- **WHEN** the design interview has no active proposal and the AI waits on the current Play revision
- **THEN** the call returns when the developer creates a plan through Play or `plan`, or after a short resumable timeout

#### Scenario: AI lists open builds
- **WHEN** a build skill starts without a selector
- **THEN** the tool returns a bounded deterministic list of open plans with human scope labels, semantic versions, states, and opaque IDs suitable for assigning short invocation-local menu numbers

#### Scenario: AI claims build work
- **WHEN** a client supplies a current open plan and expected progress revision
- **THEN** the tool atomically returns the next dependency-ready work item or reports that no item is claimable

#### Scenario: AI completes a build
- **WHEN** all work-item evidence, integration checks, and declared implementation-contract outputs were recorded
- **THEN** the AI completes the plan with bounded changed paths, check summaries, base graph revision, and completion graph revision

### Requirement: Versioned build-plan lookup
The MCP service SHALL allow build plans to be retrieved by opaque plan ID or by the compound identity of exact scope and semantic version. A bare semantic version SHALL NOT silently select among plans for different scopes, and list results SHALL be deterministically ordered and bounded.

#### Scenario: Same version exists for two components
- **WHEN** two scopes both have an open version `1.0.0` and the client supplies only `1.0.0`
- **THEN** the service returns bounded candidates and does not choose one plan

#### Scenario: Exact scope and version are supplied
- **WHEN** the client supplies a valid scope identity and semantic version
- **THEN** the service returns that plan and its current progress revision

### Requirement: Natural-language plan scope resolution
The MCP service SHALL resolve natural-language plan intent against reachable diagram, pool, participant, node, edge, event, label, Name, documentation, and composition-stack context. It SHALL return one scope only when confidence and candidate separation satisfy configured bounds; otherwise it SHALL return deterministic human-readable candidates and SHALL exclude orphan diagrams from plan authority.

#### Scenario: Description uniquely identifies a composed node
- **WHEN** plan intent names a node and enough of its containing process stack to identify it uniquely
- **THEN** the service returns its universal entity identity and a human-readable composition breadcrumb suitable for confirmation

#### Scenario: Description matches duplicate labels
- **WHEN** multiple reachable entities have the same or similarly relevant label
- **THEN** the service returns bounded candidates without creating a plan or choosing silently

### Requirement: Work-item implementation context
The MCP service SHALL return revision-bound implementation context for one claimed work item containing its eligible focused entity, permitted in-scope entities, containment and composition descendants, incident edges and events, connected context entities, prior completed-item evidence, citations, and explicit exclusions. It SHALL NOT use ancestor traversal to add parent-service implementation targets.

#### Scenario: Component work item requests context
- **WHEN** a build client requests context for a claimed component item
- **THEN** the response contains enough selected, downward, connected, and prior-work context to implement working software while distinguishing authorized targets from context-only entities

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
