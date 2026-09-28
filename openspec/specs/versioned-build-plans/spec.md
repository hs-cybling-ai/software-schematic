# versioned-build-plans Specification

## Purpose

Define durable, semantically versioned implementation plans that turn a selected logical diagram boundary into a resumable and strictly scoped software build workflow.

## Requirements

### Requirement: Scope-relative semantic plan identity
Each build plan SHALL be identified by the selected scope's stable graph identity plus a semantic version and an opaque plan ID. The first plan for a scope SHALL use `1.0.0`; a later materially changed plan SHALL apply the developer's explicit major, minor, or fix/patch choice to that scope's latest version; and retrying the same unchanged scope and graph contract SHALL reuse the existing plan version. A semantic version SHALL NOT be treated as globally unique across unrelated scopes.

#### Scenario: First plan for a component
- **WHEN** the developer clicks Play for a scope that has no prior plan
- **THEN** the runtime creates an immutable plan identified as that scope at version `1.0.0`

#### Scenario: Developer selects a minor advance
- **WHEN** the scope already has version `1.2.3` and the developer selects a minor change for a materially changed graph contract
- **THEN** the runtime creates version `1.3.0` for that scope

#### Scenario: Unchanged build is retried
- **WHEN** Play selects the same scope against the same graph contract as an existing incomplete or failed plan
- **THEN** the runtime resumes or retries that plan without inventing a new semantic version

### Requirement: Equivalent visual and conversational plan creation
The runtime SHALL use one plan-creation operation for both the diagram Play action and the project-local `plan` skill. Play SHALL provide the exact browser selection; the `plan` skill SHALL provide natural-language scope intent and SHALL resolve it to one reachable diagram entity before invoking the same operation. Both entry points SHALL apply identical graph-publication, scope, semantic-version, persistence, and validation rules.

#### Scenario: Natural language identifies one node
- **WHEN** the developer invokes `plan` with a description that confidently resolves to one reachable node or pool
- **THEN** the skill confirms the human-readable scope and creates the same plan that Play would create with that entity selected

#### Scenario: Natural language is ambiguous
- **WHEN** the description plausibly matches multiple nodes, labels, processes, or compositions
- **THEN** the skill presents a bounded numbered candidate list and creates no plan until the developer selects one

#### Scenario: Unsaved browser edits exist
- **WHEN** the `plan` skill cannot prove that current browser edits are durably published
- **THEN** it creates no plan and directs the developer to save or finish the pending editor update before retrying

### Requirement: Durable plan, progress, and evidence records
The runtime SHALL persist each immutable build plan beneath `.ss/workflows/builds/<plan-id>/` and SHALL keep mutable progress and verification evidence separate from the immutable plan. A plan SHALL contain project identity, selected scope, semantic version, source-manifest and graph revisions, ordered work items, context references, dependency relationships, and declared implementation-contract outputs, but SHALL NOT contain credentials, chat transcripts, copied requirement corpora, or executable scripts.

#### Scenario: Host changes after Play
- **WHEN** Play creates a plan and the developer later invokes the build skill from Codex or Claude
- **THEN** the selected host can recover the same validated plan, progress, and prior evidence from the project runtime

#### Scenario: Plan content is inspected
- **WHEN** a developer inspects the persisted plan
- **THEN** the record is declarative, bounded, non-sensitive, and contains no generated shell, Python, Codex, or Claude executable payload

### Requirement: Selected-boundary target derivation
The runtime SHALL derive plan targets from the selected entity without traversing upward into an owning or parent service. Selecting a pool SHALL include its contained service boundary, downward-contained entities, and recursively reachable compositions. Selecting a node SHALL include that component, its contained entities, incident edges and events, connected items needed to understand or realize its behavior, and downward compositions reached through an in-scope composable node. Only entities marked `new` or `modify` SHALL become implementation work items; `open` and `locked` entities SHALL remain context-only.

