## ADDED Requirements

### Requirement: Authorized folder workspace
The application SHALL read and write files only within a folder explicitly selected by the user and SHALL retain restorable access through a security-scoped bookmark when the macOS sandbox permits it.

#### Scenario: User authorizes a folder
- **WHEN** the user selects a folder in the native panel
- **THEN** the system opens it as the workspace and stores the access information required for later restoration

#### Scenario: Stored authorization is stale
- **WHEN** the application cannot restore a previously authorized folder
- **THEN** the system requests the user to select the folder again and does not attempt unauthorized file access

#### Scenario: Linked path escapes the workspace
- **WHEN** a discovered symlink resolves outside the authorized root
- **THEN** the system excludes that target from readable and writable diagram nodes

### Requirement: Supported diagram discovery
The application SHALL recursively display folders and supported BPMN and ArchiMate diagram files beneath the authorized root, SHALL sort folders before files by localized name, and SHALL exclude unsupported files from the visible diagram tree.

#### Scenario: Folder contains mixed content
- **WHEN** the selected folder contains nested supported diagrams and unrelated files
- **THEN** the tree displays the folder hierarchy and supported diagrams but not unrelated files

#### Scenario: Workspace content changes
- **WHEN** a supported diagram is added, removed, or renamed while the workspace is open
- **THEN** the system refreshes the tree while preserving valid selection and open tabs

### Requirement: Open diagram identity
The application SHALL open a supported diagram from the tree in at most one tab per canonical file URL.

#### Scenario: User opens a closed diagram
- **WHEN** the user activates a supported diagram in the tree
- **THEN** the system reads the file and opens it in a selected editor tab matching its format

#### Scenario: User opens an already open diagram
- **WHEN** the user activates a diagram whose canonical URL already has a tab
- **THEN** the system selects the existing tab instead of creating a duplicate

### Requirement: Safe persistence
The application SHALL export editor content and atomically replace the original file only after a successful export, and SHALL clear dirty state only after a successful write.

#### Scenario: User saves an edited diagram
- **WHEN** the active tab is dirty and the user invokes Save
- **THEN** the system exports the diagram, atomically writes it to its original file, and marks the tab clean

#### Scenario: Export or write fails
- **WHEN** export or atomic file replacement fails
- **THEN** the system preserves the original file, keeps the tab dirty, and reports the failure

#### Scenario: Source changed externally
- **WHEN** the on-disk file has changed since it was loaded and the user attempts to save
- **THEN** the system requires an explicit conflict decision before overwriting or reloading the file

### Requirement: Unsaved-change protection
The application SHALL offer Save, Don’t Save, and Cancel before closing a dirty tab, replacing a workspace that has dirty tabs, or terminating with dirty tabs.

#### Scenario: User cancels a dirty close
- **WHEN** the user attempts to close a dirty tab and chooses Cancel
- **THEN** the tab remains open with its edits intact

#### Scenario: User saves during dirty close
- **WHEN** the user chooses Save from an unsaved-change prompt
- **THEN** the system closes or proceeds only after the save succeeds

#### Scenario: User discards changes
- **WHEN** the user chooses Don’t Save from an unsaved-change prompt
- **THEN** the system proceeds without writing the edited content
