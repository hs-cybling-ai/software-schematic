## Purpose

Define a portable, diagram-first software delivery workflow in which conversational design, graph synchronization, and node-scoped implementation share one annotated schematic contract across Codex and Claude Code.

## ADDED Requirements

### Requirement: Portable skill suite
The project SHALL provide `design`, `graph`, and `build` skills with a shared behavioral contract and documented adapters for Codex and Claude Code. Each skill SHALL verify project identity and required capabilities before acting, SHALL carry stable diagram entity references and graph revisions across handoffs, and SHALL return actionable guidance when the host or project is incompatible.

#### Scenario: Skills run in a supported host
- **WHEN** a user invokes any delivery skill from Codex or Claude Code in an initialized project
- **THEN** the skill verifies the same project identity and uses the same diagram and graph contracts independent of the host

#### Scenario: Required integration is unavailable
- **WHEN** a skill cannot reach the matching project service or the service lacks a required protocol version
- **THEN** it makes no diagram or code change and explains how to restore the integration

### Requirement: Interview-driven design skill
The `design` skill SHALL interview the user in bounded rounds, identify missing decisions, and translate approved answers into typed proposals for BPMN or CMMN structure and richly annotated Markdown on diagrams, nodes, and edges. It SHALL show the semantic proposal before mutation, require approval, apply it against the source revisions used to generate it, and continue the interview from the newly published graph revision.

#### Scenario: Interview produces a diagram increment
- **WHEN** the user's answers are sufficient to define a coherent portion of a process
- **THEN** the skill proposes the affected elements, flows, statuses, and Markdown annotations and applies them only after approval

#### Scenario: Design contract remains ambiguous
- **WHEN** two materially different process interpretations remain plausible
- **THEN** the skill asks a focused follow-up question instead of inventing diagram structure

#### Scenario: Approved design becomes stale
- **WHEN** any affected document changes after proposal generation and before application
- **THEN** the skill refuses the stale write, summarizes the conflict, and regenerates or asks the user to reconcile it

### Requirement: Changed-document graph skill
The `graph` skill SHALL identify durable diagram and Markdown changes since a known source manifest, request synchronization only for those canonical documents, and wait for a publication receipt that identifies the source manifest and active graph revision. It SHALL report unchanged content without rebuilding it and SHALL permit the service to select a safe full rebuild when incremental correctness cannot be proven.

#### Scenario: One annotation changes
- **WHEN** one node Markdown document has a new durable content revision and its owner is unchanged
- **THEN** the skill requests synchronization for that document and returns the graph revision that includes it without requesting unrelated diagram reloads

#### Scenario: No authoritative content changed
- **WHEN** every current document digest matches the last published source manifest
- **THEN** the skill reports the graph current without starting a rebuild

#### Scenario: Structural impact is uncertain
- **WHEN** a rename, deletion, composition change, or dependency ambiguity prevents bounded invalidation
- **THEN** the graph service performs or requests a safe full rebuild and the skill clearly reports the fallback

### Requirement: Natural-language node-scoped build skill
The `build` skill SHALL resolve a natural-language request to one or more eligible `new` or `modify` diagram entities, require user selection when resolution is ambiguous, and freeze the authorized entity set and graph revision before editing source code. It SHALL use node and edge annotations plus bounded graph context as the implementation contract, SHALL limit code changes to that scope, SHALL run relevant verification, and SHALL report changed code and verification against stable entity references.

#### Scenario: One node clearly matches
- **WHEN** a natural-language request confidently resolves to one eligible node
- **THEN** the skill presents the resolved node and implementation boundary, then authors and verifies code using its revision-bound annotated neighborhood

#### Scenario: Multiple nodes plausibly match
- **WHEN** natural language resolves to multiple similarly ranked eligible nodes
- **THEN** the skill presents bounded candidates and makes no source change until the user selects the intended scope

#### Scenario: Contract changes during implementation
- **WHEN** the active graph revision changes in a way that modifies an authorized entity or its contract before completion
- **THEN** the skill stops before further writes and requires scope revalidation against the new revision

### Requirement: Seamless skill handoff
The skills SHALL persist a small project-local handoff record containing project identity, source manifest revision, graph revision, selected entity references, workflow phase, and non-sensitive outcome metadata. A subsequent skill invocation SHALL resume from that record only after validating it against the current project and SHALL not require the user to restate diagram identifiers.

#### Scenario: User moves from design to build
- **WHEN** an approved design change has been published and the user asks to build the discussed feature
- **THEN** the build skill reuses the validated entity references and graph revision without requiring manual ID lookup or a separate specification command

#### Scenario: Handoff belongs to another project
- **WHEN** a handoff record's project identity differs from the active project
- **THEN** the skill rejects the record and performs fresh project-local resolution

