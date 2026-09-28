## ADDED Requirements

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

## MODIFIED Requirements

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
