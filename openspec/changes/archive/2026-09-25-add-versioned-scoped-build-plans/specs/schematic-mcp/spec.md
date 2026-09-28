## MODIFIED Requirements

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

### Requirement: Model-driven workflow tools
The MCP surface SHALL expose only capability discovery, guided design workspace entry, interview summary save/resume, typed diagram proposal submit/get/wait/cancel, graph synchronization/query, versioned build-plan list/get/wait/claim/record/complete operations, scope resolution, revision-bound implementation context, and bounded generated implementation-contract publication.

#### Scenario: AI enters the design workflow
- **WHEN** a registered design skill invokes `open_design_workspace`
- **THEN** the operation returns bounded orientation and launch state for the matching project without granting diagram mutation or proposal approval authority

#### Scenario: AI submits a diagram proposal
- **WHEN** registered operations reference current durable diagram revisions
- **THEN** the proposal enters browser review and cannot be self-approved by the AI

#### Scenario: AI lists open builds
- **WHEN** a build skill starts without a selector
- **THEN** the tool returns a bounded deterministic list of open plans with human scope labels, semantic versions, states, and opaque IDs suitable for assigning short invocation-local menu numbers

#### Scenario: AI waits for Play
- **WHEN** the design interview has no active proposal and the AI waits on the current plan revision
- **THEN** the call returns when the developer creates a plan through Play or `plan`, or after a short resumable timeout

#### Scenario: AI claims build work
- **WHEN** a client supplies a current open plan and expected progress revision
- **THEN** the tool atomically returns the next dependency-ready work item or reports that no item is claimable

#### Scenario: AI completes a build
- **WHEN** all work-item evidence, integration checks, and declared implementation-contract outputs were recorded
- **THEN** the AI completes the plan with bounded changed paths, check summaries, base graph revision, and completion graph revision

## ADDED Requirements

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
