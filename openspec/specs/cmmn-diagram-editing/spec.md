# CMMN Diagram Editing

## Purpose

Define self-contained CMMN editing, metadata, persistence, diagnostics, and graph-ready authored identity without adding graph behavior.

## Requirements

### Requirement: Local CMMN modeler
The application SHALL load supported CMMN 1.1 XML files into an editable `cmmn-js` modeler backed entirely by assets bundled beneath `.ss/`. It SHALL preserve supported CMMN semantics, diagram interchange data, stable IDs, and registered SSW extension metadata when importing and exporting a document.

#### Scenario: CMMN file opens offline
- **WHEN** a user opens a valid `.cmmn` file without network access
- **THEN** the application renders it in an editable CMMN tab using only project-local assets

#### Scenario: CMMN content round-trips
- **WHEN** a valid CMMN document containing SSW Names is imported, edited, and saved
- **THEN** its supported semantic elements, diagram layout, stable IDs, and registered SSW metadata remain represented in the exported CMMN XML

### Requirement: CMMN editor parity
The CMMN editor SHALL provide the existing diagram workspace behaviors for retained tabs, selection, undo and redo where supported by `cmmn-js`, viewport fitting, automatic persistence, save status, and safe tab closure. It SHALL release CMMN modeler resources when a CMMN tab closes.

#### Scenario: User edits two diagram types
- **WHEN** a user opens one CMMN file and one BPMN file, edits each, and switches between their tabs
- **THEN** each tab retains its own modeler state and saves to its own canonical path

### Requirement: CMMN element metadata
For every supported selectable CMMN node or connection, the inspector SHALL expose ID, Type, Label, Name, Implementation Status, and Documentation with the same editability rules and visual treatment used for BPMN equivalents. Type SHALL remain read-only, and the application SHALL NOT expose an editable Documentation path.

#### Scenario: CMMN node is selected
- **WHEN** a user selects a supported CMMN plan item
- **THEN** the inspector shows its ID, Type, Label, Name, Implementation Status, and element Documentation

#### Scenario: CMMN connection is selected
- **WHEN** a user selects a supported CMMN connection with a stable ID
- **THEN** the inspector targets that connection and its own metadata and Documentation

#### Scenario: Process Task Label is edited from metadata
- **WHEN** a user selects a CMMN Process Task and changes its Label in the inspector
- **THEN** the selected plan item's semantic Label and rendered node Label update together, persist on save, and its Process Task composition link remains unchanged

#### Scenario: No CMMN node is selected
- **WHEN** the root CMMN diagram is active without a selected node
- **THEN** the Label field is blank and read-only rather than displaying the composition name

#### Scenario: Process Task remains navigable after Label edit
- **WHEN** a user changes a Process Task Label and then double-clicks that task
- **THEN** the editor opens its named BPMN child while preserving the edited CMMN Label

### Requirement: CMMN serialization and diagnostics
The application SHALL serialize saves through `cmmn-js`, SHALL treat fatal import or serialization errors as failed operations, and SHALL report non-fatal import warnings without silently rewriting the source file.

#### Scenario: Invalid CMMN is opened
- **WHEN** `cmmn-js` cannot import a CMMN document
- **THEN** the application reports an actionable error and leaves the source file unchanged

### Requirement: Graph-ready authored identity without graph behavior
CMMN XML saved by this change SHALL retain stable element IDs, element types, labels, complete SSW Names, composition references, and explicit documentation ownership needed by a later graph-ingestion change. This change SHALL NOT parse CMMN into a property graph, create graph records, generate embeddings, or alter graph storage.

#### Scenario: CMMN document is saved
- **WHEN** a user saves a documented and named CMMN diagram
- **THEN** the authored CMMN and Markdown files retain their stable identities without creating or modifying any property-graph artifact
