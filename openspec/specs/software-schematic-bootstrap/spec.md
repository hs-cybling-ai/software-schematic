# Software Schematic Bootstrap

## Purpose

Provide a versioned, self-contained project-local CLI, wrapper, and loopback application runtime for Software Schematic workspaces.
## Requirements
### Requirement: Cross-platform single-executable CLI
The maintained source distribution SHALL build `ss` as one self-contained native executable for each supported Windows and macOS target. Each executable SHALL provide the `init`, `update`, `serve`, authentication, and MCP commands without requiring a language runtime or package manager after installation, and SHALL carry the project license, notice, required third-party notices, bundled web application, starter documents, and embedding assets needed by those commands.

#### Scenario: Run the CLI on a supported target
- **WHEN** a user invokes the matching `ss` executable on Windows or macOS
- **THEN** the CLI runs without requiring Node.js, Rust, or another separately installed language runtime

#### Scenario: Build the CLI from the public source tree
- **WHEN** a contributor follows the documented clean-checkout release workflow on a supported build host
- **THEN** the workflow produces the self-contained executable entirely from retained source, lockfiles, and bundled assets without any retired native-editor or Data Graph directory

#### Scenario: Inspect an installed distribution
- **WHEN** a user initializes or updates a project with the release executable
- **THEN** the installed runtime contains the applicable Software Schematic license, Cybling Labs notice, required third-party notices, browser assets, templates, and model assets

### Requirement: Project initialization
`ss init` SHALL install the project wrapper, bundled UI/runtime, project MCP entry, managed guidance, and `design`, `graph`, and `build` skills. Generated configuration SHALL use project-relative commands and preserve unrelated settings.

#### Scenario: Developer initializes a project
- **WHEN** `ss init` succeeds
- **THEN** running `./ssw` opens the diagram tool and the configured AI client can discover the model-driven workflow tools

#### Scenario: Initialize an empty project
- **WHEN** a developer runs `ss init` in a project without Software Schematic paths
- **THEN** it creates the runnable local workspace, project MCP entry, managed guidance, and the three workflow skills

#### Scenario: Existing agent instructions are present
- **WHEN** initialization finds user-authored `AGENTS.md` content outside the managed sentinels
- **THEN** it changes only the Software Schematic block and preserves the user content

#### Scenario: Managed guidance is applied twice
- **WHEN** the same managed guidance is installed again
- **THEN** `AGENTS.md` contains one current managed block and is otherwise unchanged

#### Scenario: Existing Codex project configuration is present
- **WHEN** initialization finds unrelated valid entries in `.codex/config.toml`
- **THEN** it changes only the project-local Software Schematic MCP entry

#### Scenario: Initialization target collides
- **WHEN** a required Software Schematic path already exists during first-time initialization
- **THEN** initialization stops without overwriting it and reports the collision

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
Update and doctor repair SHALL replace only managed wrapper, runtime, MCP, guidance, and skill assets while preserving authored diagrams, Markdown, code, and unrelated host configuration.

#### Scenario: Managed skill is missing
- **WHEN** `./ssw doctor --repair` runs
- **THEN** the missing skill is restored and no enterprise health or migration workflow is required

#### Scenario: Existing project is updated
- **WHEN** a developer runs the project update or repair path
- **THEN** managed runtime and AI integration files become current while diagrams, Markdown, code, and unrelated configuration remain unchanged

### Requirement: Workflow-focused generated skills
The generated design skill SHALL conduct the deep iterative interview, the graph skill SHALL publish current durable diagram changes, and the build skill SHALL respond to Play with revision-bound implementation and evidence.

#### Scenario: Developer clicks Play after interviewing
- **WHEN** the AI client is waiting on the project workflow
- **THEN** the generated skills carry current diagram scope into implementation without copied prompts or identifiers

### Requirement: Codex project activation guidance
Initialized project documentation SHALL explain that project `.codex/config.toml` is scoped to the repository, that Codex must trust or activate the repository configuration, and that a fresh Codex task may be required after registration changes. It SHALL instruct users to verify the Software Schematic MCP project overview before development.

#### Scenario: User opens the initialized project in Codex
- **WHEN** Codex has not yet activated the repository's project configuration
- **THEN** the documentation provides the activation and verification steps without recommending a global Software Schematic MCP registration
