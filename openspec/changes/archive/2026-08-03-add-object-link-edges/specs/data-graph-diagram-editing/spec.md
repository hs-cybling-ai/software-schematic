## ADDED Requirements

### Requirement: Two object connection choices
The editor SHALL expose exactly two connection choices from an object node: `Relationship`, which creates or routes through an `edgeNode`, and `Object link`, which creates a direct dashed connection to another object. The editor SHALL NOT present the internal domain and range segments of a relationship as additional authoring edge types.

#### Scenario: Object connection choices are shown
- **WHEN** the user opens the connection actions for an object node
- **THEN** the editor offers Relationship and Object link without offering domain or range as separate choices

### Requirement: Object relationship gesture completion
The editor SHALL interpret an ordinary relationship gesture from an object node as creation of one scalar collection relationship containing a new first-class `edgeNode`. When the gesture terminates without reaching another object, the edge node SHALL be placed at the terminal position and connected only by a domain edge from the source object. When the gesture reaches another object, the edge node SHALL be placed midway between the source and target objects and connected by both the domain edge and a range edge to the target. Each complete creation SHALL be one undoable and redoable command, and the editor SHALL NOT retain an untyped direct object-to-object edge from the gesture.

#### Scenario: Relationship terminates without an object
- **WHEN** the user ends an ordinary relationship gesture from a Person object on empty canvas
- **THEN** the editor creates a scalar edge node at the terminal position with a domain edge from Person and no range edge

#### Scenario: Objects are connected with the ordinary action
- **WHEN** the user completes the ordinary Connect action from a Person object onto a Company object
- **THEN** the editor creates a scalar edge node at the midpoint between them with a Person-to-edge-node domain edge and an edge-node-to-Company range edge

#### Scenario: Ordinary object relationship creation is undone
- **WHEN** the user undoes a newly created object-to-object relationship
- **THEN** the editor removes the intermediate edge node, its label, and both generated edges as one operation and redo restores the complete topology

### Requirement: Typed object links
The editor SHALL allow a user to create a direct `objectLink` edge only between two distinct object nodes, SHALL render it as an outline-docked dashed connection without an intermediate node, and SHALL store a registered `linkType`. The initial registered types SHALL include `sameAs` and `subclassOf`; a new object link SHALL default to `sameAs`; and changing its type SHALL be undoable and redoable.

#### Scenario: Same-as object link is created
- **WHEN** the user invokes Link object from a Coworker object and selects a Person object
- **THEN** the editor creates one direct dashed object link with `linkType` `sameAs` and creates no edge node

#### Scenario: Object link type is changed
- **WHEN** the user opens the selected object link's type chooser and selects `subclassOf`
- **THEN** the editor stores `subclassOf`, updates the link's accessible type label, and supports undo and redo of the change

#### Scenario: Invalid object-link endpoint is attempted
- **WHEN** the user attempts to finish an object link on a property, mutation, or edge node
- **THEN** the editor rejects or disables the connection and leaves the model unchanged

#### Scenario: Duplicate or self link is attempted
- **WHEN** the user attempts an object link to the same source object or repeats an existing source-target-type tuple
- **THEN** the editor rejects or omits the link while retaining the existing model

### Requirement: Cycle-safe linked connection inheritance
The system SHALL expose an object's effective collection connections as its directly modeled collection relationships plus the directly modeled collection relationships reachable by following applicable object links from source to target. Resolution SHALL NOT copy or persist inherited topology, SHALL stop revisiting an object already traversed for the same resolution, SHALL deduplicate inherited relationships by edge-node identity, and SHALL return deterministic results. The initial `sameAs` and `subclassOf` types SHALL both make the target object's effective collection connections available to the source object.

#### Scenario: Linked object assumes target connections
- **WHEN** Coworker has a `sameAs` object link to Person and Person has a modeled `works_for` relationship
- **THEN** Coworker's effective connections include the same stored `works_for` relationship without creating another edge node or relationship record

#### Scenario: Linked objects form a cycle
- **WHEN** two or more object links lead traversal back to an object already visited
- **THEN** effective-connection resolution terminates and returns each reachable relationship at most once

#### Scenario: Multiple paths reach one relationship
- **WHEN** an object can reach the same relationship through more than one chain of object links
- **THEN** effective-connection resolution returns that relationship once in deterministic order

#### Scenario: Inherited topology is exported
- **WHEN** a document with an object link and inherited effective connections is exported
- **THEN** the export contains the stored object link and original relationships but does not materialize inferred nodes or edges

## MODIFIED Requirements

### Requirement: Data-graph document persistence
The application SHALL load and export a current-version locally stored data-graph ontology containing only `nodes`, `edges`, and `properties`. Nodes SHALL use `objectNode`, `edgeNode`, or `mutationNode`; domain edges SHALL connect `objectNode` to `edgeNode`; range edges SHALL connect `edgeNode` to `objectNode`; modifier edges SHALL connect `mutationNode` to either `edgeNode` or `objectNode`; object-link edges SHALL connect distinct `objectNode` records and include a registered `linkType`; and properties SHALL reference a valid node or edge owner. The application SHALL reject invalid or obsolete documents without modifying their source files.

#### Scenario: Valid multi-target document opens
- **WHEN** the user opens a current-version document containing a targetless or multi-target collection relationship
- **THEN** the editor reconstructs its complete typed editable topology and leaves the tab clean

#### Scenario: Valid object-link document opens
- **WHEN** the user opens a current-version document containing `sameAs` or `subclassOf` object-link edges, including a cycle across multiple objects
- **THEN** the editor reconstructs the dashed typed links, accepts the cycle, and leaves the tab clean

#### Scenario: Obsolete version-one shape is rejected
- **WHEN** the user opens a version-1 Data Graph document using the replaced edge/control ownership shape
- **THEN** the editor reports that the obsolete development definition is invalid and leaves the source file unchanged

#### Scenario: Edited document exports
- **WHEN** the native layer requests export from a valid edited Data Graph session
- **THEN** the web editor returns a deterministic current-version document containing only normalized nodes, typed stored edges including object links, and owned properties, with no relationships, attachments, or inferred topology aggregate

#### Scenario: Domain and range IDs are generated
- **WHEN** an edge node with ID `works_for` is connected from `person` and targets `company`
- **THEN** the exported domain edge is `works_for_domain` and the exported range edge is `works_for_range_company`

#### Scenario: Invalid relationship document opens
- **WHEN** a document has an intermediate relationship with a missing source, a non-object source or target, duplicate target IDs, more than one target for scalar, broken references, or an unknown subtype
- **THEN** the editor reports an actionable import failure, does not mark the tab dirty, and leaves the source file unchanged

#### Scenario: Invalid object-link document opens
- **WHEN** an object link has a missing or non-object endpoint, a self-reference, an unknown `linkType`, or duplicates an existing source-target-type tuple
- **THEN** the editor reports an actionable import failure, does not mark the tab dirty, and leaves the source file unchanged

#### Scenario: Obsolete filter document is rejected
- **WHEN** the user opens a development Data Graph document containing an obsolete filter node or filter attachment
- **THEN** the editor reports that the obsolete graph definition is unsupported and leaves the source file unchanged
