## ADDED Requirements

### Requirement: Data-graph node notation
The editor SHALL represent object and property nodes as labeled, single-outline circles sized comparably to BPMN activity elements, and SHALL render property nodes with a pastel-blue fill that visually distinguishes them from object nodes.

#### Scenario: Object node is displayed
- **WHEN** a data-graph document contains an object node with a label
- **THEN** the editor renders a labeled, single-outline circular object node at its stored position

#### Scenario: Property node is displayed
- **WHEN** a data-graph document contains a property node with a label
- **THEN** the editor renders a labeled, single-outline circular node with the designated pastel-blue property fill

### Requirement: Labeled directed relationships
The editor SHALL represent graph relationships as directed edges with editable, movable labels centered on the logical edge by default, and SHALL preserve each edge's source, target, label, label position, and routing when serialized.

#### Scenario: Relationship is created
- **WHEN** the user connects a source node to a target node and supplies a relationship label
- **THEN** the editor displays a directed, labeled edge between those nodes and marks the document dirty

#### Scenario: Relationship label is edited
- **WHEN** the user changes an edge label
- **THEN** the editor displays the new label and includes it in the next export

#### Scenario: Relationship label is repositioned
- **WHEN** the user drags an edge label away from its default centered position
- **THEN** the editor preserves the new label position in the next export

### Requirement: Object-edge intermediate control
The editor SHALL place a non-optional, half-node-size, double-outline circular control at the geometric midpoint of every object-to-object relationship, and SHALL keep that control associated with and positioned from its owning edge as connected nodes move. An object-to-property edge represents a literal property link and SHALL connect directly without an intermediate control.

#### Scenario: Relationship connects objects
- **WHEN** an edge is created between two object nodes
- **THEN** the editor creates and displays one double-outline intermediate control halfway along that edge

#### Scenario: Connected node moves
- **WHEN** the user moves a node connected to an edge with an intermediate control
- **THEN** the editor reroutes the edge and repositions the control at the updated edge midpoint

#### Scenario: Property literal connects to an object
- **WHEN** an edge connects an object node and a property node
- **THEN** the editor draws one direct, outline-docked edge and creates no intermediate control

#### Scenario: Edge connects only property nodes
- **WHEN** an edge connects two property nodes
- **THEN** the editor does not require an intermediate control on that edge

### Requirement: Edge attachments
The editor SHALL allow property and filter nodes to be attached to an edge through its intermediate control and SHALL persist the attachment to that edge rather than treating it as an ordinary relationship endpoint.

#### Scenario: Property is attached to a relationship
- **WHEN** the user adds a property from an edge's intermediate control
- **THEN** the editor creates a property node connected to and owned by that edge control

#### Scenario: Filter is attached to a relationship
- **WHEN** the user adds a filter from an edge's intermediate control
- **THEN** the editor creates a labeled filter node connected to and owned by that edge control

#### Scenario: Owning edge is deleted
- **WHEN** the user deletes an edge that owns an intermediate control and attachments
- **THEN** the editor removes the control and its edge-owned attachments as one undoable operation

### Requirement: Context-palette authoring
The editor SHALL provide a context palette on eligible nodes and edge controls for creating object nodes, property nodes, relationships, and supported edge attachments without switching to a separate global drawing mode.

#### Scenario: User expands from a node
- **WHEN** the user selects a node and invokes an enabled creation action from its context palette
- **THEN** the editor creates the selected element or begins a connection from that node with placement requiring no unrelated tool selection

#### Scenario: User expands from an edge control
- **WHEN** the user selects an intermediate control
- **THEN** its context palette offers property and filter attachment actions and excludes deferred mutation actions

#### Scenario: Action is invalid for selection
- **WHEN** a palette action would violate data-graph connection or ownership rules
- **THEN** the editor disables or omits that action and leaves the model unchanged

### Requirement: Data-graph document persistence
The application SHALL load and export a versioned, locally stored data-graph document containing nodes, edges, labels, geometry, intermediate-control associations, and edge attachments, and SHALL reject invalid documents without modifying their source files.

#### Scenario: Valid document opens
- **WHEN** the user opens a supported, valid data-graph document
- **THEN** the editor reconstructs its complete editable model and leaves the tab clean

#### Scenario: Edited document exports
- **WHEN** the native layer requests export from a valid edited data-graph session
- **THEN** the web editor returns a deterministic serialized document representing the current model

#### Scenario: Invalid document opens
- **WHEN** a data-graph document has unsupported version data, broken references, or an invalid element shape
- **THEN** the editor reports an actionable import failure, does not mark the tab dirty, and leaves the source file unchanged

### Requirement: Data-graph mutation tracking and isolation
The application SHALL mark a data-graph tab dirty after model-changing commands, support undo and redo for authoring commands, and retain each editor session independently while its tab remains open.

#### Scenario: User changes a data-graph model
- **WHEN** the user creates, edits, moves, connects, or removes a data-graph element
- **THEN** the active tab becomes dirty and the command can be undone and redone

#### Scenario: User only navigates the canvas
- **WHEN** the user pans, zooms, selects, or switches tabs without changing model content
- **THEN** the data-graph tab remains clean and its session state remains available
