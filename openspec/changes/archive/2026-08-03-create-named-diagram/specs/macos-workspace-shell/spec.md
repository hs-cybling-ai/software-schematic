## ADDED Requirements

### Requirement: Native new diagram command
The application SHALL provide a New Diagram command when a workspace is open and SHALL present a native creation sheet for choosing the diagram type, name, and destination.

#### Scenario: User invokes New Diagram
- **WHEN** the user selects File > New Diagram while a workspace is open
- **THEN** the system presents a sheet with every supported diagram type, a name field, and a destination picker

#### Scenario: No workspace is open
- **WHEN** the application has no authorized workspace folder
- **THEN** the New Diagram command is disabled

#### Scenario: User cancels creation
- **WHEN** the user cancels the New Diagram sheet
- **THEN** the system closes the sheet without changing workspace files, the tree, or open tabs

### Requirement: Displayed-folder placement
The creation sheet SHALL offer the workspace root and every folder currently displayed in the sidebar as diagram destinations using clear workspace-relative labels.

#### Scenario: Workspace contains displayed nested folders
- **WHEN** the New Diagram sheet opens for a workspace whose sidebar displays nested folders
- **THEN** the destination picker lists the workspace root and each displayed nested folder in recognizable hierarchy order

#### Scenario: Selected folder is no longer available
- **WHEN** the selected displayed folder is removed or moved before creation is submitted
- **THEN** the sheet remains available, no file is created, and the system asks the user to select an available destination

### Requirement: Inline creation validation
The creation sheet SHALL prevent submission until its current name, format, and destination are valid and SHALL show actionable validation feedback without dismissing the sheet.

#### Scenario: User enters an invalid name
- **WHEN** the name is empty, contains a path separator, or has an extension incompatible with the selected type
- **THEN** the Create action is unavailable and the sheet explains how to correct the name

#### Scenario: Creation encounters a filename collision
- **WHEN** submission finds an existing item with the resolved filename
- **THEN** the sheet remains open and identifies the collision so the user can change the name or destination
