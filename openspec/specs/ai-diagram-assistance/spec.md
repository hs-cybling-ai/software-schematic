# AI Diagram Assistance

## Purpose

Provide safe, scoped AI proposals for coordinated Software Schematic diagram and documentation changes.

## Requirements

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

### Requirement: Provider-neutral structured proposals
The Rust server SHALL expose a confined assistant-proposal operation backed by a provider-neutral interface and SHALL require the selected provider to return a versioned structured plan containing a summary, assumptions or warnings, and ordered operations from an allowed schema. The first implementation SHALL include an OpenAI Responses API adapter and a deterministic fake provider for tests; adding another provider SHALL NOT require changes to the browser dialog, context schema, plan schema, validators, or executor. The server SHALL reject arbitrary code, shell commands, unrestricted filesystem paths, raw patches, and unsupported operations.

#### Scenario: Provider returns a valid plan
- **WHEN** a configured provider returns a plan conforming to the current operation schema
- **THEN** the server returns the correlated proposal and usage metadata to the browser without applying it

#### Scenario: Provider returns invalid output
- **WHEN** provider output is malformed, exceeds limits, references an unsupported schema version, or contains an unallowed operation
- **THEN** the server rejects it with a redacted actionable error and no editor or file state changes

#### Scenario: Provider request is cancelled
- **WHEN** the user cancels an in-progress request
- **THEN** the server cancels or abandons the provider operation, releases request resources, and does not return a proposal for application

#### Scenario: Provider is not configured
- **WHEN** the user submits a request without a usable provider and credential configuration
- **THEN** the dialog explains how to configure assistance without exposing or requesting a credential in browser content

### Requirement: Allowed diagram operation schema
The initial operation schema SHALL support replacing an eligible node's BPMN type, changing an eligible node label or complete Name, assigning or preserving a named process reference, creating or opening a composition from a complete process Name, adding supported flow nodes, connecting supported nodes with sequence flows, and replacing diagram or node Markdown. Every operation SHALL use stable element IDs and complete process or member Names and SHALL contain no executable content. Providers SHALL NOT supply composition or Documentation paths. Existing ID, Label, and Name values SHALL be preserved unless an explicit operation changes them.

#### Scenario: Task becomes a documented subprocess
- **WHEN** a valid proposal replaces a task, links it to a new composition, adds four ordered steps and required start/end flow in that composition, and updates parent, diagram, or node Markdown
- **THEN** the plan expresses every change using only allowed typed operations with stable IDs and complete Names

#### Scenario: Provider creates a reusable process
- **WHEN** a provider proposes process Name `sales.checkout.PlaceOrder`
- **THEN** SSW derives the composition folder from that Name

#### Scenario: Provider supplies a path
- **WHEN** a provider attempts to choose a composition or Documentation path
- **THEN** validation rejects the proposal

#### Scenario: Unsupported mutation is proposed
- **WHEN** a plan requests arbitrary BPMN XML replacement, JavaScript execution, a shell command, or an operation outside the registered schema
- **THEN** validation rejects the complete plan before preview or application

### Requirement: Independent proposal validation
The application SHALL treat provider output as untrusted and SHALL validate the complete plan before preview and again before application. Validation SHALL confirm schema and request correlation, current source revision, existing references, unique created IDs, permitted BPMN replacements and connections, valid complete Names, configured size and operation limits, and Implementation Status constraints. An operation SHALL NOT modify, replace, delete, relabel, or reconfigure a `locked` element. `new` and `modify` SHALL be supplied as positive authoring hints but SHALL NOT bypass validation.

#### Scenario: Locked node would change
- **WHEN** any operation would change a node whose live status is `locked`
- **THEN** the application rejects the complete proposal, identifies the protected node, and makes no changes

#### Scenario: Proposal becomes stale
- **WHEN** the diagram or relevant Markdown revision changes after proposal generation and before approval
- **THEN** approval is refused and the user is asked to regenerate the proposal from current context

#### Scenario: Created identifier conflicts
- **WHEN** two proposed elements share an ID or a proposed ID already belongs to an incompatible existing element
- **THEN** validation rejects the complete proposal and identifies the conflict

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

### Requirement: Coordinated application and recovery
The application SHALL validate and stage all approved diagram, composition, connection, linkage, and Markdown operations before reporting success; SHALL apply live diagram mutations through supported `bpmn-js` services; SHALL use existing confined server operations for composition and file handling; SHALL preserve complete before-state for every affected diagram and Markdown document; and SHALL integrate resulting writes with automatic persistence. If any operation cannot be staged or applied, the application SHALL restore all affected content and SHALL NOT report partial success. Successful application SHALL provide coherent per-diagram undo where supported and one assistant-level revert action for the coordinated change.

