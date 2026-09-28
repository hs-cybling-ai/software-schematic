# Project Runtime Daemon

## Purpose

Provide one lightweight local runtime for a developer iterating between an AI design interview, the diagram UI, and code generation.

## Requirements

### Requirement: One project-local runtime
An initialized project SHALL use one loopback-only runtime for its browser UI, current workflow, documents, graph, and MCP tools. Starting the wrapper or MCP adapter SHALL reuse that runtime when it is healthy and otherwise start it with bounded readiness.

#### Scenario: Developer opens the project
- **WHEN** the developer runs `./ssw`
- **THEN** the project UI opens against the single matching local runtime

#### Scenario: AI starts first
- **WHEN** Codex or Claude launches the project MCP adapter before the UI
- **THEN** the same runtime starts headlessly without opening a browser

### Requirement: Accidental cross-project attachment prevention
Discovery SHALL include the canonical project identity, runtime generation, loopback address, and random local token. A connection with mismatched project identity, generation, or token SHALL be rejected.

#### Scenario: Stale runtime metadata exists
- **WHEN** discovery points to a stopped or different runtime
- **THEN** startup replaces it or returns one short actionable retry message without touching authored files

### Requirement: Current workflow persistence
The runtime SHALL retain the current bounded interview summary, current proposal, latest publication receipt, and a bounded history of versioned build plans with separate progress and evidence records needed to resume work. Build plans SHALL be stored beneath `.ss/workflows`, validated against the current project and protocol on load, and excluded from the authoritative schematic source manifest. The runtime SHALL NOT retain credentials, full transcripts, copied requirement corpora, or executable orchestration scripts.

#### Scenario: UI is opened after an AI proposal
- **WHEN** a current proposal is waiting and the developer opens `./ssw`
- **THEN** the UI resumes that proposal on its affected diagram

#### Scenario: Build host starts after Play
- **WHEN** one or more open plans were created before Codex or Claude starts
- **THEN** the runtime restores their scope labels, semantic versions, lifecycle states, work progress, and bounded evidence

#### Scenario: Persisted plan belongs to another project
- **WHEN** a plan's project identity does not match the initialized project
- **THEN** the runtime rejects that plan without exposing it as executable work

#### Scenario: Legacy Play request exists
- **WHEN** an initialized project contains a compatible legacy `.ss/workflows/play.json`
- **THEN** the runtime migrates or presents it through a bounded compatibility path without modifying authored diagrams or Markdown

### Requirement: Lightweight diagnostics
The project wrapper SHALL report only project identity, runtime reachability, MCP wiring, and installed workflow skills, with an idempotent repair option for managed files.

#### Scenario: Developer repairs setup
- **WHEN** the developer runs `./ssw doctor --repair`
- **THEN** missing managed wrapper, MCP, guidance, and skill files are restored without changing diagrams, Markdown, or unrelated configuration

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
