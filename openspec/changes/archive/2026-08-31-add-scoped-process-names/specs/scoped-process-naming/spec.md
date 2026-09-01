## ADDED Requirements

### Requirement: One qualified Name per element
The system SHALL store one architectural Name independently from BPMN ID and Label. A process Name SHALL use `package.Process`; a node or edge Name SHALL use `package.Process#nodeOrEdgeName`. The stored Name SHALL be the complete qualified value and SHALL NOT be split into local and derived fields.

#### Scenario: Process is named
- **WHEN** a reusable process receives Name `cybling.subscription.SelectAndOutfit`
- **THEN** that exact value is stored and displayed as its Name

#### Scenario: Node or edge is named
- **WHEN** a node or edge receives Name `cybling.subscription.SelectAndOutfit#validateOrder`
- **THEN** that exact value is stored and displayed as its Name

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