#### Scenario: Cross-diagram proposal succeeds
- **WHEN** an approved plan replaces a parent task, creates and populates a child composition, and updates related Markdown successfully
- **THEN** every affected diagram and document reflects the previewed state, opens through canonical tab behavior as needed, and enters existing persistence handling

#### Scenario: Operation fails during staging
- **WHEN** any approved operation fails before the coordinated change commits
- **THEN** the application restores every affected diagram and Markdown document to its captured before-state and reports the failing operation

#### Scenario: User reverts an applied proposal
- **WHEN** the user invokes assistant-level revert for the most recently applied coordinated proposal and no conflicting later revision prevents it
- **THEN** the application restores the captured before-state across all affected diagrams and documentation

### Requirement: Provider credential and request security
Provider credentials SHALL remain in the Rust host environment or an approved OS credential store and SHALL NOT be embedded in browser assets, `.ss` web files, diagrams, Markdown, request previews, logs, or project metadata responses. The assistant endpoint SHALL enforce outbound-provider allowlisting, HTTPS, timeouts, cancellation, concurrency, request/response limits, and redacted errors. Before transmission, the dialog SHALL identify the selected provider and the categories of project context that will be sent.

#### Scenario: Assistant request is reviewed before transmission
- **WHEN** the user prepares to submit an assistant prompt
- **THEN** the dialog identifies the provider and states that active diagram structure and relevant Markdown will be transmitted

#### Scenario: Credential error occurs
- **WHEN** the provider rejects or cannot authenticate the configured credential
- **THEN** the application reports a redacted configuration error without returning the credential or provider response body to browser logs

#### Scenario: Provider target is not allowed
- **WHEN** configuration attempts to send assistant context to a non-allowlisted or non-HTTPS endpoint
- **THEN** the Rust server rejects the request before transmitting project context

### Requirement: Local account authentication
The project wrapper SHALL provide `auth login`, `auth status`, and `auth logout` commands for supported locally installed Codex and Claude Code CLIs. Login SHALL delegate to the provider's official authentication flow and SHALL NOT collect or persist passwords, MFA codes, session cookies, or access tokens. The project SHALL persist only its selected provider. Local proposal generation SHALL disable agent tools, use read-only or plan permissions, avoid conversation persistence, require the canonical structured operation plan, and retain the normal SSW validation and approval boundary.

#### Scenario: User configures an installed local agent
- **WHEN** the user runs `ssw auth login` and completes the official provider sign-in
- **THEN** the project selects that provider and subsequent assistant proposals use its cached account authentication without a project API key

#### Scenario: No supported local agent is installed
- **WHEN** the user runs `ssw auth login` without Codex or Claude Code available
- **THEN** the command explains that a supported CLI must be installed and stores no configuration

#### Scenario: User signs out
- **WHEN** the user runs `ssw auth logout`
- **THEN** SSW delegates logout to the selected provider and removes the project provider selection

### Requirement: CMMN assistant context
For a CMMN diagram- or node-scoped magic request, the application SHALL construct the existing bounded context shape using normalized CMMN elements and SHALL identify the CMMN content as business-need context. Each included element SHALL provide ID, CMMN Type, Label, complete Name, Implementation Status, and Documentation when in scope, and the request SHALL identify the owning package and `.cmmn` diagram without exposing an absolute project path.

#### Scenario: CMMN diagram request is prepared
- **WHEN** a user submits a prompt from the magic action on `cybling/sdk/main.cmmn`
- **THEN** the provider receives a bounded normalized CMMN snapshot and package identity rather than raw filesystem authority

### Requirement: Bounded CMMN operations
The assistant operation schema SHALL support only registered CMMN documentation-tool operations: changing an eligible Label or complete Name, replacing diagram or element Markdown, adding a supported CMMN plan item, connecting supported elements, and assigning or opening a Process Task BPMN-process link. It SHALL reject raw XML replacement, arbitrary paths, project-structure inference, executable content, runtime case behavior, and unsupported CMMN semantics.

#### Scenario: Assistant links a Process Task
- **WHEN** a valid proposal assigns BPMN process Name `cybling.sdk.Birth` to a CMMN Process Task
- **THEN** the preview identifies the derived BPMN composition and the provider supplies no filesystem path

#### Scenario: One need anchors a descendant package design
- **WHEN** a proposal links a Process Task in CMMN package `cybling` to BPMN process `cybling.sdk.Birth`
- **THEN** validation accepts the complete BPMN Name without requiring `cybling.sdk/main.cmmn`

