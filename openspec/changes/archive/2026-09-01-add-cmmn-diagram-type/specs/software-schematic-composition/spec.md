## ADDED Requirements

### Requirement: Mixed diagram compositions
The system SHALL allow CMMN business-anchor diagrams and BPMN design compositions to coexist beneath `schematics/` and SHALL distinguish them by their complete canonical path and extension. It SHALL allow a CMMN anchor to link to BPMN processes in descendant package folders without requiring CMMN at each intermediate level. Existing BPMN process folder and `main.bpmn` behavior SHALL remain unchanged.

#### Scenario: Package contains CMMN and BPMN compositions
- **WHEN** package `cybling.sdk` has a package diagram and process `Birth`
- **THEN** `cybling/sdk/main.cmmn` and `cybling/sdk/Birth/main.bpmn` open as separate compositions with independent Markdown and save state

### Requirement: CMMN ID-bound documentation
The system SHALL store CMMN diagram Documentation in sibling `main.md` and Documentation for each supported CMMN node or connection at `docs/<element-id>.md` within its owning package folder. Changing a CMMN Name SHALL NOT rename the ID-bound Documentation file.

#### Scenario: CMMN element Documentation is created
- **WHEN** element `PlanItem_42` in `cybling/sdk/main.cmmn` becomes the Documentation target
- **THEN** its Documentation is stored at `schematics/cybling/sdk/docs/PlanItem_42.md`

#### Scenario: CMMN connection ID changes
- **WHEN** a documented CMMN connection ID changes from `Connection_1` to an unused `Connection_2`
- **THEN** its Documentation is safely renamed from `docs/Connection_1.md` to `docs/Connection_2.md`
