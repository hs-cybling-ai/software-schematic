## MODIFIED Requirements

### Requirement: Data-graph node notation
The editor SHALL represent object nodes as labeled, neutral-fill, single-outline circles sized comparably to BPMN activity elements; SHALL represent property nodes as neutral-fill, single-outline circles at half the object-node diameter with a centered datatype mark of no more than four characters; and SHALL NOT use fill color to communicate node type.

#### Scenario: Object node is displayed
- **WHEN** a data-graph document contains an object node with a label
- **THEN** the editor renders a labeled, neutral-fill, single-outline circular object node at its stored position

#### Scenario: Typed property node is displayed
- **WHEN** a data-graph document contains a property node with a supported datatype
- **THEN** the editor renders a half-size neutral property circle with its centered datatype mark and its semantic label available without relying on color

#### Scenario: Node type is viewed without color
- **WHEN** the diagram is viewed in monochrome or by a user who cannot distinguish the prior semantic colors
- **THEN** object and property meaning remains distinguishable by size, outline, type mark, and accessible labeling

### Requirement: Object-edge intermediate control
The editor SHALL place a non-optional, half-node-size, double-outline circular control at the geometric midpoint of every object-to-object relationship, SHALL display its selected collection type as a centered icon, and SHALL keep that control associated with and positioned from its owning edge as connected nodes move. An object-to-property edge represents a literal property link and SHALL connect directly without an intermediate control.

#### Scenario: Relationship connects objects
- **WHEN** an edge is created between two object nodes
- **THEN** the editor creates and displays one double-outline intermediate control halfway along that edge with the default collection-type icon and positions the relationship label above the control without overlapping it

#### Scenario: Intermediate control is selected beneath its label
- **WHEN** an object relationship and its label are displayed around an intermediate control
- **THEN** the control's full visible hit area remains unobstructed so the user can select it and open its wrench palette

#### Scenario: Overlapping stored label position is opened
- **WHEN** a controlled relationship is imported with a label position that overlaps its intermediate control
- **THEN** the editor normalizes the label above the control while preserving non-overlapping manually positioned labels

#### Scenario: Connected node moves
- **WHEN** the user moves a node connected to an edge with an intermediate control
- **THEN** the editor reroutes the edge and repositions the control at the updated edge midpoint without changing its collection type

#### Scenario: Property literal connects to an object
- **WHEN** an edge connects an object node and a property node
- **THEN** the editor draws one direct, outline-docked edge and creates no intermediate control

#### Scenario: Edge connects only property nodes
- **WHEN** an edge connects two property nodes
- **THEN** the editor does not require an intermediate control on that edge

### Requirement: Edge attachments
The editor SHALL allow property and mutation nodes to be attached to an edge through its intermediate control and SHALL persist the attachment to that edge rather than treating it as an ordinary relationship endpoint. Mutation nodes SHALL be half the object-node diameter, use a neutral fill and thick outline, and display their selected mutation type without relying on color.

#### Scenario: Property is attached to a relationship
- **WHEN** the user adds a property from an edge's intermediate control
- **THEN** the editor creates a half-size typed property node connected to and owned by that edge control

#### Scenario: Mutation is attached to a relationship
- **WHEN** the user adds a mutation from an edge's intermediate control
- **THEN** the editor creates a half-size, thick-outline mutation node connected to and owned by that edge control

#### Scenario: Owning edge is deleted
- **WHEN** the user deletes an edge that owns an intermediate control and attachments
- **THEN** the editor removes the control and its edge-owned property and mutation attachments as one undoable operation

### Requirement: Context-palette authoring
The editor SHALL provide a context palette on eligible nodes and edge controls for creating object nodes, property nodes, relationships, and supported edge attachments without switching to a separate global drawing mode. Every palette action SHALL have a centered icon, a readable label or accessible name, deliberate spacing and border treatment, and visible hover and keyboard-focus states.

#### Scenario: User expands from a node
- **WHEN** the user selects a node and invokes an enabled creation action from its context palette
- **THEN** the editor creates the selected element or begins a connection from that node with placement requiring no unrelated tool selection

