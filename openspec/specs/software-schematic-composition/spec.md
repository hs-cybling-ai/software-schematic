# Software Schematic Composition

## Purpose

Define deterministic folder conventions, documentation ownership, and navigation for CMMN business anchors and reusable BPMN process models.

## Requirements

### Requirement: Folder-based base diagrams
The system SHALL represent BPMN process compositions with `main.bpmn`, CMMN package anchors with `main.cmmn`, and either diagram type with sibling `main.md` documentation. A new project SHALL use `schematics/main.cmmn` as its root anchor; `schematics/main.bpmn` SHALL remain a legacy fallback only when no root CMMN exists.

#### Scenario: Root composition opens
- **WHEN** the initialized application starts
- **THEN** it opens the sole root anchor and associates `schematics/main.md` with that diagram

### Requirement: Name-based composition folders
The system SHALL map process Name `package.Process` to `schematics/package/Process/`, containing `main.bpmn` and `main.md`. Users and providers SHALL supply the Name and SHALL NOT supply a separate folder path.

#### Scenario: Named process opens
- **WHEN** an element has Name `sales.checkout.PlaceOrder`
- **THEN** SSW opens or creates `schematics/sales/checkout/PlaceOrder/main.bpmn`

### Requirement: ID-bound element documentation
The system SHALL store node and edge Documentation at `docs/<element-id>.md` within the owning composition. It SHALL use `main.md` for process Documentation and SHALL NOT expose a Documentation-path metadata field.

#### Scenario: Node documentation is created
- **WHEN** node `Activity_42` has Name `sales.checkout.PlaceOrder#validateCart`
- **THEN** its Documentation is stored as `schematics/sales/checkout/PlaceOrder/docs/Activity_42.md`

#### Scenario: Edge documentation is created
- **WHEN** edge `Flow_7` has Name `sales.checkout.PlaceOrder#paymentApproved`
- **THEN** its Documentation is stored as `schematics/sales/checkout/PlaceOrder/docs/Flow_7.md`

#### Scenario: Two call sites share one implementation
- **WHEN** two call activities use the same process Name from different owning diagrams
- **THEN** each caller keeps separate ID-bound Documentation in its owning composition and both open the same subprocess `main.md`

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
