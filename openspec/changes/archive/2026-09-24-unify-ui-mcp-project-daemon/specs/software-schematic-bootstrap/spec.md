## MODIFIED Requirements

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

### Requirement: Existing-project agent guidance update
Update and doctor repair SHALL replace only managed wrapper, runtime, MCP, guidance, and skill assets while preserving authored diagrams, Markdown, code, and unrelated host configuration.

#### Scenario: Managed skill is missing
- **WHEN** `./ssw doctor --repair` runs
- **THEN** the missing skill is restored and no enterprise health or migration workflow is required

#### Scenario: Existing project is updated
- **WHEN** a developer runs the project update or repair path
- **THEN** managed runtime and AI integration files become current while diagrams, Markdown, code, and unrelated configuration remain unchanged

## ADDED Requirements

### Requirement: Workflow-focused generated skills
The generated design skill SHALL conduct the deep iterative interview, the graph skill SHALL publish current durable diagram changes, and the build skill SHALL respond to Play with revision-bound implementation and evidence.

#### Scenario: Developer clicks Play after interviewing
- **WHEN** the AI client is waiting on the project workflow
- **THEN** the generated skills carry current diagram scope into implementation without copied prompts or identifiers
