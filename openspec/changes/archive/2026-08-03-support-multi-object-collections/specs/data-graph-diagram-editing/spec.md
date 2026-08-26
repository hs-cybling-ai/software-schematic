## MODIFIED Requirements

### Requirement: Labeled directed relationships
The editor SHALL represent each collection relationship as one directed, labeled source-to-intermediate connection followed by zero or more directed intermediate-to-object membership connections; SHALL allow distinct relationships to share the same target object; SHALL provide an editable, movable relationship label near the source-to-intermediate connection; and SHALL preserve each relationship's source, targets, semantic label, applicable label position, intermediate position, and routing when serialized. Direct dependent-property and mutation connections SHALL continue to follow their target-driven notation.

#### Scenario: Targetless relationship is created
- **WHEN** the user adds an intermediate collection relationship from an object without choosing a target
- **THEN** the editor displays a labeled directed connection from the source object to a selectable intermediate node, permits zero targets, and marks the document dirty

#### Scenario: Relationship gains multiple object targets
- **WHEN** the user connects an intermediate collection node to more than one object node
- **THEN** the editor displays a directed membership connection to every target while retaining one collection type and one relationship label

#### Scenario: Distinct relationships share a target
- **WHEN** the user creates `works_at` and `worker_for` relationships from the same source object and connects both intermediate nodes to the same Company object
- **THEN** the editor preserves two independently editable relationships that converge on the Company object

#### Scenario: Visible relationship label is edited
- **WHEN** the user changes the visible label of a collection relationship
- **THEN** the editor displays the new label once near its source-to-intermediate connection and includes it in the next export

#### Scenario: Visible relationship label is repositioned
- **WHEN** the user drags a relationship label away from its default position
- **THEN** the editor preserves the new label position in the next export without duplicating it on membership connections

### Requirement: Object-edge intermediate control
The editor SHALL represent every collection relationship with one non-optional, first-class, half-node-size, double-outline edge node that displays its selected type using the registered icon treatment, has exactly one source object, and permits targets according to that type's cardinality. Scalar edge nodes SHALL permit zero or one target and use a blank center treatment; stack, queue, set, and map edge nodes SHALL permit zero or more unique target objects and display their registered icons. The edge node SHALL have its own independently persisted identity and coordinates and SHALL NOT be represented as an edge waypoint. An edge targeting a property SHALL remain a direct dashed dependent-property link without creating another edge node or edge label.

#### Scenario: Intermediate is added without a target
- **WHEN** the user adds a collection relationship from an object and does not select a target object
- **THEN** the editor creates one scalar edge node connected to the source, renders its center without an icon, treats it as a temporary incomplete authoring state, and exposes actions to connect or create the required target

#### Scenario: Scalar relationship gains one target
- **WHEN** a targetless scalar intermediate node is connected to one eligible object
- **THEN** the editor accepts the object as its single target and preserves the scalar type

#### Scenario: Scalar relationship is given a second target
- **WHEN** a scalar intermediate node already has one target and the user attempts to add another
- **THEN** the editor rejects or disables the action and instructs the user to select a multi-valued intermediate type

#### Scenario: Heterogeneous collection is authored
- **WHEN** the user adds Company and Person objects as targets of the same `works_for` intermediate node
- **THEN** the editor retains both differently typed object targets as members of that one collection relationship

#### Scenario: Duplicate membership is attempted
- **WHEN** a relationship already targets an object and the user attempts to add the same object to that relationship again
- **THEN** the editor rejects or omits the duplicate membership while leaving the existing membership unchanged

#### Scenario: Intermediate node is moved
- **WHEN** the user moves an intermediate node belonging to a targetless or multi-target relationship
- **THEN** the node remains at its committed position and all of that relationship's source and membership connections reroute to it

#### Scenario: Any node is moved
- **WHEN** the user moves an object, edge, mutation, or property node
- **THEN** the editor persists its new center coordinates, reroutes its attached connections, marks the document dirty, and supports undo and redo of the movement

#### Scenario: Relationship label and intermediate remain selectable
- **WHEN** a collection relationship and its label are displayed around an intermediate node
- **THEN** the intermediate node's full visible hit area remains unobstructed so the user can select it and open its palette

#### Scenario: Property literal connects to an object or intermediate
- **WHEN** an edge from an object or intermediate node targets a property node
- **THEN** the editor draws one direct, dashed, outline-docked edge without creating another intermediate node, target marker, or edge label

