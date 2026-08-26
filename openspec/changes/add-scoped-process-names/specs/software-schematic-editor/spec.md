## MODIFIED Requirements

### Requirement: Compact selected-item metadata
The top of the right column SHALL display the selected diagram item's ID, architectural Name, Label, BPMN type, derived qualified symbol, and derived Markdown documentation path in a compact form. ID, Name, and Label SHALL be editable when applicable; type, qualified symbol, and documentation path SHALL be read-only. The editor SHALL NOT expose a separate External process field.

#### Scenario: User selects a BPMN node or edge
- **WHEN** a BPMN node or edge is selected in the active editor
- **THEN** the metadata header displays its instance identity, architectural identity, presentation text, and owning composition information

#### Scenario: User updates an element label
- **WHEN** the user changes the selected element's Label
- **THEN** the active BPMN model updates its display name and schedules the diagram for automatic save without changing Name or qualified identity

#### Scenario: User updates an element ID
- **WHEN** the user submits a valid unused BPMN ID
- **THEN** the model adopts the ID and any existing documentation file is renamed from the old ID to the new ID without overwriting another file

#### Scenario: User updates an ordinary member Name
- **WHEN** the user submits a valid lowerCamelCase Name for a task or event
- **THEN** the model updates the Name and immediately displays its owning process-qualified member symbol without moving a composition

## ADDED Requirements

### Requirement: Diagram process identity and rename
Every named child process diagram SHALL display its qualified process Name in diagram metadata and SHALL provide a rename action distinct from Label editing. The action SHALL preview the derived folder move and affected references before invoking the atomic process refactor. The root navigation diagram SHALL not expose this rename action.

#### Scenario: User prepares a process rename
- **WHEN** the user changes a child diagram's process Name
- **THEN** the editor previews the old and new qualified Names, folder location, and reference count before confirmation

#### Scenario: User cancels process rename
- **WHEN** the user dismisses the rename preview
- **THEN** the process Name, folder, references, tabs, and documentation remain unchanged

### Requirement: Scope-aware Name editing
The editor SHALL validate package, process, and member Names according to their architectural kind and SHALL show the derived qualified identity before persistence. A scope-changing move SHALL offer an explicit choice to retain the existing qualified target or rebind under the new scope.

#### Scenario: Process node is named in a pool
- **WHEN** a user assigns process Name `Init` inside pool package `cybling`
- **THEN** the editor displays derived identity `cybling.Init`

#### Scenario: Node crosses a package boundary
- **WHEN** a named node moves from pool `cybling` to pool `subscription`
- **THEN** the editor asks whether it should retain its former target or rebind under `subscription`
