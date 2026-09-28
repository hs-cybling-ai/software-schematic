## MODIFIED Requirements

### Requirement: Portable skill suite
The project SHALL provide small `design`, `graph`, and `build` skills for Codex and Claude Code. Each skill SHALL use the one project runtime and current saved diagram workflow, return actionable setup guidance when unavailable, and SHALL NOT require a separate handoff store or copied specification.

#### Scenario: Skills run in a supported host
- **WHEN** a user invokes any delivery skill from Codex or Claude Code in an initialized project
- **THEN** the skill uses the same current project diagram and runtime workflow independent of the host

#### Scenario: Required integration is unavailable
- **WHEN** a skill cannot reach the matching project runtime
- **THEN** it makes no diagram or code change and explains how to restart or repair the project integration

### Requirement: Interview-driven design skill
The `design` skill SHALL deeply interview the developer, turn each settled slice into a small typed proposal in the live browser, observe approved and manual diagram changes, and continue until the developer clicks Play. It SHALL NOT begin implementation before Play.

#### Scenario: Interview produces a diagram increment
- **WHEN** the developer's answers settle one coherent portion of the model
- **THEN** the skill submits a focused visual proposal and continues from the resulting saved diagram

#### Scenario: Design contract remains ambiguous
- **WHEN** two materially different process interpretations remain plausible
- **THEN** the skill asks a focused follow-up question instead of inventing diagram structure

#### Scenario: Approved design becomes stale
- **WHEN** an affected document changes after proposal generation and before application
- **THEN** the skill refuses the stale proposal and regenerates it from the current diagram

### Requirement: Changed-document graph skill
The `graph` skill SHALL publish current saved diagram and Markdown changes through the project runtime and wait for the resulting graph revision. It SHALL submit only changed canonical document identities and SHALL permit a safe complete rebuild when bounded refresh cannot be proven.

#### Scenario: One annotation changes
- **WHEN** one node Markdown document has a new durable content revision and its owner is unchanged
- **THEN** the skill requests synchronization for that document and returns the graph revision that includes it

#### Scenario: No authoritative content changed
- **WHEN** every current document digest matches the active source manifest
- **THEN** the skill reports the graph current without starting a rebuild

#### Scenario: Structural impact is uncertain
- **WHEN** a rename, deletion, composition change, or dependency ambiguity prevents bounded invalidation
- **THEN** the runtime uses a safe complete rebuild and the skill reports the fallback

### Requirement: Natural-language node-scoped build skill
The `build` skill SHALL wait for the developer's current Play request, resolve its active diagram or selected element to eligible `new` or `modify` entities, start the build against that graph revision, implement and verify code, and report changed paths and checks to the UI.

#### Scenario: One node clearly matches
- **WHEN** Play identifies one eligible selected model element
- **THEN** the skill starts the build and implements its revision-bound annotated scope

#### Scenario: Multiple nodes plausibly match
- **WHEN** the active diagram resolves to multiple plausible eligible roots
- **THEN** the skill presents bounded candidates and makes no source change until the developer selects the intended scope

#### Scenario: Contract changes during implementation
- **WHEN** the active diagram revision changes before completion
- **THEN** the skill stops further writes, marks the request stale, and waits for the developer to click Play again

## REMOVED Requirements

### Requirement: Seamless skill handoff
**Reason**: The one project runtime and current Play request now carry workflow state directly; a separate handoff record adds friction and duplicate state.

**Migration**: Use the current interview, proposal, graph receipt, and Play request owned by the project runtime.