### Requirement: Function attachments
The editor SHALL represent functions as first-class `mutationNode` nodes connected to edge nodes or object nodes by typed `modifier` edges. A mutation node SHALL be half the object-node diameter, use a neutral fill and thick outline, display a readable, mutation-specific-size `←ƒ` for a query that returns collection items, `ƒ→` for a function that updates the collection, or `ƒ` for a transformation that modifies internal domain-object state, and accept owned property definitions as its parameter contract. These marks SHALL use Unicode arrow characters and SHALL have accessible labels that do not rely on glyph interpretation. A modifier SHALL connect `mutationNode → edgeNode` when the function applies to a relationship in the context of its domain object, or `mutationNode → objectNode` when the function operates directly on that object type.

#### Scenario: Function is attached directly to an object
- **WHEN** the user adds a function from an object node's context palette
- **THEN** the editor creates a mutation node and modifier edge targeting that object node while preserving function type, label, geometry, and parameter properties

#### Scenario: Function direction is displayed
- **WHEN** the user selects query, function, or transformation for a mutation node
- **THEN** the node displays `←ƒ`, `ƒ→`, or `ƒ` respectively and exposes an accessible description of its return, update, or internal-state semantics

#### Scenario: Property is attached to a relationship
- **WHEN** the user adds a property from a relationship's intermediate node
- **THEN** the editor creates a half-size typed property node connected to and owned by that relationship

#### Scenario: Mutation is attached to a relationship
- **WHEN** the user adds a mutation from a relationship's intermediate node
- **THEN** the editor creates a half-size, thick-outline mutation node and a modifier edge from that mutation node to the selected edge node

#### Scenario: Mutation parameter is defined
- **WHEN** the user adds a property such as `companyUrn` or `name` from a mutation node
- **THEN** the editor stores that property with the mutation node as its owner so the mutation signature is explicit

#### Scenario: Membership is deleted
- **WHEN** the user deletes one intermediate-to-object membership connection
- **THEN** the editor removes only that target membership and retains the relationship, its other targets, its attachments, and the target object

#### Scenario: Owning relationship is deleted
- **WHEN** the user deletes a relationship's source connection or intermediate node
- **THEN** the editor removes the relationship, its intermediate node, label, membership connections, and relationship-owned property and mutation attachments as one undoable operation while retaining its former target objects

### Requirement: Context-palette authoring
The editor SHALL provide a context palette on eligible nodes and intermediate nodes for creating object nodes, property nodes, collection relationships, membership connections, and supported relationship attachments without switching to a separate global drawing mode. Every palette action SHALL have a centered icon, a readable label or accessible name, deliberate spacing and border treatment, and visible hover and keyboard-focus states.

#### Scenario: User adds a targetless relationship from an object
- **WHEN** the user selects an object and invokes the intermediate-relationship creation action
- **THEN** the editor creates a labeled intermediate node sourced by that object without requiring an object target

#### Scenario: User connects an existing object from an intermediate
- **WHEN** the user invokes Connect object from an intermediate palette and selects an eligible existing object
- **THEN** the editor adds that object as a target of the selected collection relationship in one undoable command

#### Scenario: User creates an object from an intermediate
- **WHEN** the user invokes Create object from an intermediate palette and places the new object
- **THEN** the editor creates the object and adds it as a target of the selected collection relationship in one undoable command

#### Scenario: User expands attachments from an intermediate
- **WHEN** the user selects an intermediate node
- **THEN** its context palette also offers property and mutation attachment actions with aligned icons and understandable labels

#### Scenario: Action is invalid for selection
- **WHEN** a palette action would create a second source, a non-object member, or a duplicate target membership
- **THEN** the editor disables or omits that action and leaves the model unchanged

#### Scenario: User navigates a palette with a keyboard
- **WHEN** focus enters a Data Graph palette or type chooser
- **THEN** the user can identify, focus, activate, and dismiss its actions without a pointer

### Requirement: Node type selection
The editor SHALL expose a wrench action on property, mutation, and intermediate nodes that opens an anchored type chooser; SHALL list all supported choices with both an accessible label and their registered visual treatment; and SHALL apply a valid choice as one undoable, redoable model command. The property chooser SHALL include URI as a first-class datatype for identifiers and resource references. The intermediate chooser SHALL include Scalar (`0..1`), Stack, Queue, Set, and Map, with Scalar as the default for newly created intermediate nodes.

