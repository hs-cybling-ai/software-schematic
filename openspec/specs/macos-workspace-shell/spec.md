# macOS Workspace Shell

## Purpose

Define the native dark-mode workspace, folder command, tabbed editor shell, and resilient empty and error states.

## Requirements

### Requirement: Native dark-mode workspace window
The application SHALL present a native macOS window using the system dark appearance with a resizable file-tree sidebar on the left and an editor workspace on the right.

#### Scenario: Application launches
- **WHEN** the user launches the application
- **THEN** the system displays the file-tree sidebar and editor workspace using native dark-mode controls and styling

#### Scenario: User resizes the split
- **WHEN** the user drags the divider between the sidebar and editor workspace
- **THEN** the system resizes both areas while preserving a usable minimum width for each

### Requirement: Native folder command
The application SHALL provide an Open Folder command in the File menu and SHALL expose the standard macOS keyboard shortcut for that command.

#### Scenario: User invokes Open Folder
- **WHEN** the user selects File > Open Folder or invokes its keyboard shortcut
- **THEN** the system presents a native folder-selection panel

#### Scenario: User cancels folder selection
- **WHEN** the user cancels the folder-selection panel
- **THEN** the current workspace and open tabs remain unchanged

### Requirement: Tabbed editor workspace
The application SHALL display each open diagram in a selectable, individually closable native tab labeled with its filename and dirty state.

#### Scenario: Multiple diagrams are open
- **WHEN** the user opens more than one supported diagram
- **THEN** the system shows one tab per diagram and displays the selected diagram's editor

#### Scenario: User changes selected tab
- **WHEN** the user selects another tab
- **THEN** the system displays that tab's retained editor session and preserves the prior tab's session state

### Requirement: Empty and error states
The application SHALL provide native empty-state guidance when no folder or diagram is selected and SHALL surface editor and file-operation failures without replacing the entire workspace.

#### Scenario: No folder is open
- **WHEN** the application has no authorized workspace folder
- **THEN** the editor area explains how to open a folder

#### Scenario: An editor fails
- **WHEN** a diagram cannot be loaded or exported
- **THEN** the affected tab shows an actionable error while the sidebar and other tabs remain usable

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
