## ADDED Requirements

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
