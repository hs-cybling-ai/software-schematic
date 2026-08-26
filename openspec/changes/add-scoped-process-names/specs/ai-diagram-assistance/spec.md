## MODIFIED Requirements

### Requirement: Scope-aware assistant prompting
The application SHALL accept a natural-language change request from the assistant dialog and SHALL construct a versioned, size-bounded context snapshot for the invocation scope. Every snapshot SHALL include the active composition's qualified process Name; normalized nodes and flows with ID, local Name, qualified symbol, Label, kind, owning scope, composition resolution, and live status hints; active diagram Markdown; and a source revision. A node-scoped snapshot SHALL additionally identify the primary node and include its Markdown; a diagram-scoped snapshot SHALL treat the complete active diagram as modifiable scope without selecting a primary node. The application SHALL flush pending relevant edits before snapshotting and SHALL disclose when optional context was truncated.

#### Scenario: Node-scoped request is submitted
- **WHEN** the user submits a prompt from a node magic action
- **THEN** the provider request distinguishes the selected node's ID, Name, qualified symbol, Label, kind, status, and Markdown

#### Scenario: Diagram-scoped request is submitted
- **WHEN** the user submits a prompt from the diagram magic action
- **THEN** the provider request contains the complete active diagram and its qualified process scope with no primary node restriction

#### Scenario: Pending content exists
- **WHEN** the active diagram or relevant Markdown has pending local edits at submission time
- **THEN** the application flushes those edits and derives the snapshot revision from the resulting current state

#### Scenario: Required context exceeds limits
- **WHEN** the structural context required to interpret the active diagram cannot fit within configured request limits
- **THEN** the application refuses the request and explains the exceeded limit rather than silently omitting required structure

### Requirement: Allowed diagram operation schema
The operation schema SHALL support explicit ID, Name, and Label changes; qualified process creation/opening; reviewed process rename; supported flow-node creation; sequence-flow creation; and diagram or node Markdown replacement. Providers SHALL emit qualified architectural Names and SHALL NOT emit composition folder paths. SSW SHALL derive every composition path and SHALL preserve existing Names and Labels unless the plan contains a separate explicit operation previewing that change.

#### Scenario: Task becomes a documented reusable process
- **WHEN** a valid proposal replaces a task with a reusable process node, retains its useful Label, assigns a valid Name in pool scope, creates its composition, adds ordered members, and updates Markdown
- **THEN** the plan identifies the process and members by scoped Names while SSW derives all folders and documentation locations

#### Scenario: Provider shortens an existing identity implicitly
- **WHEN** a proposal creates or links a child composition under a shortened Name without an explicit Name-change operation
- **THEN** validation rejects the proposal before preview or application

#### Scenario: Unsupported mutation is proposed
- **WHEN** a plan supplies a composition path, arbitrary BPMN XML, executable content, or an operation outside the registered schema
- **THEN** validation rejects the complete plan before preview or application

### Requirement: Independent proposal validation
The application SHALL treat provider output as untrusted and SHALL validate the complete plan before preview and again before application. Validation SHALL confirm schema and request correlation, current revision, stable IDs, Name syntax and scope, derived qualified-symbol resolution, process/member kind, collision-free deterministic destinations, BPMN compatibility, size and operation limits, and node-status constraints. An operation SHALL NOT modify ID, Name, Label, type, or configuration of a `locked` node. `new` and `modify` remain positive authoring hints but SHALL NOT bypass validation.

#### Scenario: Locked node would change
- **WHEN** any operation would change a node whose live status is `locked`
- **THEN** the application rejects the complete proposal, identifies the protected node, and makes no changes

#### Scenario: Proposal becomes stale
- **WHEN** the diagram, scoped identity, or relevant Markdown revision changes after proposal generation and before approval
- **THEN** approval is refused and the user is asked to regenerate the proposal from current context

#### Scenario: Qualified Name conflicts
- **WHEN** a proposed process Name resolves to an incompatible existing process or a case-insensitive folder collision
- **THEN** validation rejects the complete proposal and identifies the conflict

#### Scenario: Provider emits a folder path
- **WHEN** an operation attempts to choose or mutate a composition path directly
- **THEN** validation rejects it and directs the provider to supply a scoped process Name instead

## ADDED Requirements

### Requirement: Assistant process rename preview
An assistant-proposed process rename SHALL preview the old and new qualified Names, deterministic folder move, affected definitions and references, and unchanged relative documentation links. Approval SHALL execute the same atomic refactor used by manual diagram-level rename.

#### Scenario: User reviews an AI rename
- **WHEN** the assistant proposes renaming a process
- **THEN** the preview explicitly distinguishes Name changes from Label changes and lists the composition and references that will move

#### Scenario: User approves an AI rename
- **WHEN** the proposal remains current and the user approves it
- **THEN** SSW performs one atomic Name-based refactor with rollback protection
