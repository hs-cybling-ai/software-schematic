## MODIFIED Requirements

### Requirement: Portable skill suite
The project SHALL provide small `design`, `plan`, and `build` skills for Codex and Claude Code. Each skill SHALL use the one project runtime and current saved diagram workflow, return actionable setup guidance when unavailable, and treat runtime-generated build plans as disposable execution state rather than copied specifications. Equivalent host adapters SHALL consume the same plan protocol and SHALL NOT generate provider-specific executable orchestration scripts. The managed user-facing `graph` skill SHALL be removed; graph synchronization remains an internal runtime capability used by plan creation.

#### Scenario: Skills run in a supported host
- **WHEN** a user invokes any delivery skill from Codex or Claude Code in an initialized project
- **THEN** the skill uses the same current project diagram, versioned plans, and runtime workflow independent of the host

#### Scenario: Required integration is unavailable
- **WHEN** a skill cannot reach the matching project runtime
- **THEN** it makes no diagram or code change and explains how to restart or repair the project integration

## ADDED Requirements

### Requirement: Conversational plan skill
The `plan` skill SHALL accept a natural-language description of the pool, participant, process, node, label, event, edge, or composed item the developer intends to build; verify project identity and graph publication; search only the reachable diagram stack; resolve one stable scope identity or present a bounded numbered candidate list; obtain a major, minor, or fix/patch choice when a changed prior scope version requires one; and invoke the same plan-creation operation as Play. It SHALL NOT implement code, approve diagram proposals, use orphan diagrams, or create a prose task specification.

#### Scenario: Developer describes a unique labeled activity
- **WHEN** the developer invokes `plan` with natural language that confidently matches one reachable labeled activity
- **THEN** the skill confirms its diagram and label, applies the normal version rule, and creates the same selected-boundary plan as Play

#### Scenario: Label occurs in multiple compositions
- **WHEN** the requested label exists in more than one reachable diagram or composition
- **THEN** the skill presents short numbered candidates with their human diagram stack and waits for selection

#### Scenario: Requested item is not reachable
- **WHEN** the description matches only an orphan diagram or no current model entity
- **THEN** the skill creates no plan and directs the developer back to the reachable design

#### Scenario: Scope contract changed since its latest plan
- **WHEN** the resolved scope has a prior plan but a different current scope-contract revision
- **THEN** the skill asks for major, minor, or fix/patch and reports the created semantic version

## MODIFIED Requirements

### Requirement: Natural-language node-scoped build skill
When invoked without a selector, the `build` skill SHALL list open build plans as a short numbered menu containing human scope labels, semantic versions, and states. It SHALL accept an invocation-local menu number or an exact scope/version selector, verify the chosen plan's project and revisions, and execute one dependency-ready work item at a time. For each item it SHALL load the focused `new` or `modify` entity plus bounded contained, composed, edge, event, connected, and completed-item context; implement only the authorized behavior and necessary physical effects; run relevant checks; record evidence; and continue until integration verification and implementation-contract publication complete the plan.

#### Scenario: One node clearly matches
- **WHEN** the selected plan contains one dependency-ready eligible model element
- **THEN** the skill claims and implements its revision-bound annotated work item

#### Scenario: Multiple nodes plausibly match
- **WHEN** plan creation from natural language returns multiple plausible eligible roots
- **THEN** the plan skill presents bounded candidates and no build source change occurs until the developer selects the intended scope and a plan exists

#### Scenario: User does not remember a plan identifier
- **WHEN** the developer invokes `build` without a version or plan ID
- **THEN** the skill presents numbered open builds and asks only for the short number needed for that invocation

#### Scenario: User requests an exact version
- **WHEN** the developer supplies a scope/version selector that identifies one open plan
- **THEN** the skill resumes that plan without asking the developer to restate diagram identifiers or requirements

#### Scenario: One work item is ready
- **WHEN** the selected plan has a dependency-ready pending item
- **THEN** the skill claims it, loads its revision-bound focused and supporting context, implements and verifies it, records its physical effects, and then requests the next item

#### Scenario: Physical implementation needs tangent repository changes
- **WHEN** working in-scope software requires shared wiring, configuration, persistence, migration, or test changes not modeled as separate logical features
- **THEN** the skill may make those changes as evidence-linked effects of the current work item without authorizing unrelated features

#### Scenario: Contract changes during implementation
- **WHEN** the authoritative graph changes outside the plan's declared generated-contract outputs before completion
- **THEN** the skill stops further writes, marks the plan stale, and waits for a revalidated Play action

#### Scenario: Plan implementation completes
- **WHEN** every work item and integration check succeeds
- **THEN** the skill publishes required edge and event implementation contracts, records final changed paths and checks, and completes the plan against the resulting graph revision

## REMOVED Requirements

### Requirement: Changed-document graph skill
**Reason**: Developers need a memorable plan-creation entry point rather than a separate user-facing publication workflow. Plan creation itself must verify that saved diagram and Markdown changes are represented by the current graph.

**Migration**: Replace generated `.codex/skills/graph` and `.claude/commands/graph.md` adapters with the new `plan` adapters during initialization, update, or repair. Retain constrained synchronization tools as internal capabilities used by the runtime and skills.