#### Scenario: Assistant proposes enterprise behavior
- **WHEN** a proposal requests runtime case execution, compensation, deployment, permissions, or arbitrary expressions
- **THEN** validation rejects the operation before preview or mutation

### Requirement: CMMN proposal application
Approved CMMN proposals SHALL use supported `cmmn-js` modeling services and the existing confined file operations, automatic persistence, before-state capture, rollback, and assistant-level revert behavior. A failed mixed CMMN, BPMN, and Markdown proposal SHALL NOT report partial success.

#### Scenario: Approved CMMN documentation proposal succeeds
- **WHEN** an approved proposal adds a supported CMMN plan item and updates its Documentation successfully
- **THEN** the CMMN XML and Markdown save through their existing paths and the complete change can be reverted

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

### Requirement: Vibe diagram iteration
Agent proposals SHALL appear in the live browser without copied identifiers. The developer SHALL be able to approve, reject, manually adjust, and continue the same interview from the current durable diagram revision.

#### Scenario: Developer manually reshapes the model
- **WHEN** the developer changes the diagram between AI proposals
- **THEN** the next interview turn observes current revisions and treats the adjustment as design input

### Requirement: Play-to-build transition
The diagram UI SHALL expose a prominent Play action. Play SHALL flush all pending relevant diagram and Markdown edits, wait for their graph publication, resolve the active diagram selection to a bounded build scope, and create or resume an immutable semantic-versioned plan for the exact source-manifest and graph revisions. The UI SHALL allow an explicit major, minor, or fix/patch advance when the selected scope has a materially changed prior plan, SHALL publish the created scope label and version after success, and SHALL NOT itself edit source code.

#### Scenario: Developer clicks Play
- **WHEN** the current selected scope is saved and published
- **THEN** the runtime creates or resumes its versioned build plan and the waiting AI can discover the authorized work

#### Scenario: Developer clicks Play for a new scope
- **WHEN** current relevant edits are saved and published and the selected scope has no prior plan
- **THEN** the runtime creates version `1.0.0`, the waiting AI can discover it, and the UI displays the scope and version

#### Scenario: Developer versions a changed scope
- **WHEN** a prior version exists and the selected graph contract has materially changed
- **THEN** Play requires a major, minor, or fix/patch choice and publishes the resulting version only after the new plan is durably created

#### Scenario: Diagram changes after Play
- **WHEN** an unrelated authoritative model revision advances before the build completes
- **THEN** the prior plan becomes stale and the UI directs the developer to review the design and click Play again

#### Scenario: Generated completion contracts publish
- **WHEN** the build publishes only the implementation-contract outputs declared by its plan
- **THEN** the UI records the resulting completion graph revision without treating the plan as stale from its own expected output

### Requirement: Discoverable plan status in the editor
The diagram UI SHALL show the selected scope's current plan version and lifecycle state after Play and after subsequent runtime updates. The display SHALL distinguish ready, building, complete, failed, and stale plans and SHALL remain a status surface rather than an implementation executor.

#### Scenario: Developer returns while another host builds
- **WHEN** Codex or Claude advances a selected plan
- **THEN** the editor displays that plan's current semantic version and lifecycle state

### Requirement: Visible implementation-contract drift
When an edge or event has both logical Markdown and generated implementation-contract Markdown with different normalized content, the diagram UI SHALL indicate that implementation drift exists and SHALL let the developer inspect both documents without overwriting either one. Drift alone SHALL NOT make an otherwise authorized build fail or silently change design intent.

#### Scenario: Built contract differs from logical design
- **WHEN** the current graph reports different logical and implementation contracts for a selected edge or event
- **THEN** the editor shows a drift warning and distinct logical and implemented contract views

#### Scenario: Divergence was required for working code
- **WHEN** the generated contract remains inside plan scope but records a workable implementation detail absent from or different from logical design
- **THEN** the completed plan remains complete while the UI preserves the drift warning for later design review

### Requirement: Build outcome feedback
The runtime SHALL expose designing, ready, building, complete, and failed states for the current Play request so the UI and AI share one understandable handoff.

#### Scenario: AI completes implementation
- **WHEN** the build workflow records checks and changed paths against the current request
- **THEN** the browser shows the build as complete and keeps the diagram available for another iteration

### Requirement: Shared conversational design protocol
The web assistant and external `design` skill SHALL use the same versioned context, typed proposal, validation, preview, approval, application, and revert contracts. Either surface SHALL be able to continue an interview begun on the other after validating project identity and source revisions, and neither surface SHALL receive privileged mutation semantics unavailable to the other.