#### Scenario: Pool selects a complete service
- **WHEN** Play is invoked with a participant or pool selected
- **THEN** the plan includes eligible `new` and `modify` entities within that participant and its downward compositions without including sibling or parent services

#### Scenario: Node selects one component
- **WHEN** Play is invoked with an eligible activity selected
- **THEN** the plan includes that component, eligible contained or connected implementation items, and its edge and event contracts without authorizing unrelated diagram branches

#### Scenario: Selected component owns a composition
- **WHEN** an in-scope subprocess or call activity marked `new` or `modify` links to a composed process
- **THEN** the plan may include eligible `new` and `modify` descendants in that composition while preserving the selected component as the scope boundary

#### Scenario: Connected entity is not implementation eligible
- **WHEN** a connected, contained, edge, or event entity is `open` or `locked`
- **THEN** it may be supplied as context but does not become a work item or authorize code changes for its feature

### Requirement: Working-software effects remain scope constrained
The build plan SHALL permit code, configuration, schema, migration, and test changes that are necessary to make authorized work items function in the repository's physical architecture even when those effects do not correspond one-to-one with logical diagram entities. The plan SHALL NOT authorize unrelated product capabilities, unmodeled replay behavior, unmodeled alternate flows, or new stubbed or duplicate services merely to satisfy a work item.

#### Scenario: Logical component uses shared physical infrastructure
- **WHEN** implementing an authorized component requires a bounded update to shared routing, dependency wiring, configuration, persistence, or tests
- **THEN** the build may make that supporting change and records it as an effect of the authorized work item

#### Scenario: Convenient adjacent feature is out of scope
- **WHEN** an implementation could add replay, another alternate path, or a duplicate service not authorized by a `new` or `modify` target
- **THEN** the build omits that feature and reports any resulting design gap rather than expanding the plan

### Requirement: Resumable ordered work queue
Each plan SHALL expose a deterministic dependency-aware queue whose mutable work-item states support pending, active, complete, failed, and stale outcomes. At most one work item SHALL be active for a plan unless a future protocol explicitly authorizes isolated parallel execution, and every state transition SHALL use revision preconditions so another host cannot silently duplicate or overwrite work. Active build authority SHALL be validated against the plan's selected scope contract rather than the project-wide graph revision, so edits to nodes outside the plan do not interrupt the build while changes to the captured scope make it stale.

#### Scenario: Build resumes after interruption
- **WHEN** a host stops after completing some work items
- **THEN** another invocation resumes with the next dependency-ready pending item and retains completed-item evidence

#### Scenario: Two hosts attempt the same item
- **WHEN** two clients try to claim one pending work item at the same plan revision
- **THEN** exactly one claim succeeds and the other receives current resumable state

#### Scenario: Developer documents a different function during a build
- **WHEN** a function build is ready or running and the developer edits and publishes a node that is not listed in that plan
- **THEN** the graph incorporates the edit and the existing function build remains resumable

#### Scenario: Developer changes the active build scope
- **WHEN** a ready or running build's selected scope contract changes
- **THEN** the runtime marks that plan stale before granting another claim, context read, contract publication, or completion

### Requirement: Implementation contract evidence
For every implemented edge or event in plan scope, completion SHALL publish a generated `docs/<element-id>-contract.md` document describing the behavior actually built. The generated contract SHALL remain distinct from logical `docs/<element-id>.md`, SHALL cite its plan and semantic version, and MAY differ where the logical design was incomplete or physically unworkable, but SHALL NOT introduce behavior outside the plan's authorized scope.

#### Scenario: Working transport differs from logical detail
- **WHEN** the in-scope implementation requires a workable payload, endpoint, acknowledgement, or error detail that differs from logical Markdown
- **THEN** the generated contract records the implemented behavior and divergence without overwriting the logical design document

#### Scenario: Implementation attempts to add scope
- **WHEN** proposed generated-contract content describes replay, an alternate flow, or another service not authorized by the plan
- **THEN** contract publication and plan completion are rejected as an out-of-scope expansion
