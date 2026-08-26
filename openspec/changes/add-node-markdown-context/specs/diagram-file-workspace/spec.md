## MODIFIED Requirements

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
