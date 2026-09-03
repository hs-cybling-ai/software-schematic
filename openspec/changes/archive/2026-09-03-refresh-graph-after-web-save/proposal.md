## Why

The running MCP graph becomes stale after a person evaluates and saves a diagram or connected Markdown correction in the Software Schematic web application. The document-to-code path should refresh immediately while preserving the diagram and Documentation as the only human-authored source of truth.

## What Changes

- After the web application successfully saves dirty CMMN/BPMN XML or connected Markdown, trigger a project-local graph update in the running MCP process.
- Rebuild and atomically publish the derived in-memory graph, retaining the last valid snapshot if the newly saved document set is invalid.
- Report graph-refresh failure in the web application's save status without undoing or hiding the successful document save.
- Remove the LLM-facing `reload_model` MCP tool so Codex can query the compiled graph but cannot trigger graph changes or mutate diagrams or Documentation.
- Do not add a user shell command, file watcher, graph mutation tool, locking workflow, or automatic source edit.

## Capabilities

### New Capabilities

- `document-save-graph-refresh`: One-way refresh from successful human-authored web document saves to the running project MCP graph.

### Modified Capabilities

- `software-schematic-editor`: Extend automatic diagram and Markdown persistence to refresh the derived MCP graph and surface refresh status.

## Impact

- Affects the Rust web-save handlers, MCP snapshot lifecycle, a small project-local internal signal, and web save-status presentation.
- Narrows the MCP tool surface by removing `reload_model`; all remaining MCP tools are read-only queries.
- Preserves the intended flow: person evaluates and edits documents, SSW compiles them, and Codex implements only from the resulting graph.
