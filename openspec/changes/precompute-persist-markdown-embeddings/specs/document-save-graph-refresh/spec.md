## MODIFIED Requirements

### Requirement: Successful web save refreshes the running graph
After the Software Schematic web application's typed server successfully persists complete CMMN XML, BPMN XML, or connected Markdown, the system SHALL notify the running MCP process for that exact canonical project to refresh its derived graph. For connected Markdown, it SHALL also schedule owner-scoped embedding derivation from the durable content. Neither save acknowledgment nor initial graph availability SHALL wait for embedding generation.

#### Scenario: Human saves a corrected diagram
- **WHEN** the web application successfully persists dirty diagram XML containing an evaluated contract correction
- **THEN** the running project MCP refreshes from the current root-reachable documents and subsequent queries observe the correction under the current source revision

#### Scenario: Human saves connected Markdown
- **WHEN** the web application successfully persists diagram-level or element-ID-bound Markdown
- **THEN** it schedules latest-content embedding derivation and refreshes the owning graph entity and text index without waiting for vector readiness

#### Scenario: Document save fails
- **WHEN** complete document replacement fails
- **THEN** the system reports the save failure and schedules neither graph refresh nor embedding derivation

### Requirement: Complete atomic replacement
Each document-change notification SHALL cause or coalesce into a validated graph refresh for the newest durable authoritative content. The system SHALL atomically publish a complete text-searchable snapshot and SHALL retain the last valid snapshot when parsing, validation, text indexing, or another authoritative build step fails. Embedding artifact updates SHALL trigger or coalesce into a subsequent atomic vector-index refresh only when the artifact still matches that authoritative content.

#### Scenario: Saved documents produce a valid graph
- **WHEN** the notified authoritative document set compiles successfully
- **THEN** all MCP readers switch from the prior complete revision to the replacement complete text-searchable revision without waiting for pending embeddings

#### Scenario: Saved documents are temporarily invalid
- **WHEN** the notified authoritative document set cannot produce a complete valid graph
- **THEN** MCP queries continue using the prior revision and the web application receives an actionable graph-refresh diagnostic

#### Scenario: Saves occur during a rebuild
- **WHEN** one or more successful saves arrive while embedding derivation or graph refresh is active
- **THEN** work is coalesced by owner and at least one subsequent serialized publication represents the newest durable documents and only matching embedding artifacts

#### Scenario: Matching artifact becomes ready
- **WHEN** asynchronous derivation publishes a valid artifact for the current Markdown hash
- **THEN** the MCP atomically enables its vectors without rebuilding unrelated embeddings