#### Scenario: URI property type is selected
- **WHEN** the user assigns URI to an object property such as `id` or `companyUrn`
- **THEN** the property stores `uri`, displays the `URI` datatype mark, and preserves that semantic type in the exported version-1 document

#### Scenario: Default intermediate type is scalar
- **WHEN** the user creates a new intermediate relationship
- **THEN** the intermediate stores `scalar`, presents a blank center icon treatment, and exposes an accessible `Scalar (0..1)` name

#### Scenario: Multi-valued intermediate type is selected
- **WHEN** the user opens an intermediate node's wrench chooser and selects stack, queue, set, or map
- **THEN** the node stores the selected type, permits multiple unique targets, and displays respectively its horizontal-stack, vertical-queue, brace-set, or brace-map icon

#### Scenario: Scalar is selected for a relationship with at most one target
- **WHEN** the user selects Scalar for an intermediate node with zero or one target
- **THEN** the node stores `scalar`, changes its cardinality to `0..1`, and removes the prior collection icon without removing its existing target

#### Scenario: Scalar is selected for a multi-target relationship
- **WHEN** the user attempts to select Scalar for an intermediate node with more than one target
- **THEN** the editor disables or rejects the type change and leaves the relationship and all targets unchanged

#### Scenario: Type change is undone
- **WHEN** the user changes an intermediate type and invokes undo and then redo
- **THEN** the persisted type, cardinality behavior, and rendered icon treatment return to the previous value and then the selected value

### Requirement: Data-graph document persistence
The application SHALL load and export a current-version locally stored data-graph ontology containing only `nodes`, `edges`, and `properties`. Nodes SHALL use `objectNode`, `edgeNode`, or `mutationNode`; domain edges SHALL connect `objectNode` to `edgeNode`; range edges SHALL connect `edgeNode` to `objectNode`; modifier edges SHALL connect `mutationNode` to either `edgeNode` or `objectNode`; and properties SHALL reference a valid node or edge owner. The application SHALL reject invalid or obsolete documents without modifying their source files.

#### Scenario: Valid multi-target document opens
- **WHEN** the user opens a current-version document containing a targetless or multi-target collection relationship
- **THEN** the editor reconstructs its complete typed editable topology and leaves the tab clean

#### Scenario: Obsolete version-one shape is rejected
- **WHEN** the user opens a version-1 Data Graph document using the replaced edge/control ownership shape
- **THEN** the editor reports that the obsolete development definition is invalid and leaves the source file unchanged

#### Scenario: Edited document exports
- **WHEN** the native layer requests export from a valid edited Data Graph session
- **THEN** the web editor returns a deterministic current-version document containing only normalized nodes, typed edges, and owned properties, with no relationships or attachments aggregate

#### Scenario: Domain and range IDs are generated
- **WHEN** an edge node with ID `works_for` is connected from `person` and targets `company`
- **THEN** the exported domain edge is `works_for_domain` and the exported range edge is `works_for_range_company`

#### Scenario: Invalid relationship document opens
- **WHEN** a document has an intermediate relationship with a missing source, a non-object source or target, duplicate target IDs, more than one target for scalar, broken references, or an unknown subtype
- **THEN** the editor reports an actionable import failure, does not mark the tab dirty, and leaves the source file unchanged

#### Scenario: Obsolete filter document is rejected
- **WHEN** the user opens a development Data Graph document containing an obsolete filter node or filter attachment
- **THEN** the editor reports that the obsolete graph definition is unsupported and leaves the source file unchanged

### Requirement: Data-graph mutation tracking and isolation
The application SHALL mark a Data Graph tab dirty after model-changing commands, support undo and redo for targetless relationship creation and all relationship membership changes, and retain each editor session independently while its tab remains open.

#### Scenario: User changes collection topology
- **WHEN** the user creates or deletes a targetless relationship, adds or removes a target membership, or creates a target object from an intermediate node
- **THEN** the active tab becomes dirty and the complete topology change can be undone and redone atomically

#### Scenario: User only navigates the canvas
- **WHEN** the user pans, zooms, selects, or switches tabs without changing model content
- **THEN** the Data Graph tab remains clean and its session state remains available
