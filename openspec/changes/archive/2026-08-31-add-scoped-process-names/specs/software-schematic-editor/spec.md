## MODIFIED Requirements

### Requirement: Compact selected-item metadata
The inspector SHALL expose exactly ID, Type, Label, Name, Implementation Status, and Documentation for a selected diagram element. ID, Label, Name, Implementation Status, and Documentation SHALL be editable when applicable; Type SHALL be read-only. Documentation SHALL be the Markdown editing surface, not a path field. The editor SHALL NOT expose Qualified Name, Documentation path, or External process fields.

#### Scenario: User selects a node or edge
- **WHEN** a BPMN node or edge is selected
- **THEN** the inspector shows ID, Type, Label, Name, Implementation Status, and its Documentation editor

#### Scenario: User edits Name
- **WHEN** the user enters a valid complete qualified Name
- **THEN** the exact Name is stored and the associated composition or documentation location follows it

#### Scenario: User edits Label
- **WHEN** the user changes Label
- **THEN** the Name and associated files do not change

### Requirement: Name required before subprocess navigation
When a user opens a call activity or subprocess by double-clicking it or selecting its subprocess link icon and the element has no Name, the editor SHALL show a modal dialog that collects a valid `package.Process` Name. The editor SHALL NOT generate a fallback Name or create a composition until the user submits a valid Name.

#### Scenario: Unnamed subprocess is opened
- **WHEN** the user double-clicks an unnamed subprocess or selects its subprocess link icon
- **THEN** a Name dialog opens and the composition remains unopened

#### Scenario: Valid Name is submitted
- **WHEN** the user submits `sales.checkout.PlaceOrder`
- **THEN** the exact Name is stored on the element and the shared subprocess composition opens

#### Scenario: Name dialog is cancelled
- **WHEN** the user cancels the Name dialog
- **THEN** the subprocess remains unnamed and no composition is created
