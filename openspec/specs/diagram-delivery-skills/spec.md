# Diagram Delivery Skills

## Purpose

Define the lightweight diagram-first workflow shared by a developer, the live Software Schematic UI, and project-local AI coding assistants.

## Requirements

### Requirement: Portable skill suite
The project SHALL provide small `design`, `plan`, and `build` skills for Codex and Claude Code. Each skill SHALL use the one project runtime and current saved diagram workflow, return actionable setup guidance when unavailable, and treat runtime-generated build plans as disposable execution state rather than copied specifications. Equivalent host adapters SHALL consume the same plan protocol and SHALL NOT generate provider-specific executable orchestration scripts. The managed user-facing `graph` skill SHALL be removed; graph synchronization remains an internal runtime capability used by plan creation.

#### Scenario: Skills run in a supported host
- **WHEN** a user invokes any delivery skill from Codex or Claude Code in an initialized project
- **THEN** the skill uses the same current project diagram, versioned plans, and runtime workflow independent of the host

#### Scenario: Required integration is unavailable
- **WHEN** a skill cannot reach the matching project runtime
- **THEN** it makes no diagram or code change and explains how to restart or repair the project integration

### Requirement: Interview-driven design skill
The `design` skill SHALL begin by opening or resuming the matching project design workspace, verifying project identity and delivery capabilities, and orienting the developer to the current visual model. It SHALL prefer a fresh browser selection or resumable interview target; otherwise it SHALL present bounded choices to refine the root business model, resume an existing named process, or design a new process. For a new process it SHALL begin with the desired outcome, actors, and boundary rather than requiring the developer to supply BPMN/CMMN terminology. It SHALL deeply interview the developer, maintain a bounded adaptive assessment of material design gaps, turn each settled slice into a small typed proposal in the live browser, observe approved and manual diagram changes, and continue until the developer confirms that the model is complete and clicks Play. It SHALL NOT expose workspace credentials, approve its own proposal, or begin implementation before Play.

#### Scenario: Design starts without an open browser
- **WHEN** the developer invokes the `design` skill for an initialized project whose runtime is headless
- **THEN** the skill invokes `open_design_workspace`, directs the developer to the securely opened modeling workspace, and performs no proposal mutation until current workspace context is available

#### Scenario: Active visual context is available
- **WHEN** a fresh browser session identifies an active diagram or selected model element
- **THEN** the skill offers to continue from that context before presenting unrelated process choices

#### Scenario: Existing model needs orientation
- **WHEN** no active selection or resumable interview determines the target and the model contains named processes
- **THEN** the skill presents a bounded, human-readable choice among the root business model, existing process candidates, and a new process without requiring diagram paths or entity IDs

#### Scenario: Developer begins a new process
- **WHEN** the developer chooses to design a new process
- **THEN** the skill first establishes the intended outcome, actors, and system boundary and then proposes the business-need anchor and BPMN composition incrementally through supported operations

#### Scenario: Interview produces a diagram increment
- **WHEN** the developer's answers settle one coherent portion of a process
- **THEN** the skill saves the decisions, submits a focused visual proposal, waits for the browser decision, and continues from the resulting current saved diagram

#### Scenario: Design contract remains ambiguous
- **WHEN** two materially different process interpretations remain plausible
- **THEN** the skill asks a focused follow-up question instead of inventing diagram structure

#### Scenario: Approved design becomes stale
- **WHEN** an affected document changes after proposal generation and before application
- **THEN** the skill refuses the stale proposal and regenerates it from the current diagram

#### Scenario: Material design areas remain unresolved
- **WHEN** the current model still lacks consequential decisions about paths, recovery, data or state, integrations, trust, operations, exclusions, documentation, or implementation status
- **THEN** the skill explains the relevant gaps and continues with focused questions instead of declaring the model complete

#### Scenario: Developer confirms the model is ready
- **WHEN** no material known gap remains and the developer confirms that the model feels complete
- **THEN** the skill summarizes the modeled scope, asks the developer to click Play in the diagram UI, and waits for the resulting build request without starting implementation

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
