## ADDED Requirements

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
