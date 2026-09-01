# Software Schematic Bootstrap

## Purpose

Provide a versioned, self-contained project-local CLI, wrapper, and loopback application runtime for Software Schematic workspaces.

## Requirements

### Requirement: Cross-platform single-executable CLI
The system SHALL distribute `ss` as one self-contained native executable for each supported Windows and macOS target, and the executable SHALL provide the `init` command without requiring a language runtime or package manager.

#### Scenario: Run the CLI on a supported target
- **WHEN** a user invokes the matching `ss` executable on Windows or macOS
- **THEN** the CLI runs without requiring Node.js, Rust, or another separately installed runtime

### Requirement: Project initialization
The `ss init` command SHALL initialize the current directory with `.ss/`, `schematics/`, and platform wrapper launchers, including a versioned runtime, bundled web assets, `schematics/main.cmmn`, and `schematics/main.md`.

#### Scenario: Initialize an empty project
- **WHEN** a user runs `ss init` at the root of a project where none of the target paths exist
- **THEN** the command creates the complete runnable Software Schematic workspace and records the installed tool version

#### Scenario: Initialization target collides
- **WHEN** `.ss/`, `schematics/`, or a generated wrapper path already exists
- **THEN** initialization stops without overwriting the existing path and reports the collision

### Requirement: Project-local wrapper launch
The generated `ssw` launcher on macOS and `ssw.cmd` launcher on Windows SHALL invoke the runtime pinned in `.ss/bin/` for the containing project.

#### Scenario: Launch from the project root
- **WHEN** a user executes the platform wrapper from an initialized project root
- **THEN** the wrapper starts that project's pinned Software Schematic runtime with the project root as its workspace

### Requirement: Local application serving
The runtime SHALL bind an available loopback port, serve the bundled application and typed schematic operations, and open the application URL in the default browser.

#### Scenario: Wrapper starts successfully
- **WHEN** the project runtime can bind a loopback port and read its initialized assets
- **THEN** it serves the application on `127.0.0.1` and opens the resulting URL in the default browser

### Requirement: Workspace path confinement
The local server SHALL normalize every schematic operation path and SHALL reject any path that does not resolve beneath the initialized project's `schematics/` directory.

#### Scenario: Request attempts to escape schematics
- **WHEN** an application request contains an absolute path or traversal that resolves outside `schematics/`
- **THEN** the server rejects the operation without reading or writing the target

### Requirement: Bundled CMMN assets
The `ss init` output SHALL include the pinned local CMMN modeler runtime, styles and required vendor assets, the registered SSW CMMN moddle descriptor, and a starter CMMN document in the self-contained `.ss/` installation. Browser-executable assets SHALL be produced by the existing web build from the HTML, CSS, and TypeScript source and SHALL require no runtime package manager or CDN.

#### Scenario: Initialized project opens CMMN offline
- **WHEN** a newly initialized project opens a `.cmmn` file without network access
- **THEN** the extracted `.ss/` assets load and edit the diagram without requesting external code, styles, fonts, or services

### Requirement: CMMN-rooted initialization
Adding CMMN support SHALL initialize `schematics/main.cmmn` as the sole project anchor and SHALL NOT create a competing `schematics/main.bpmn`. The root CMMN SHALL be the project entry for domain, actors, inputs, outputs, needs, business services, and Process Task links to BPMN logical architecture and solution building blocks.

#### Scenario: New project starts normally
- **WHEN** `ss init` completes after this change
- **THEN** the project starts from `schematics/main.cmmn`, no root BPMN exists, and BPMN designs are created or opened from CMMN Process Tasks

#### Scenario: Legacy BPMN-only project starts
- **WHEN** an existing project has `schematics/main.bpmn` and no `schematics/main.cmmn`
- **THEN** the editor opens the legacy BPMN root as a compatibility fallback without creating a second anchor

#### Scenario: Competing roots are detected
- **WHEN** both `schematics/main.cmmn` and `schematics/main.bpmn` exist
- **THEN** startup reports competing anchors instead of choosing one silently
