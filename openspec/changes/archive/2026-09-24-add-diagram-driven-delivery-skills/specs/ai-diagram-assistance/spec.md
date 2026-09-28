## ADDED Requirements

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
The diagram tool SHALL provide a persistent chat-based design interview that lets a user discuss the active diagram, selected elements, and their documentation over multiple turns. Each turn SHALL retain the approved conversation decisions and current revision-bound diagram context, SHALL distinguish questions and explanations from proposed mutations, and SHALL present every structured change set for approval before applying it.

#### Scenario: User refines a diagram conversationally
- **WHEN** the user discusses a process over several turns and answers follow-up questions
- **THEN** the interview retains the established decisions, asks only for unresolved material details, and produces a revision-bound structured change set without requiring the user to restate diagram context

#### Scenario: User asks for an explanation only
- **WHEN** the user's message asks what an existing node or flow means without requesting a change
- **THEN** the assistant answers from the diagram and Markdown context and does not create a mutation proposal

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
