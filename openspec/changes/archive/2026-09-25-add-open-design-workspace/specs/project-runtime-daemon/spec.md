## ADDED Requirements

### Requirement: Secure assistant-requested workspace opening
The project runtime SHALL allow its authenticated project-local MCP adapter to request opening the browser modeling workspace against the already selected daemon generation. If a fresh browser session is already connected, the runtime SHALL reuse that session state without opening a duplicate workspace. Otherwise it SHALL construct the authenticated loopback launch URL internally, request the system browser, wait only for a bounded session-readiness interval, and return a launch state that contains no credential. A failed or unsupported browser launch SHALL leave the runtime and authored model unchanged and SHALL return an actionable wrapper command fallback.

#### Scenario: MCP started the runtime headlessly
- **WHEN** the design workflow requests the workspace before any browser session is connected
- **THEN** the runtime opens its own authenticated workspace URL in the system browser and reports whether a matching session registers within the bounded interval

#### Scenario: Browser session is already current
- **WHEN** the runtime has a fresh session for its current generation
- **THEN** the runtime returns that session's active context without opening another browser tab or window

#### Scenario: Browser session is stale
- **WHEN** the last registered session has exceeded the existing heartbeat timeout or belongs to another daemon generation
- **THEN** the runtime excludes its active context and treats the workspace as disconnected before deciding whether to request a browser launch

#### Scenario: Launch request targets the wrong project
- **WHEN** a workspace-open request is not authenticated through the matching project-local MCP connection
- **THEN** the runtime rejects it and does not open a browser or expose discovery credentials

#### Scenario: Local browser cannot be opened
- **WHEN** the operating system rejects or cannot service the browser request
- **THEN** the runtime returns a non-destructive failure with guidance to run `./ssw` or `ssw.cmd` for that project

#### Scenario: Launch result is serialized
- **WHEN** the workspace launch result is returned through MCP or retained in diagnostics
- **THEN** it omits the daemon token, authenticated launch URL, canonical project path, and browser-session identifier
