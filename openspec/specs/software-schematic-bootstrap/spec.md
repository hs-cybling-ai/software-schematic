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
The `ss init` command SHALL initialize the current directory with `.ss/`, `schematics/`, platform wrapper launchers, a managed project `.codex/config.toml` MCP entry, and a managed Software Schematic block in root `AGENTS.md`, including a versioned runtime, bundled web and embedding assets, `schematics/main.cmmn`, and `schematics/main.md`. The MCP entry SHALL launch the pinned project wrapper and derive the project root from that wrapper. The managed instructions SHALL require project-identity verification, treat diagrams and Markdown as the detailed contract, send natural proposal language to MCP, record the resolved root identity, link every task to authorized nodes, and implement only returned `new` or `modify` scope.

#### Scenario: Initialize an empty project
- **WHEN** a user runs `ss init` at the root of a project where none of the required Software Schematic paths exist
- **THEN** the command creates the complete runnable workspace, records the installed tool version, registers its project MCP in `.codex/config.toml`, and installs managed guidance in `AGENTS.md`

#### Scenario: Existing agent instructions are present
- **WHEN** initialization or an explicit Software Schematic project update encounters an `AGENTS.md` with user-authored content outside the managed sentinels
- **THEN** it creates or replaces only the sentinel-delimited Software Schematic block and preserves the unrelated content

#### Scenario: Managed guidance is applied twice
- **WHEN** the same guidance version is installed more than once
- **THEN** `AGENTS.md` contains exactly one current managed block and otherwise remains unchanged

#### Scenario: Existing Codex project configuration is present
- **WHEN** initialization or update encounters `.codex/config.toml` with unrelated project settings or MCP servers
- **THEN** it creates or replaces only the managed Software Schematic MCP entry and preserves all unrelated valid configuration

#### Scenario: Initialization target collides
- **WHEN** `.ss/`, `schematics/`, or a generated wrapper path already exists during first-time initialization
- **THEN** initialization stops without overwriting the existing runtime or schematic path and reports the collision

### Requirement: Project-local wrapper launch
The generated `ssw` launcher on macOS and `ssw.cmd` launcher on Windows SHALL invoke the runtime pinned in `.ss/bin/` for the containing project and SHALL route an `mcp` invocation to the pinned runtime's project-local stdio MCP command.

#### Scenario: Launch from the project root
- **WHEN** a user executes the platform wrapper without the `mcp` command from an initialized project root
- **THEN** the wrapper starts that project's pinned Software Schematic application runtime with the project root as its workspace

#### Scenario: MCP client launches the project wrapper
- **WHEN** an MCP client executes the generated wrapper with `mcp`
- **THEN** the wrapper starts that project's pinned Software Schematic MCP runtime with the project root as its workspace and does not open a browser

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

### Requirement: Existing-project agent guidance update
The system SHALL provide an explicit, idempotent project update path that installs the current project-local Codex MCP entry, managed Software Schematic `AGENTS.md` block, and required MCP runtime assets in an already initialized compatible project without rewriting schematic content or unrelated Codex configuration or agent instructions.

#### Scenario: Existing project is updated
- **WHEN** a user invokes the documented update path in a compatible initialized project
- **THEN** the pinned runtime, MCP assets, project Codex registration, wrapper routing, and managed guidance are current while authored diagrams, Markdown, unrelated Codex configuration, and unrelated `AGENTS.md` content are preserved

### Requirement: Codex project activation guidance
Initialized project documentation SHALL explain that project `.codex/config.toml` is scoped to the repository, that Codex must trust or activate the repository configuration, and that a fresh Codex task may be required after registration changes. It SHALL instruct users to verify the Software Schematic MCP project overview before development.

#### Scenario: User opens the initialized project in Codex
- **WHEN** Codex has not yet activated the repository's project configuration
- **THEN** the documentation provides the activation and verification steps without recommending a global Software Schematic MCP registration

