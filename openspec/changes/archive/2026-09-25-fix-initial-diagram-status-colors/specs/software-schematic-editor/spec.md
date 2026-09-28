## MODIFIED Requirements

### Requirement: Implementation Status
The SSW editor SHALL let a user assign exactly one Implementation Status of `new`, `locked`, `modify`, or `open` to each eligible BPMN or CMMN node or edge. The statuses SHALL mean, respectively, that development must create the represented work/content, must not change the element, is allowed and expected to change the element, or is context-only and outside implementation scope. An element with no assigned or recognized status SHALL be treated as `open`. Status SHALL guide agents and SHALL NOT prevent manual editing. The editor SHALL persist recognized non-default status as SSW diagram XML metadata, restore every eligible element's status presentation when the diagram is rendered or rerendered without requiring selection, and include status changes in undo/redo, dirty state, and automatic save. Green `new` and orange `modify` elements SHALL be the only eligible development targets; gray `locked` and white `open` elements SHALL remain readable context.

#### Scenario: User selects an eligible element
- **WHEN** the user selects a BPMN or CMMN node or edge in the active editor
- **THEN** the selected-item metadata displays its current status and a textual explanation of its development-scope meaning

#### Scenario: User marks development scope
- **WHEN** the user changes an eligible element from `open` to `new` or `modify`
- **THEN** the model records the status through its command stack, renders it green or orange, marks the diagram dirty, and automatically persists it in diagram XML

#### Scenario: Persisted status is reopened
- **WHEN** a saved BPMN or CMMN diagram containing `new`, `modify`, or `locked` metadata is opened
- **THEN** every eligible element restores the same status, color, and accessible meaning as part of initial canvas rendering before the user selects or otherwise interacts with an element

#### Scenario: Diagram is rerendered from XML
- **WHEN** an open tab reimports its BPMN or CMMN XML after a supported reload, rename, conflict-resolution, or revert workflow
- **THEN** every eligible element's status presentation is rebuilt from the reimported model before further user interaction

#### Scenario: User clears development scope
- **WHEN** the user changes an element from `new` or `modify` to `open`
- **THEN** the saved model removes or normalizes the default status metadata and the element is no longer eligible for development scope

#### Scenario: Unsupported element is selected
- **WHEN** the current selection is a label, unsupported diagram root, or no element
- **THEN** the status control is unavailable and no element status changes

### Requirement: Status color and non-color presentation
The editor SHALL render the primary fill of an eligible node green for `new`, grey for `locked`, orange for `modify`, and white for `open` from the first visible rendering of the diagram, while preserving BPMN and CMMN outlines, icons, labels, selection cues, and type semantics. The editor SHALL also expose the status name and LLM meaning in text and accessible labeling so status is not communicated by color alone, including before a node has been selected.

#### Scenario: Diagram opens with persisted status colors
- **WHEN** a BPMN or CMMN diagram containing eligible nodes with persisted statuses becomes visible
- **THEN** every node displays the color for its normalized status without requiring a click, selection, focus, or hover

#### Scenario: Node status changes color
- **WHEN** a user assigns `new`, `locked`, `modify`, or `open` to a node
- **THEN** its primary fill changes immediately to green, grey, orange, or white respectively while its diagram type remains recognizable

#### Scenario: Status is interpreted without color
- **WHEN** a user cannot distinguish the node fill colors or navigates with assistive technology
- **THEN** the selected-item metadata and accessible node description identify the exact status and its LLM meaning

#### Scenario: Node retains interaction cues
- **WHEN** a status-colored node is selected, focused, or hovered
- **THEN** its status fill remains understandable and its existing interaction outline or marker remains visible
