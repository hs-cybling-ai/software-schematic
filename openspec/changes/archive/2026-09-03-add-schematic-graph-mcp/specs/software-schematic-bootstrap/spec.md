## MODIFIED Requirements

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

## ADDED Requirements

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
