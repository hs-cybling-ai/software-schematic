## MODIFIED Requirements

### Requirement: Observable asynchronous refresh lifecycle
The system SHALL maintain project-scoped refresh state containing the latest notification ID, lifecycle state, affected paths, prior and active revisions, timestamps, retrieval mode, vector readiness, and any actionable graph diagnostic. Expected absence or staleness of an embedding artifact while derivation is queued or processing SHALL be represented as vector readiness rather than an actionable graph diagnostic. The typed web server SHALL provide this state to its own browser client without exposing a graph mutation operation.

#### Scenario: Accepted refresh is still running
- **WHEN** the browser requests refresh status after an accepted notification has been queued or started
- **THEN** it receives `queued` or `processing` state while MCP queries continue against the prior complete revision

#### Scenario: Background refresh completes
- **WHEN** the worker publishes or determines no change to a complete candidate
- **THEN** refresh status becomes `updated` or `unchanged` and identifies the active revision

#### Scenario: Text-ready graph is published before vectors
- **WHEN** a connected Markdown body is published into a complete text-searchable graph before its current embedding artifact is available
- **THEN** refresh status reports text retrieval ready and vectors pending without an actionable graph diagnostic for the expected missing or stale header

#### Scenario: Background refresh fails
- **WHEN** the worker cannot build a valid candidate
- **THEN** refresh status becomes `failed`, includes an actionable diagnostic, and identifies the retained active revision
