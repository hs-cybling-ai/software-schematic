## MODIFIED Requirements

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

### Requirement: Read-only constrained MCP surface
MCP tools SHALL reject raw XML replacement, arbitrary filesystem writes, executable payloads, unrestricted paths, direct graph mutation, and proposal approval.

#### Scenario: AI needs more contract detail
- **WHEN** implementation context is incomplete
- **THEN** the workflow returns to the design interview and diagram instead of inventing a parallel prose specification

#### Scenario: Client enumerates MCP capabilities
- **WHEN** MCP initialization and tool listing complete
- **THEN** only the focused interview, proposal, graph, Play, and build-context operations are available

## ADDED Requirements

### Requirement: Model-driven workflow tools
The MCP surface SHALL expose only capability discovery, interview summary save/resume, typed diagram proposal submit/get/wait/cancel, graph synchronization/query, Play request get/wait/complete, scope resolution, and revision-bound implementation context.

#### Scenario: AI submits a diagram proposal
- **WHEN** registered operations reference current durable diagram revisions
- **THEN** the proposal enters browser review and cannot be self-approved by the AI

#### Scenario: AI waits for Play
- **WHEN** the design interview has no active proposal and the AI waits on the current Play revision
- **THEN** the call returns when the developer clicks Play or at a short resumable timeout

#### Scenario: AI completes a build
- **WHEN** code and checks were produced for the current Play request and context grant
- **THEN** the AI records a bounded outcome containing changed paths and check summaries
