## MODIFIED Requirements

### Requirement: Supported diagram discovery
The application SHALL recursively display folders and supported BPMN and data-graph diagram files beneath the authorized root, SHALL sort folders before files by localized name, and SHALL exclude unsupported files, including legacy ArchiMate files, from the visible diagram tree.

#### Scenario: Folder contains mixed content
- **WHEN** the selected folder contains nested supported diagrams and unrelated files
- **THEN** the tree displays the folder hierarchy and supported BPMN and data-graph diagrams but not unrelated or ArchiMate files

#### Scenario: Workspace content changes
- **WHEN** a supported diagram is added, removed, or renamed while the workspace is open
- **THEN** the system refreshes the tree while preserving valid selection and open tabs

### Requirement: Safe named diagram creation
The application SHALL create a new BPMN or data-graph diagram under the authorized workspace using a user-provided name and SHALL never overwrite an existing filesystem item.

#### Scenario: Create in workspace root
- **WHEN** the user submits a valid unused name, a supported diagram type, and the workspace root as the destination
- **THEN** the system creates a valid starter document with the canonical extension for that type inside the workspace root

#### Scenario: Create in a displayed folder
- **WHEN** the user submits a valid unused name, a supported diagram type, and a displayed workspace folder as the destination
- **THEN** the system creates the diagram directly inside that folder

#### Scenario: Name omits an extension
- **WHEN** the user enters a valid name without a file extension
- **THEN** the system appends the canonical extension for the selected type

#### Scenario: Name or destination is invalid
- **WHEN** the submitted name is empty, contains a path separator, conflicts with the selected type, or the destination is no longer an existing directory inside the workspace
- **THEN** the system creates no file and reports an actionable validation error

#### Scenario: Destination item already exists
- **WHEN** an item with the resolved filename already exists or appears concurrently in the destination folder
- **THEN** the system preserves the existing item, creates no replacement, and asks the user to choose another name or destination
