## ADDED Requirements

### Requirement: Scope-aware assistant prompting
The application SHALL accept a natural-language change request from the assistant dialog and SHALL construct a versioned, size-bounded context snapshot for the invocation scope. Every snapshot SHALL include the active composition path, normalized diagram nodes and flows, BPMN identity and labels, composition references, live node-status hints, active diagram Markdown, and a source revision. A node-scoped snapshot SHALL additionally identify the primary node and include its Markdown; a diagram-scoped snapshot SHALL treat the complete active diagram as modifiable scope without selecting a primary node. The application SHALL flush pending relevant edits before snapshotting and SHALL disclose when optional context was truncated.

#### Scenario: Node-scoped request is submitted
- **WHEN** the user submits a prompt from a node magic action
- **THEN** the provider request contains the active diagram context plus the selected node identity, status, and Markdown as the primary target context

#### Scenario: Diagram-scoped request is submitted
- **WHEN** the user submits a prompt from the diagram magic action
- **THEN** the provider request contains the complete active diagram and diagram Markdown with no primary node restriction

#### Scenario: Pending content exists
- **WHEN** the active diagram or relevant Markdown has pending local edits at submission time
- **THEN** the application flushes those edits and derives the snapshot revision from the resulting current state

#### Scenario: Required context exceeds limits
- **WHEN** the structural context required to interpret the active diagram cannot fit within configured request limits
- **THEN** the application refuses the request and explains the exceeded limit rather than silently omitting required structure

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
The initial operation schema SHALL support replacing an eligible node's BPMN type, changing an eligible node label, assigning or preserving an external composition reference, creating or opening a confined composition, adding supported flow nodes, connecting supported nodes with sequence flows, and replacing diagram or node Markdown. Every operation SHALL use stable element identifiers and normalized composition-relative paths and SHALL contain no executable content. The schema SHALL support the coordinated replacement of a task with a subprocess or call activity, creation of its composition, creation of an ordered multi-step flow, and update of related Markdown.

#### Scenario: Task becomes a documented subprocess
- **WHEN** a valid proposal replaces a task, links it to a new composition, adds four ordered steps and required start/end flow in that composition, and updates parent, diagram, or node Markdown
- **THEN** the plan expresses every change using only allowed typed operations with stable IDs and confined paths

#### Scenario: Unsupported mutation is proposed
- **WHEN** a plan requests arbitrary BPMN XML replacement, JavaScript execution, a shell command, or an operation outside the registered schema
- **THEN** validation rejects the complete plan before preview or application

### Requirement: Independent proposal validation
The application SHALL treat provider output as untrusted and SHALL validate the complete plan before preview and again before application. Validation SHALL confirm schema and request correlation, current source revision, existing references, unique created IDs, permitted BPMN replacements and connections, confined composition/documentation paths, configured size and operation limits, and node-status constraints. An operation SHALL NOT modify, replace, delete, relabel, or reconfigure a `locked` node. `new` and `modify` SHALL be supplied as positive authoring hints but SHALL NOT bypass validation.

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
The application SHALL show every valid proposal in a human-readable preview grouped by affected diagram and documentation file. The preview SHALL identify replacements, additions, connections, composition creation or reuse, Markdown changes, assumptions, and warnings. The application SHALL require an explicit approval action after generation and SHALL provide reject and close actions that make no changes.

#### Scenario: Valid proposal is previewed
- **WHEN** a provider returns a plan that passes validation
- **THEN** the dialog displays its summary and complete grouped semantic change list without mutating the project

#### Scenario: User rejects a proposal
- **WHEN** the user rejects or closes the proposal preview
- **THEN** the proposal is discarded and all diagrams, documentation, tabs, and files remain unchanged

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