#### Scenario: Interview continues in the editor
- **WHEN** a chat-originated design proposal is opened in the diagram UI
- **THEN** the UI renders the same semantic proposal and current revision state and can approve, reject, or continue the interview

#### Scenario: UI change precedes chat approval
- **WHEN** a user edits an affected diagram in the UI after a chat proposal was generated
- **THEN** the shared validator rejects the stale proposal and identifies the documents requiring regeneration or reconciliation

### Requirement: Interview completeness and annotation quality
Conversational design SHALL evaluate each in-scope process for actors, triggers, outcomes, happy path, alternatives, failures, data or message contracts, system boundaries, dependencies, non-functional constraints, acceptance evidence, and implementation status. It SHALL record applicable answers as concise Markdown owned by the relevant diagram, node, or edge and SHALL explicitly identify unresolved decisions rather than silently omitting them.

#### Scenario: Process lacks failure behavior
- **WHEN** the interview defines a happy path but no failure outcome for an external call
- **THEN** the assistant asks for failure behavior or records an explicitly approved unresolved decision before declaring the design ready to build

#### Scenario: Answer applies to one flow
- **WHEN** an answer constrains a specific transition rather than the whole process
- **THEN** the resulting annotation is attached to that edge instead of being duplicated in diagram-level prose

### Requirement: Complete structured diagram change sets
The shared design protocol SHALL translate open-ended discussion into a versioned structured diagram change set whose operations can create, update, replace, move, connect, disconnect, or remove supported BPMN and CMMN elements and can replace diagram-, node-, or edge-owned Markdown. Every operation SHALL carry stable target or temporary identities, required semantic properties, expected document revisions, and enough information for deterministic preview, validation, application, and reversal; it SHALL contain no executable code or unrestricted file path.

#### Scenario: Open discussion produces concrete changes
- **WHEN** the interview establishes a new activity, two transitions, an exception path, and acceptance criteria
- **THEN** the design result contains typed element, connection, and annotation operations rather than prose instructions for manually editing the diagram

#### Scenario: Discussion requests an unsupported mutation
- **WHEN** the desired change cannot be represented by the current diagram operation registry
- **THEN** the design skill identifies the unsupported capability and does not emit an approximate or raw-XML operation

### Requirement: Discoverable diagram operation coverage
The diagram tool SHALL expose a versioned registry of supported operations by diagram type and element type, including property constraints and whether each operation supports preview, application, undo, and coordinated rollback. The design skill SHALL constrain structured output to that registry, and conformance tests SHALL prove that every advertised operation is accepted by validation and executed through supported modeler services.

#### Scenario: Design begins against a compatible editor
- **WHEN** the design skill discovers the editor operation registry
- **THEN** it uses only operations advertised for the active BPMN or CMMN scope and records the registry version with the proposal

#### Scenario: Advertised operation lacks an executor
- **WHEN** conformance testing finds an advertised operation without equivalent preview, application, or reversal behavior
- **THEN** the compatibility gate fails and that operation is not available to design output

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

### Requirement: Conversational bulk naming and detailed annotation
The conversational design workflow SHALL support bulk updates that assign concise operation-style Labels or Names to selected or in-scope nodes while moving behavioral detail into each node's Markdown. It SHALL also support describing one named function or activity and replacing or extending the Markdown owned by that exact node without requiring unrelated diagram mutations.

#### Scenario: Simplify every node name
- **WHEN** the user asks to update each node with a simple operation name and place the details in Markdown
- **THEN** the structured change set contains explicit per-node naming operations and per-node Markdown operations, preserves stable IDs, and previews every affected node

#### Scenario: Document save data
- **WHEN** the user asks what the `save data` function does and requests that its node be documented
- **THEN** the assistant resolves the intended node, asks for selection if ambiguous, and proposes Markdown for that node using available diagram context and confirmed details

### Requirement: Conversational external-system collaboration modeling
The conversational design workflow SHALL support adding BPMN participants or pools for external systems and adding message flows between those participants and the activities that call them. Structured operations SHALL distinguish participants, process-owned flow nodes, sequence flows, and cross-participant message flows and SHALL validate BPMN collaboration semantics before preview or application.

#### Scenario: Add an S3 external-system pool
- **WHEN** the user asks to add an S3 pool and connect every activity that calls S3
- **THEN** the assistant identifies or asks the user to confirm the S3-calling activities and proposes one external-system participant plus explicit message flows between each confirmed activity and that participant

#### Scenario: S3 callers cannot be determined confidently
- **WHEN** diagram annotations do not identify which activities call S3 and semantic matches are ambiguous
- **THEN** the assistant asks a focused clarification question and does not connect guessed activities

