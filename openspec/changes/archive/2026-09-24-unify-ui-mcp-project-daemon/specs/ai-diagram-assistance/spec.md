## ADDED Requirements

### Requirement: Deep interview with early visualization
The design workflow SHALL interview the developer until actors, goals, boundaries, important data, states, integrations, trust, happy paths, and failure behavior are coherent. It SHALL turn settled slices into small diagram proposals during the interview instead of waiting for one final large proposal.

#### Scenario: A design slice becomes clear
- **WHEN** the developer has answered enough questions to settle one part of the model
- **THEN** the AI submits a small typed proposal and continues the interview from the updated diagram

#### Scenario: A material decision is unresolved
- **WHEN** the answer would change diagram structure or an implementation contract
- **THEN** the AI asks a focused follow-up rather than inventing a default

### Requirement: Vibe diagram iteration
Agent proposals SHALL appear in the live browser without copied identifiers. The developer SHALL be able to approve, reject, manually adjust, and continue the same interview from the current durable diagram revision.

#### Scenario: Developer manually reshapes the model
- **WHEN** the developer changes the diagram between AI proposals
- **THEN** the next interview turn observes current revisions and treats the adjustment as design input

### Requirement: Play-to-build transition
The diagram UI SHALL expose a prominent Play action. Play SHALL flush current edits and create a revision-bound build request for the active diagram and optional selected element. It SHALL NOT itself edit source code.

#### Scenario: Developer clicks Play
- **WHEN** the current diagram is saved and published
- **THEN** the waiting AI receives the build request, resolves eligible implementation scope, and begins the build workflow

#### Scenario: Diagram changes after Play
- **WHEN** the model revision advances before the build completes
- **THEN** the prior build request becomes stale and the UI returns to designing until the developer clicks Play again

### Requirement: Build outcome feedback
The runtime SHALL expose designing, ready, building, complete, and failed states for the current Play request so the UI and AI share one understandable handoff.

#### Scenario: AI completes implementation
- **WHEN** the build workflow records checks and changed paths against the current request
- **THEN** the browser shows the build as complete and keeps the diagram available for another iteration
