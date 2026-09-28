## ADDED Requirements

### Requirement: Provider-neutral interview and suggestion modes
The assistant service SHALL expose provider-neutral conversational and structured-suggestion modes over the same bounded invocation context. Conversational mode SHALL accept the ordered turns from the current invocation and return assistant prose without diagram operations. Structured-suggestion mode SHALL accept that same invocation transcript and a current revision-bound snapshot and SHALL return only the versioned structured plan required by the operation schema. Provider adapters SHALL disable tools and external mutation authority in both modes, and adding a provider SHALL NOT require provider-specific browser behavior.

#### Scenario: User continues the interview
- **WHEN** the user submits a message while the invocation is in interview mode
- **THEN** the provider returns a conversational answer or focused follow-up with no structured diagram operations

#### Scenario: User requests suggested updates
- **WHEN** the user explicitly asks the current invocation to suggest updates from its context
- **THEN** the provider receives the ordered current-invocation transcript and latest scoped snapshot and returns a correlated structured plan

#### Scenario: Conversation provider emits operations
- **WHEN** a provider returns structured mutations during conversational mode
- **THEN** the service rejects or ignores those mutations and exposes no proposal for approval

### Requirement: Acute element-scoped mutations
An element-scoped structured proposal SHALL treat the invoked element as the only mutable element in the active parent diagram. It SHALL permit changes to that element's properties and owned Markdown and MAY create or update a composition directly owned or referenced by that element. It SHALL NOT mutate unrelated peer elements in the parent diagram. A diagram-scoped proposal SHALL be required for changes spanning unrelated parent-diagram elements.

#### Scenario: Node interview proposes a peer change
- **WHEN** a node-scoped structured plan attempts to rename, move, connect, disconnect, document, or remove an unrelated peer element in the active diagram
- **THEN** validation rejects the complete proposal and directs the user to invoke diagram-level assistance for multi-element changes

#### Scenario: Node becomes a subprocess
- **WHEN** a node-scoped interview concludes that the invoked node should reference a subprocess and the structured plan updates that node and populates its directly owned composition
- **THEN** validation permits the selected-node and child-composition operations without broadening mutation authority to peer nodes in the parent diagram

#### Scenario: Diagram assistance coordinates multiple nodes
- **WHEN** a diagram-scoped structured plan changes multiple eligible elements in the active diagram
- **THEN** validation evaluates those operations under the complete-diagram scope

## MODIFIED Requirements

### Requirement: Scope-aware assistant prompting
The application SHALL construct one versioned, size-bounded context snapshot when a magic-button invocation opens, using the latest persisted content for that invocation scope. Every normalized diagram element in assistant context SHALL include ID, Type, Label, the single complete Name, Implementation Status, and Documentation when included by scope. Context SHALL NOT contain separate local Name, qualified symbol, composition path, or Documentation-path fields. A node-scoped snapshot SHALL additionally identify the primary node and include its Markdown; a supported edge-scoped snapshot SHALL identify the primary edge and include its Markdown; a diagram-scoped snapshot SHALL treat the complete active diagram as modifiable scope without selecting a primary element. The application SHALL flush pending relevant edits before creating the opening snapshot, SHALL disclose when optional context was truncated, and SHALL keep the snapshot and invocation transcript only for the lifetime of that open assistant invocation. Before generating structured suggestions, the application SHALL refresh the persisted scoped snapshot and revision while retaining only the turns from the current invocation.

#### Scenario: Node-scoped invocation starts
- **WHEN** the user activates a node magic action
- **THEN** a new invocation starts from the latest persisted active diagram context plus the selected node identity, status, and Markdown as the primary target, with no turns or draft proposal from any earlier invocation

#### Scenario: Node-scoped request is submitted
- **WHEN** the user submits an interview turn or explicitly requests suggested updates from a node magic action
- **THEN** the provider request contains the current invocation transcript and active diagram context plus the selected node identity, status, and Markdown as the primary target context

#### Scenario: Diagram-scoped invocation starts
- **WHEN** the user activates the diagram magic action
- **THEN** a new invocation starts from the latest persisted complete active diagram and diagram Markdown with no primary-element restriction and no turns or draft proposal from any earlier invocation

#### Scenario: Diagram-scoped request is submitted
- **WHEN** the user submits an interview turn or explicitly requests suggested updates from the diagram magic action
- **THEN** the provider request contains the current invocation transcript and complete active diagram and diagram Markdown with no primary-element restriction

#### Scenario: Supported edge-scoped invocation starts
- **WHEN** an assistant entry point invokes assistance for an eligible edge
- **THEN** a new invocation starts from the latest persisted connected diagram context plus that edge's identity, endpoints, status, and Markdown as the primary target