### Requirement: Endpoint-aware system-flow interview
When designing a BPMN process that crosses service or system boundaries, the interview SHALL ask for the participating systems and relevant service endpoints, including operation or method, logical route or topic, caller, receiver, request and response purpose, authentication or trust boundary when material, success outcome, timeout or retry behavior, and modeled failure path. It SHALL place endpoint detail in the Markdown owned by the calling activity, receiving activity, or message flow while keeping diagram Labels concise.

#### Scenario: Service call lacks an endpoint contract
- **WHEN** the user identifies a service-to-service call but the endpoint or message contract is not documented
- **THEN** the interview asks focused endpoint questions before declaring that part of the BPMN design build-ready

#### Scenario: Endpoint details are confirmed
- **WHEN** the user confirms that an activity calls `PUT /objects/{key}` on an S3-compatible service
- **THEN** the proposal keeps a concise operation-style activity Label, records the endpoint and behavioral details in owned Markdown, and represents the cross-system interaction with a message flow

### Requirement: Discrete diagrams with progressive composition
The interview SHALL organize BPMN diagrams around discrete cohesive functionality and SHALL use named compositions to reveal additional levels of detail as a user traverses call activities or reusable subprocesses. It SHALL recommend extraction when a flow contains a separately meaningful capability, crosses abstraction levels, or becomes difficult to review, while preserving the parent diagram's end-to-end intent and explicit linkage to the child composition.

#### Scenario: One activity contains a detailed subflow
- **WHEN** an activity expands into multiple endpoint calls, decisions, retries, or compensating steps that form a cohesive capability
- **THEN** the assistant proposes a named child composition, keeps one concise parent activity, and places the detailed flow in the child BPMN diagram

#### Scenario: Flow is already discrete and readable
- **WHEN** the active diagram describes one cohesive function at a consistent level of detail
- **THEN** the interview does not create composition solely to reduce node count

#### Scenario: User traverses into detail
- **WHEN** the user opens a composed activity during the interview
- **THEN** the child diagram becomes the active conversational scope while retaining navigable context back to its parent process

### Requirement: Readable labels and Markdown-owned contracts
The design workflow SHALL keep diagram Labels concise, meaningful, and understandable to a reader scanning the process while storing detailed behavioral and contract information in the Markdown owned by the relevant activity, event, or edge. It SHALL NOT attempt to encode complete procedures, payload schemas, endpoint contracts, validation rules, or error semantics in diagram Labels.

#### Scenario: Activity contains multiple internal steps
- **WHEN** an activity performs four ordered steps but remains one cohesive process phase
- **THEN** the activity receives one meaningful phase Label and its Markdown enumerates the four steps in order instead of listing them in the Label

#### Scenario: Activity represents one operation
- **WHEN** an activity performs a service or data operation
- **THEN** its Label communicates the process phase or intent and its Markdown records operation details such as endpoint, inputs, outputs, rules, side effects, retries, and failure behavior

### Requirement: Message-edge names and contracts
Every modeled message edge SHALL have a short Name or Label reflecting the function call, command, event, or data being transferred. Its edge-owned Markdown SHALL contain the applicable message contract, including producer and consumer, transport or endpoint, direction, payload purpose and schema or fields, correlation and identity rules, delivery guarantees, security constraints, success acknowledgement, timeout or retry behavior, and failure semantics when applicable.

#### Scenario: Activity sends object data to S3
- **WHEN** a message flow represents an activity storing object data in an S3-compatible system
- **THEN** the edge has a concise name such as `putObject` or `Object data` and its Markdown contains the complete request, response, delivery, security, and failure contract

#### Scenario: Message contract details are incomplete
- **WHEN** a proposed message edge lacks material producer, consumer, payload, transport, delivery, or failure information
- **THEN** the interview asks focused questions or records explicitly approved unresolved items before declaring the edge build-ready

### Requirement: Event-owned contract documentation
Events SHALL use concise Labels that communicate their business meaning, while event-owned Markdown SHALL contain the complete applicable trigger, emitted or consumed data, correlation, timing, delivery, idempotency, ordering, security, and error-handling contract. Contract data specific to an event or edge SHALL NOT be duplicated into unrelated activity Labels or diagram-level prose.

#### Scenario: Message event receives a notification
- **WHEN** a BPMN event consumes an external notification
- **THEN** the event Label identifies the notification meaning and the event Markdown documents its trigger, data contract, correlation, delivery, and failure behavior

#### Scenario: Contract belongs to a sequence or message edge
- **WHEN** a validation condition or transfer contract governs one transition
- **THEN** the assistant attaches that detail to the edge Markdown and keeps adjacent activity and event Labels concise
