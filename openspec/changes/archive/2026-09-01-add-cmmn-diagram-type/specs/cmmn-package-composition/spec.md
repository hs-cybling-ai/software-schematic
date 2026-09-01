## ADDED Requirements

### Requirement: Root CMMN domain anchor
The system SHALL treat `schematics/main.cmmn` as the sole new-project anchor. The root CMMN SHALL define the project domain, actors, inputs, outputs, needs, and business services using CMMN concepts and Documentation. BPMN pools and lanes SHALL NOT be required to represent that abstract context.

#### Scenario: Root need points to design
- **WHEN** the root CMMN represents a business service with a named Process Task
- **THEN** the Process Task points to a qualified BPMN process that defines logical architecture and solution building blocks

### Requirement: Name-based CMMN package files
The system SHALL map complete CMMN package Name `lowerCamel(.lowerCamel)*` to `schematics/<package-segments>/main.cmmn` and SHALL treat that diagram as a visual business anchor for the needs modeled within it. The package folder SHALL remain a neutral grouping container that can exist, contain nested packages, and be created by existing BPMN composition without requiring a CMMN document at that level. A CMMN business anchor SHALL use `main.md` as its diagram Documentation and SHALL NOT store an independent editable composition path.

#### Scenario: Nested package opens
- **WHEN** a CMMN package has Name `cybling.sdk`
- **THEN** the system opens or creates `schematics/cybling/sdk/main.cmmn` and associates `schematics/cybling/sdk/main.md`

### Requirement: CMMN member Names
An ordinary named CMMN node or connection SHALL accept a short lower-camel member Name and resolve it as `<owning-package>#<memberName>`. A complete `package#memberName` SHALL remain available as an explicit override. IDs SHALL identify occurrences and Labels SHALL remain unrestricted presentation text.

#### Scenario: Package member is named
- **WHEN** a CMMN element in package `cybling.sdk` receives Name `birthRequirements`
- **THEN** the short Name is stored independently from its ID and Label and resolves as `cybling.sdk#birthRequirements`

### Requirement: BPMN process composition link
A supported CMMN Process Task SHALL accept either a short BPMN process Name `Process`, inherited from the owning CMMN package, or a fully qualified `package.Process` override as a direct authored trace from a modeled business need to its BPMN design. Opening the task SHALL resolve the Name and reuse the existing BPMN mapping to `schematics/<package-segments>/<Process>/main.bpmn`; the user and assistant SHALL NOT supply a file path.

#### Scenario: Process Task opens Birth
- **WHEN** a Process Task in package `cybling.sdk` has Name `Birth`
- **THEN** it opens or creates `schematics/cybling/sdk/Birth/main.bpmn`

#### Scenario: Higher-level need anchors descendant design
- **WHEN** Process Task `Birth` in CMMN anchor `cybling` has Name `cybling.sdk.Birth` and package `cybling.sdk` has no `main.cmmn`
- **THEN** the task still opens or creates `schematics/cybling/sdk/Birth/main.bpmn` and remains its business-need anchor

### Requirement: Anchor-first design workflow
The editor SHALL make Process Task composition navigation the direct workflow for creating or opening a BPMN design from its CMMN need and SHALL preserve the originating CMMN diagram as available context. It SHALL NOT require a CMMN document in the target BPMN package or any intermediate package.

#### Scenario: User designs from a need
- **WHEN** a user opens a named Process Task from a CMMN business anchor
- **THEN** the target BPMN composition opens while the originating CMMN tab remains available as the parent business context

### Requirement: Flexible direct BPMN authoring
The system SHALL retain existing direct BPMN composition and folder creation for simple authoring and existing projects. It SHALL NOT invent CMMN needs, block a BPMN save, or require one CMMN document per BPMN process or package level.

#### Scenario: BPMN is created directly
- **WHEN** a user creates a BPMN composition through the existing BPMN workflow
- **THEN** the composition is created normally without synthesizing or requiring a CMMN file

### Requirement: Name required before CMMN composition navigation
When a user opens an unnamed CMMN Process Task, the editor SHALL request a valid short or fully qualified BPMN process Name and SHALL create no file or folder if the dialog is cancelled or invalid.

#### Scenario: Unnamed Process Task is opened
- **WHEN** a user invokes composition navigation on an unnamed CMMN Process Task
- **THEN** the editor requests a process Name, explains parent-package inheritance, and leaves the project unchanged until a valid Name is submitted

### Requirement: CMMN composition confinement
The system SHALL validate CMMN package-document and BPMN process Names, derive their paths beneath `schematics/`, reject traversal and collisions, and avoid overwriting an existing file or composition of an incompatible type. It SHALL retain existing BPMN folder creation behavior independently of CMMN.

#### Scenario: Invalid package Name is submitted
- **WHEN** a CMMN package Name contains traversal, an absolute path, an empty segment, or a segment outside the package grammar
- **THEN** composition creation fails without modifying project files
