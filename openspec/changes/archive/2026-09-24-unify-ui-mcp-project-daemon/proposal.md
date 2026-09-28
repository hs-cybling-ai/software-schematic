## Why

Software Schematic is a local developer tool for model-driven AI development. Its core experience is a deep design interview, fast visual iteration on the diagram, and a single **Play** action that tells the AI to build the code from the accepted model. Separate browser and MCP runtimes made that loop awkward, but the solution must stay small: one developer, one project process, one current design workflow.

## What Changes

- Run the browser UI and MCP tools from one lightweight project-local process started by `./ssw`.
- Make the `design` skill lead a thorough interview and submit small, frequent diagram proposals into the live browser.
- Let the developer approve, reject, manually adjust, and continue interviewing without copying prompts or diagram identifiers.
- Add a prominent **Play** control that records the current diagram revision and wakes the AI to build code from the model.
- Keep only safeguards that directly protect the loop: project identity, local-only transport, source revisions, typed modeler operations, atomic saves, rollback, and last-known-good graph publication.
- Make `ss init`, update, and a small doctor command install and verify the local wrapper, MCP entry, and `design`, `graph`, and `build` skills.
- Remove multi-user, multi-browser, load-management, enterprise health classification, legacy migration, and broad lifecycle requirements.

## Capabilities

### New Capabilities

- `project-runtime-daemon`: One small project-local runtime shared by the UI and the AI coding client.

### Modified Capabilities

- `ai-diagram-assistance`: Deep interview, rapid proposal/adjustment cycles, and Play-to-build handoff.
- `schematic-mcp`: Thin project-local access to design, graph, and build workflow operations.
- `document-save-graph-refresh`: Direct refresh after accepted durable diagram changes.
- `software-schematic-bootstrap`: One-command setup of the wrapper, MCP entry, and three workflow skills.

## Impact

The runtime remains local and project-scoped. The main product surface becomes: interview deeply, iterate visually, click Play, build against the published model.
