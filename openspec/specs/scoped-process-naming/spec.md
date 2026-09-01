# Scoped Process Naming

## Purpose

Define stable occurrence identity and reusable architectural identity for BPMN processes, nodes, and edges.

## Requirements

### Requirement: One qualified Name per element
The system SHALL store one architectural Name independently from diagram ID and Label. A Name MAY be authored as a short parent-scoped value or as a fully qualified override and SHALL NOT be split into separate local and derived fields. A BPMN process resolves to `package.Process`; a BPMN node or edge resolves to `package.Process#nodeOrEdgeName`; a CMMN package resolves to `package`; and a CMMN member resolves to `package#memberName`.

#### Scenario: Process is named
- **WHEN** a reusable process receives Name `cybling.subscription.SelectAndOutfit`
- **THEN** that exact value is stored and displayed as its Name

#### Scenario: Node or edge is named
- **WHEN** a node or edge in process `cybling.subscription.SelectAndOutfit` receives Name `validateOrder`
- **THEN** the short value is stored and displayed while resolving to `cybling.subscription.SelectAndOutfit#validateOrder`

#### Scenario: Label changes
- **WHEN** an element Label changes
- **THEN** its ID and Name remain unchanged

### Requirement: Name-based reuse and files
The system SHALL map the process portion of a Name to one composition folder. Equal process Names SHALL open the same `main.bpmn` and `main.md`. A member Name SHALL NOT create another composition or determine its documentation filename.

#### Scenario: Process is reused
- **WHEN** two elements use Name `cybling.subscription.SelectAndOutfit`
- **THEN** both open `schematics/cybling/subscription/SelectAndOutfit/main.bpmn`

#### Scenario: Member documentation is resolved
- **WHEN** element `Activity_42` uses Name `cybling.subscription.SelectAndOutfit#validateOrder`
- **THEN** its Documentation remains stored at `docs/Activity_42.md`

### Requirement: Name rename behavior
Changing a process Name SHALL rename its composition folder and local Name references. Changing a node or edge Name SHALL NOT rename its ID-bound Markdown file or move the owning process folder.

#### Scenario: Process Name changes
- **WHEN** `cybling.subscription.SelectAndOutfit` becomes `cybling.subscription.ConfigureSubscription`
- **THEN** its composition folder and local references use the new process Name

#### Scenario: Member Name changes
- **WHEN** `package.Process#validateOrder` becomes `package.Process#approveOrder`
- **THEN** the element Name changes while its ID-bound documentation file and process folder remain unchanged

### Requirement: CMMN package Names
The system SHALL store one complete architectural Name for a CMMN package independently from CMMN ID and Label. A package Name SHALL use one or more dot-separated lower-camel segments, and its exact value SHALL determine its `main.cmmn` package folder.

#### Scenario: CMMN package Name is stored
- **WHEN** a CMMN package receives Name `cybling.sdk`
- **THEN** that exact Name is stored and maps to `schematics/cybling/sdk/main.cmmn`

### Requirement: Parent-scoped diagram element Names
The system SHALL allow a node or edge Name to be authored locally without repeating the fully qualified parent identity. A short BPMN member Name SHALL resolve against the owning `package.Process`; a short reusable BPMN process Name SHALL resolve against the owning package. A short CMMN member Name SHALL resolve against the CMMN package, and a short CMMN Process Task Name SHALL resolve as a process in that package. Existing fully qualified `package.Process`, `package.Process#member`, and `package#member` forms SHALL remain explicit overrides.

#### Scenario: Local CMMN Process Task Name inherits its package
- **WHEN** Process Task Name `Birth` is authored in CMMN package `cybling.sdk`
- **THEN** it resolves to BPMN process `cybling.sdk.Birth` without requiring the fully qualified value in the inspector

#### Scenario: Local BPMN member Name inherits its process
- **WHEN** node or edge Name `prepareIdentity` is authored in BPMN process `cybling.sdk.Birth`
- **THEN** it resolves to `cybling.sdk.Birth#prepareIdentity`

#### Scenario: Fully qualified Name overrides parent scope
- **WHEN** a reusable node in `cybling.sdk.Birth` is assigned Name `shared.identity.PrepareIdentity`
- **THEN** the explicit target is retained instead of nesting it in `cybling.sdk`

### Requirement: CMMN package rename behavior
Changing a CMMN package Name SHALL rename its package `main.cmmn` composition folder and local CMMN package references through the existing confined rename workflow. It SHALL preserve element IDs and ID-bound Documentation and SHALL not infer or perform broad enterprise refactoring.

#### Scenario: Package Name changes
- **WHEN** package Name `cybling.sdk` changes to `cybling.platform`
- **THEN** the package composition and local package references use the new Name while element Documentation remains bound to stable IDs
