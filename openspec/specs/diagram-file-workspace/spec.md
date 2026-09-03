# Diagram File Workspace

## Purpose

Define authorized diagram discovery, canonical tab identity, safe persistence, and unsaved-change protection within a folder workspace.
## Requirements
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
The application SHALL recursively display folders and supported BPMN and `.dgraph` diagram files beneath the authorized root, SHALL sort folders before files by localized name, SHALL exclude unsupported files including legacy ArchiMate files from the visible diagram tree, and SHALL treat a valid `.context.sqlite` database referenced by a Data Graph diagram as that diagram's hidden companion rather than as a separately openable workspace item. The application SHALL recognize the former `.dgraph.json` extension only for explicit migration and SHALL create and export new Data Graph diagrams with `.dgraph`.

#### Scenario: Folder contains mixed content
- **WHEN** the selected folder contains nested supported diagrams, referenced SQLite context databases, and unrelated files
- **THEN** the tree displays the folder hierarchy and supported BPMN and `.dgraph` diagrams but not unrelated files, ArchiMate files, or context databases as separate items

#### Scenario: Legacy extension is migrated
- **WHEN** the user explicitly migrates `orders.dgraph.json` and neither `orders.dgraph` nor `orders.context.sqlite` conflicts
- **THEN** the application creates the logical `orders.dgraph` and paired `orders.context.sqlite`, opens the migrated diagram, and preserves its topology

#### Scenario: Workspace content changes
- **WHEN** a supported diagram or its referenced context database is added, removed, renamed, or changed while the workspace is open
- **THEN** the system refreshes or invalidates the affected logical diagram while preserving valid selection and open tabs and reports a broken companion reference when applicable

### Requirement: Open diagram identity
The application SHALL open a supported diagram from the tree in at most one tab per canonical file URL.

#### Scenario: User opens a closed diagram
- **WHEN** the user activates a supported diagram in the tree
- **THEN** the system reads the file and opens it in a selected editor tab matching its format

#### Scenario: User opens an already open diagram
- **WHEN** the user activates a diagram whose canonical URL already has a tab
- **THEN** the system selects the existing tab instead of creating a duplicate

### Requirement: Safe persistence
The application SHALL export editor content and atomically replace the original file only after a successful export, SHALL clear dirty state only after all required writes succeed, and SHALL coordinate Data Graph context capture by committing a complete SQLite revision before atomically updating the diagram's authoritative revision reference. A failure after a SQLite commit SHALL leave the prior referenced revision authoritative and SHALL NOT report capture success.

#### Scenario: User saves an edited diagram without context changes
- **WHEN** the active tab is dirty only because topology changed and the user invokes Save
- **THEN** the system exports the diagram, preserves its authoritative context revision, atomically writes the diagram file, and marks the tab clean

#### Scenario: User captures node context
- **WHEN** the active Data Graph tab has valid Markdown drafts and the user invokes Capture
- **THEN** the system generates required embeddings, commits content and vectors in one complete SQLite revision, atomically writes the diagram manifest referencing that revision, and marks the captured context clean

#### Scenario: Export or write fails
- **WHEN** export, SQLite capture, embedding generation, or atomic diagram replacement fails
- **THEN** the system preserves the original diagram and prior authoritative SQLite revision, keeps applicable state dirty, and reports the failure

#### Scenario: Dataset commits but diagram write fails
- **WHEN** a complete SQLite revision commits but atomic replacement of the diagram manifest fails
- **THEN** the prior manifest revision remains authoritative, the tab remains dirty, and the system reports the unreferenced revision as recoverable cleanup rather than a successful capture

#### Scenario: Source changed externally
- **WHEN** the on-disk diagram or its authoritative SQLite revision has changed since it was loaded and the user attempts to save or capture
- **THEN** the system requires an explicit conflict decision before overwriting, capturing, or reloading either part of the logical document

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

### Requirement: Created diagram discovery and opening
The application SHALL refresh workspace discovery after successful creation and SHALL open the created diagram through the canonical tab workflow.

#### Scenario: Creation succeeds
- **WHEN** the starter document is written successfully
- **THEN** the system displays the new diagram in the file tree, opens it in a selected editor tab matching its format, and leaves the tab clean

#### Scenario: Opening fails after creation
- **WHEN** the starter document is created but refresh or editor loading fails
- **THEN** the system keeps the created file on disk and reports an actionable error without disrupting other tabs

