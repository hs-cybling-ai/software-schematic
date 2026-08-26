## MODIFIED Requirements

### Requirement: Data-graph node notation
The editor SHALL represent object nodes as labeled, neutral-fill, single-outline circles sized comparably to BPMN activity elements; SHALL represent property nodes as neutral-fill, single-outline circles at half the object-node diameter with a centered datatype mark of no more than four characters and the property's semantic label below the node; and SHALL NOT use fill color to communicate node type.

#### Scenario: Object node is displayed
- **WHEN** a data-graph document contains an object node with a label
- **THEN** the editor renders a labeled, neutral-fill, single-outline circular object node at its stored position

#### Scenario: Typed property node is displayed
- **WHEN** a data-graph document contains a property node with a supported datatype
- **THEN** the editor renders a half-size neutral property circle with its centered datatype mark and its semantic label below the property node without relying on color

#### Scenario: Node type is viewed without color
- **WHEN** the diagram is viewed in monochrome or by a user who cannot distinguish the prior semantic colors
- **THEN** object and property meaning remains distinguishable by size, outline, type mark, and accessible labeling

### Requirement: Labeled directed relationships
The editor SHALL represent graph relationships as directed edges whose label and target marker behavior follows the target node type; SHALL provide editable, movable labels centered on the logical edge by default except when the target is a property; and SHALL preserve each edge's source, target, semantic label, applicable label position, and routing when serialized.

#### Scenario: Ordinary relationship is created
- **WHEN** the user connects a source node to a target that is neither a property nor a mutation and supplies a relationship label
- **THEN** the editor displays a directed, labeled edge with the ordinary target arrow between those nodes and marks the document dirty

#### Scenario: Mutation-targeting relationship is created
- **WHEN** the user connects any supported source to a mutation node
- **THEN** the editor displays a solid directed edge ending in a black circular target marker and marks the document dirty

#### Scenario: Property-targeting relationship is created
- **WHEN** the user connects any supported source to a property node
- **THEN** the editor displays a dashed edge without an edge label or target arrow and uses the semantic label below the property node to identify the dependent property

#### Scenario: Visible relationship label is edited
- **WHEN** the user changes the visible label of an edge whose target is not a property
- **THEN** the editor displays the new label and includes it in the next export

#### Scenario: Visible relationship label is repositioned
- **WHEN** the user drags the visible label of an edge whose target is not a property away from its default centered position
- **THEN** the editor preserves the new label position in the next export

### Requirement: Object-edge intermediate control
The editor SHALL place a non-optional, half-node-size, double-outline circular control at the geometric midpoint of every object-to-object relationship, SHALL display its selected collection type as a centered icon, and SHALL keep that control associated with and positioned from its owning edge as connected nodes move. An edge targeting a property SHALL represent a literal dependent-property link as a direct dashed connection without an intermediate control or edge label.

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

#### Scenario: Property literal connects to an object or control
- **WHEN** an edge from an object or intermediate control targets a property node
- **THEN** the editor draws one direct, dashed, outline-docked edge without an intermediate control, target marker, or edge label

#### Scenario: Edge connects only property nodes
- **WHEN** an edge from one property node targets another property node
- **THEN** the editor draws a dashed edge without an intermediate control, target marker, or edge label

## ADDED Requirements

### Requirement: Target-driven dependent edge notation
The editor SHALL determine property and mutation connection notation from the target node's semantic type, regardless of the connection's source type, ownership, creation path, or whether the connection was loaded from storage.

#### Scenario: Different sources target properties
- **WHEN** supported connections from different source types target property nodes
- **THEN** every connection is dashed and has no edge label or target marker

#### Scenario: Different sources target mutations
- **WHEN** supported connections from different source types target mutation nodes
- **THEN** every connection is solid and ends in a black circular target marker

#### Scenario: Target-driven notation survives save and reopen
- **WHEN** a diagram containing property-targeting and mutation-targeting connections is saved and reopened
- **THEN** each connection is rendered according to its target type with the same line, marker, and label behavior as a newly created connection

#### Scenario: Property is used as a source
- **WHEN** a property node is the source of a connection whose target is not a property
- **THEN** the editor determines the connection notation from the actual target and does not make the connection dashed merely because its source is a property
