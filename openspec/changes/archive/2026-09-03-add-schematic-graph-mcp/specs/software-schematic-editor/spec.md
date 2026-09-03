## MODIFIED Requirements

### Requirement: Implementation Status
The SSW editor SHALL let a user assign exactly one Implementation Status of `new`, `locked`, `modify`, or `open` to each eligible BPMN or CMMN node or edge. The statuses SHALL mean, respectively, that development must create the represented work/content, must not change the element, is allowed and expected to change the element, or is context-only and outside implementation scope. An element with no assigned or recognized status SHALL be treated as `open`. Status SHALL guide agents and SHALL NOT prevent manual editing. The editor SHALL persist recognized non-default status as SSW diagram XML metadata, restore it when the diagram reopens, and include status changes in undo/redo, dirty state, and automatic save. Green `new` and orange `modify` elements SHALL be the only eligible development targets; gray `locked` and white `open` elements SHALL remain readable context.

#### Scenario: User selects an eligible element
- **WHEN** the user selects a BPMN or CMMN node or edge in the active editor
- **THEN** the selected-item metadata displays its current status and a textual explanation of its development-scope meaning

#### Scenario: User marks development scope
- **WHEN** the user changes an eligible element from `open` to `new` or `modify`
- **THEN** the model records the status through its command stack, renders it green or orange, marks the diagram dirty, and automatically persists it in diagram XML

#### Scenario: Persisted status is reopened
- **WHEN** a saved diagram containing `new`, `modify`, or `locked` metadata is closed and reopened
- **THEN** each eligible element restores the same status, color, and textual meaning

#### Scenario: User clears development scope
- **WHEN** the user changes an element from `new` or `modify` to `open`
- **THEN** the saved model removes or normalizes the default status metadata and the element is no longer eligible for development scope

#### Scenario: Unsupported element is selected
- **WHEN** the current selection is a label, unsupported diagram root, or no element
- **THEN** the status control is unavailable and no element status changes
