## ADDED Requirements

### Requirement: Cross-platform single-executable CLI
The system SHALL distribute `ss` as one self-contained native executable for each supported Windows and macOS target, and the executable SHALL provide the `init` command without requiring a language runtime or package manager.

#### Scenario: Run the CLI on a supported target
- **WHEN** a user invokes the matching `ss` executable on Windows or macOS
- **THEN** the CLI runs without requiring Node.js, Rust, or another separately installed runtime

### Requirement: Project initialization
The `ss init` command SHALL initialize the current directory with `.ss/`, `schematics/`, and platform wrapper launchers, including a versioned runtime, bundled web assets, `schematics/main.bpmn`, and `schematics/main.md`.

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
