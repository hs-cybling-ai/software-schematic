## Purpose

Provide one lightweight local runtime for a developer iterating between an AI design interview, the diagram UI, and code generation.

## ADDED Requirements

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
The runtime SHALL retain the current bounded interview summary, current proposal, latest publication receipt, and current Play request needed to resume work. It SHALL NOT retain credentials, full transcripts, or duplicated requirement corpora.

#### Scenario: UI is opened after an AI proposal
- **WHEN** a current proposal is waiting and the developer opens `./ssw`
- **THEN** the UI resumes that proposal on its affected diagram

### Requirement: Lightweight diagnostics
The project wrapper SHALL report only project identity, runtime reachability, MCP wiring, and installed workflow skills, with an idempotent repair option for managed files.

#### Scenario: Developer repairs setup
- **WHEN** the developer runs `./ssw doctor --repair`
- **THEN** missing managed wrapper, MCP, guidance, and skill files are restored without changing diagrams, Markdown, or unrelated configuration
