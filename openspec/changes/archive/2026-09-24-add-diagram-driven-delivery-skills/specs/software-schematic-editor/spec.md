## ADDED Requirements

### Requirement: Strong document revisions and conditional writes
Every typed diagram and Markdown read SHALL return a strong content revision derived from the complete durable content. Every write SHALL provide the revision on which the edit was based and SHALL atomically replace the document only when that expected revision still matches, returning the new content revision on success. Revision correctness SHALL NOT depend on filesystem modification-time granularity or client clocks.

#### Scenario: Conditional save succeeds
- **WHEN** the editor saves complete content with the current expected revision
- **THEN** the server atomically persists it and returns a different strong revision for changed content

#### Scenario: Concurrent update wins first
- **WHEN** another editor or skill has already changed the document after the client's read
- **THEN** the server rejects the stale write with the expected and current revisions and preserves the newer durable content

### Requirement: Actionable concurrent-edit reconciliation
When a conditional write conflicts, the editor SHALL preserve the user's unsaved content, load the current durable content, and present semantic choices to compare, retry when non-overlapping, or explicitly choose a version. It SHALL NOT silently use last-writer-wins. Agent-authored coordinated proposals SHALL be regenerated from current revisions unless a deterministic non-overlapping merge of Markdown can be previewed and explicitly approved.

#### Scenario: Diagram XML conflicts
- **WHEN** a stale diagram write conflicts with a newer durable diagram revision
- **THEN** the editor preserves the local model, blocks automatic overwrite, identifies the competing revision, and offers reload or an explicitly previewed reapplication path

#### Scenario: Markdown edits do not overlap
- **WHEN** local and durable Markdown changes can be deterministically merged without overlapping hunks
- **THEN** the editor may preview the merged document and persists it only after user approval against the current revision

#### Scenario: User resolves conflict in the UI
- **WHEN** the user selects a resolution that replaces current durable content
- **THEN** the editor performs a new conditional write against the displayed current revision and records the resolution as a new revision