#### Scenario: Pending content exists when assistance opens
- **WHEN** the active diagram or relevant Markdown has pending local edits at invocation time
- **THEN** the application flushes those edits and derives the opening snapshot revision from the resulting persisted state

#### Scenario: Pending content exists
- **WHEN** the active diagram or relevant Markdown has pending local edits at invocation time or structured-suggestion time
- **THEN** the application flushes those edits and derives the applicable snapshot revision from the resulting current persisted state

#### Scenario: Durable scope changes during the interview
- **WHEN** persisted in-scope content changes after the invocation opens and the user requests suggested updates
- **THEN** the application refreshes the scoped snapshot, binds the structured request to its current revision, and retains only compatible turns from the current open invocation

#### Scenario: Required context exceeds limits
- **WHEN** the structural context required to interpret the active diagram cannot fit within configured request limits
- **THEN** the application refuses the invocation and explains the exceeded limit rather than silently omitting required structure

### Requirement: Proposal preview and explicit approval
The application SHALL show every valid structured proposal inline in the conversation transcript as a human-readable, outlined `Suggested updates` section grouped by affected diagram and documentation file. The section SHALL identify replacements, additions, connections, composition creation or reuse, Markdown changes, assumptions, and warnings. It SHALL contain a prominent `Approve changes` action and a secondary `Continue interview` action, while dialog close remains available without applying changes. Continuing the interview SHALL invalidate the displayed proposal and return focus to the anchored conversation composer; closing SHALL discard the invocation; neither action SHALL mutate project content.

#### Scenario: Valid proposal is previewed
- **WHEN** a provider returns a plan that passes validation
- **THEN** the dialog appends an outlined `Suggested updates` section containing its summary, complete grouped semantic change list, assumptions, warnings, and approval controls without mutating the project

#### Scenario: User continues from a proposal
- **WHEN** the user chooses to continue the interview from a displayed proposal
- **THEN** the proposal card becomes non-approvable, its summary remains available as current-invocation conversation context, and focus returns to the blank bottom composer for another conversational turn

#### Scenario: User closes a proposal
- **WHEN** the user closes the dialog while a proposal is displayed
- **THEN** the proposal and invocation transcript are discarded and all diagrams, documentation, tabs, and files remain unchanged

#### Scenario: User rejects a proposal
- **WHEN** the user rejects the displayed candidate by continuing the interview or closes the proposal preview
- **THEN** the candidate is no longer approvable and all diagrams, documentation, tabs, and files remain unchanged

#### Scenario: User approves a current proposal
- **WHEN** the user explicitly approves a valid proposal whose source revision remains current
- **THEN** the application begins coordinated application of exactly the previewed operations

### Requirement: Deep interview with early visualization
The design workflow SHALL interview the developer until actors, goals, boundaries, important data, states, integrations, trust, happy paths, and failure behavior are coherent. During the interview phase it SHALL discuss and describe candidate diagram updates without emitting an approvable mutation plan. It SHALL translate the settled current-invocation context into structured suggested updates only after the user explicitly requests that transition.

#### Scenario: A design slice becomes clear
- **WHEN** the developer has answered enough questions to settle one part of the model but has not requested suggested updates
- **THEN** the AI describes the intended model or asks the next focused question without creating a structured proposal

#### Scenario: User requests visualization
- **WHEN** the developer instructs the assistant to suggest updates from the current invocation context
- **THEN** the AI produces a small typed proposal for preview and approval

#### Scenario: A material decision is unresolved
- **WHEN** the answer would change diagram structure or an implementation contract
- **THEN** the AI asks a focused follow-up rather than inventing a default

### Requirement: In-editor conversational diagram interview
The diagram tool SHALL provide a chat-based design interview that lets a user discuss the invocation-scoped diagram, primary element, and relevant documentation over multiple turns. Each open invocation SHALL retain its conversation decisions and revision-bound diagram context, SHALL distinguish questions and explanations from proposed mutations, and SHALL create no structured change set until the user explicitly requests suggested updates. Closing the dialog or invoking any magic action later SHALL discard the prior invocation state and start a new interview from current persisted scope.

#### Scenario: User refines a diagram conversationally
- **WHEN** the user discusses a process over several turns and answers follow-up questions within one open invocation
- **THEN** the interview retains the established decisions, asks only for unresolved material details, and waits for an explicit request before producing a structured change set

#### Scenario: User asks for an explanation only
- **WHEN** the user's message asks what an existing node or flow means without requesting suggested updates
- **THEN** the assistant answers from the invocation's diagram and Markdown context and does not create a mutation proposal

#### Scenario: User reopens the same magic action
- **WHEN** the user closes an interview and later invokes the same node or diagram magic action again
- **THEN** the new interview contains no prior transcript, draft answer, or proposal and is seeded only from the latest persisted scope