#### Scenario: User expands from an edge control
- **WHEN** the user selects an intermediate control
- **THEN** its context palette offers property and mutation attachment actions and presents each action with an aligned icon and understandable label

#### Scenario: Action is invalid for selection
- **WHEN** a palette action would violate data-graph connection or ownership rules
- **THEN** the editor disables or omits that action and leaves the model unchanged

#### Scenario: User navigates a palette with a keyboard
- **WHEN** focus enters a Data Graph palette or type chooser
- **THEN** the user can identify, focus, activate, and dismiss its actions without a pointer

### Requirement: Data-graph document persistence
The application SHALL load and export a locally stored data-graph document containing nodes, semantic node subtypes, edges, labels, geometry, intermediate-control associations, and edge attachments, and SHALL reject invalid or obsolete documents without modifying their source files.

#### Scenario: Valid typed document opens
- **WHEN** the user opens a supported data-graph document containing property, mutation, or intermediate subtype values
- **THEN** the editor reconstructs its complete typed editable model and leaves the tab clean

#### Scenario: Obsolete filter document is rejected
- **WHEN** the user opens a development data-graph document containing an obsolete filter node or filter attachment
- **THEN** the editor reports that the obsolete graph definition is unsupported and leaves the source file unchanged

#### Scenario: Edited document exports
- **WHEN** the native layer requests export from a valid edited data-graph session
- **THEN** the web editor returns a deterministic current-version document containing the selected semantic subtypes

#### Scenario: Invalid document opens
- **WHEN** a data-graph document has unsupported version data, unknown subtype values, broken references, or an invalid element shape
- **THEN** the editor reports an actionable import failure, does not mark the tab dirty, and leaves the source file unchanged

## ADDED Requirements

### Requirement: Movable graph nodes
The editor SHALL allow object, property, and mutation nodes to be moved by dragging, SHALL keep each node at its committed drop position without snapping back, SHALL persist the resulting coordinates, and SHALL update connected relationship routes and derived intermediate controls without overwriting the moved node's position.

#### Scenario: Object node is moved
- **WHEN** the user drags an object node to a valid new canvas position and releases it
- **THEN** the object remains at the dropped position and its connected relationships reroute from that position

#### Scenario: Compact node is moved
- **WHEN** the user drags a property or mutation node to a valid new canvas position and releases it
- **THEN** the compact node remains at the dropped position with its type mark and semantic label correctly positioned

#### Scenario: Moved position survives save and reopen
- **WHEN** the user moves a graph node, saves the document, and reopens it
- **THEN** the editor restores the node at the committed position

#### Scenario: Node move is undone and redone
- **WHEN** the user moves a graph node and invokes undo and then redo
- **THEN** the node, its connected relationship routes, and any derived intermediate controls return to their prior geometry and then to the committed geometry

### Requirement: Node type selection
The editor SHALL expose a wrench action on property, mutation, and intermediate nodes that opens an anchored type chooser; SHALL list all supported choices with both a compact icon and visible label; and SHALL apply a choice as one undoable, redoable model command.

#### Scenario: Property datatype is selected
- **WHEN** the user opens a property's wrench chooser and selects String, Integer, Byte, Date, or another supported datatype
- **THEN** the property stores the selected datatype and displays its centered mark of no more than four characters

#### Scenario: Mutation type is selected
- **WHEN** the user opens a mutation's wrench chooser and selects query, function, or transformation
- **THEN** the mutation stores and displays the corresponding type icon while retaining its semantic label and thick outline

#### Scenario: Intermediate collection type is selected
- **WHEN** the user opens an intermediate control's wrench chooser and selects stack, queue, set, or map
- **THEN** the control stores the selected type and displays respectively a horizontal-stack, vertical-queue, brace-set, or brace-map icon

#### Scenario: Type change is undone
- **WHEN** the user changes a node type and invokes undo and then redo
- **THEN** the persisted semantic type and rendered icon return to the previous value and then the selected value
