## ADDED Requirements

### Requirement: Folder-based base diagrams
The system SHALL represent each composition folder with `main.bpmn` as its base diagram and `main.md` as its diagram documentation, with `schematics/main.bpmn` and `schematics/main.md` as the root composition.

#### Scenario: Root composition opens
- **WHEN** the initialized application starts
- **THEN** it opens `schematics/main.bpmn` and associates `schematics/main.md` with the diagram

### Requirement: Element documentation convention
The system SHALL store documentation for a BPMN element at `docs/<element-id>.md` within the composition folder containing that element's base diagram.

#### Scenario: Document an element for the first time
- **WHEN** the user edits documentation for an element that has no Markdown file
- **THEN** the server creates the composition folder's `docs/` directory and writes `<element-id>.md`

### Requirement: External subprocess reference
The editor SHALL use BPMN call activities as external subprocesses and SHALL store a normalized `schematics/`-relative composition folder in the call activity's standard `calledElement` value.

#### Scenario: Configure an external subprocess
- **WHEN** the user assigns a valid composition path to a call activity
- **THEN** the BPMN model stores that normalized relative path as `calledElement`

### Requirement: External subprocess navigation and creation
The application SHALL resolve an external subprocess to `<calledElement>/main.bpmn` when its call activity is double-clicked, SHALL create the composition folder with starter `main.bpmn` and `main.md` when absent, and SHALL focus the resolved diagram's canonical tab.

#### Scenario: Double-click an existing external subprocess
- **WHEN** the user double-clicks a call activity whose composition folder exists
- **THEN** the application opens or focuses that folder's `main.bpmn` tab

#### Scenario: Double-click a new external subprocess
- **WHEN** the user double-clicks a call activity whose valid composition folder does not exist
- **THEN** the server creates the folder, starter base diagram, and diagram Markdown before the application opens and focuses its tab

### Requirement: Reusable process references
The application SHALL allow multiple call activities in any composition folder to reference the same normalized external subprocess folder.

#### Scenario: Two flows reuse a process
- **WHEN** two call activities have the same `calledElement` composition path
- **THEN** both resolve to the same canonical `main.bpmn` tab and documentation files

### Requirement: Pool and lane composition folders
The system SHALL map a pool to `<parent>/<pool-id>/` and a lane within that pool to `<parent>/<pool-id>/<lane-id>/`, and SHALL create each mapped folder with `main.bpmn` and `main.md` when the user first opens it as a composition.

#### Scenario: Open a pool composition
- **WHEN** the user opens a pool as a composition and its mapped folder is absent
- **THEN** the system creates `<parent>/<pool-id>/main.bpmn` and `<parent>/<pool-id>/main.md` and focuses the base diagram

#### Scenario: Open a lane composition
- **WHEN** the user opens a lane as a composition and its mapped folder is absent
- **THEN** the system creates `<parent>/<pool-id>/<lane-id>/main.bpmn` and `<parent>/<pool-id>/<lane-id>/main.md` and focuses the base diagram
