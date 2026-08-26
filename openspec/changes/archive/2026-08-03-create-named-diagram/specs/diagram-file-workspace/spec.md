## ADDED Requirements

### Requirement: Safe named diagram creation
The application SHALL create a new diagram of any supported format under the authorized workspace using a user-provided name and SHALL never overwrite an existing filesystem item.

#### Scenario: Create in workspace root
- **WHEN** the user submits a valid unused name, a supported diagram type, and the workspace root as the destination
- **THEN** the system creates a valid starter document with the canonical extension for that type inside the workspace root

#### Scenario: Create in a displayed folder
- **WHEN** the user submits a valid unused name, a supported diagram type, and a displayed workspace folder as the destination
- **THEN** the system creates the diagram directly inside that folder

#### Scenario: Name omits an extension
- **WHEN** the user enters a valid name without a file extension
- **THEN** the system appends the canonical extension for the selected diagram type

#### Scenario: Name or destination is invalid
- **WHEN** the submitted name is empty, contains a path separator, conflicts with the selected type, or the destination is no longer an existing directory inside the workspace
- **THEN** the system creates no file and reports an actionable validation error

#### Scenario: Destination item already exists
- **WHEN** an item with the resolved filename already exists or appears concurrently in the destination folder
- **THEN** the system preserves the existing item, creates no replacement, and asks the user to choose another name or destination

### Requirement: Created diagram discovery and opening
The application SHALL refresh workspace discovery after successful creation and SHALL open the created diagram through the canonical tab workflow.

#### Scenario: Creation succeeds
- **WHEN** the starter document is written successfully
- **THEN** the system displays the new diagram in the file tree, opens it in a selected editor tab matching its format, and leaves the tab clean

#### Scenario: Opening fails after creation
- **WHEN** the starter document is created but refresh or editor loading fails
- **THEN** the system keeps the created file on disk and reports an actionable error without disrupting other tabs
