## Why

Play currently persists one revision-bound build request, but it does not produce a discoverable, resumable per-item work plan or enforce the selected service/function boundary throughout implementation. A versioned build plan created either visually through Play or conversationally through a new `plan` skill will let Codex or Claude resume later, select an open build with a short ordinal, implement only authorized `new`/`modify` scope, and record where working code justifiably differs from the logical design.

## What Changes

- Replace the single current Play request as the build handoff with an immutable, semantically versioned build plan stored under `.ss/workflows/builds/<plan-id>/`, plus separate mutable progress and evidence records.
- Give each selected scope an independent semantic version. The first plan is `1.0.0`; later Play actions explicitly choose major, minor, or fix/patch advancement, while an unchanged retry reuses its existing version.
- Publish the created plan version and scope label in the diagram UI, and show current plan state there.
- Remove the user-facing `graph` skill and replace it with a `plan` skill. The `plan` skill accepts a natural-language description of a node, label, pool, participant, process, or composed item in the reachable diagram stack; resolves or disambiguates that scope; ensures saved source is represented by a current published graph; and creates exactly the same versioned plan as the Play button.
- Let the `build` skill list open builds as a short numbered menu, accept either that invocation-local number or an exact scope/version selector, and resume the chosen plan without requiring the user to remember long identifiers.
- Derive build targets only from the selected boundary: a selected pool authorizes its contained service and downward compositions; a selected node authorizes that component, its contained/composed implementation when applicable, its edges, events, and connected items. Scope resolution SHALL NOT traverse upward into parent services.
- Restrict implementation targets to diagram entities marked `new` or `modify`. `open` and `locked` entities may supply context but cannot authorize features or implementation work.
- Permit necessary physical code, configuration, migration, and test effects needed to deliver working in-scope software even when those effects do not map one-to-one to the logical diagram, while forbidding unrelated features, replay behavior, unmodeled alternate flows, and new stubbed or duplicate services.
- Make the build skill execute a deterministic resumable work queue one item at a time, using the other in-scope and connected entities as context. The plan is declarative data; Play SHALL NOT generate executable shell, Python, Codex, or Claude scripts.
- After implementation, write generated `docs/<element-id>-contract.md` files for implemented edges and events. These implementation contracts remain separate from authored logical `docs/<element-id>.md`, are indexed in the graph as implementation evidence, and may document a workable divergence without silently changing design intent or expanding scope.
- Treat declared generated-contract publication as an expected completion transition rather than making a plan stale from its own output; unrelated authoritative model changes still require revalidation.

## Capabilities

### New Capabilities

- `versioned-build-plans`: Defines semantic plan identity, `.ss` persistence, selected-boundary scope derivation, immutable work plans, progress/evidence state, discoverability, and lifecycle rules.

### Modified Capabilities

- `ai-diagram-assistance`: Play creates a selected-scope versioned plan and publishes its version and state to the UI.
- `diagram-delivery-skills`: The portable suite becomes `design`, `plan`, and `build`; `plan` is the conversational equivalent of Play, and `build` lists, selects, resumes, and executes plan work items sequentially without requiring remembered identifiers or generated scripts.
- `project-runtime-daemon`: The runtime retains bounded build-plan history, progress, and evidence rather than only one current Play request.
- `schematic-mcp`: The MCP surface exposes bounded build-plan discovery, selection, work-item transitions, context retrieval, and generated implementation-contract submission.
- `schematic-property-graph`: The graph distinguishes authored logical Markdown from generated edge/event implementation-contract Markdown and links both to the same modeled entity.
- `document-save-graph-refresh`: Generated implementation-contract publication refreshes the graph and records the completion revision without falsely staling the plan that authorized it.

## Impact

This changes the Play/build protocol, persisted workflow schemas, MCP tool surface, generated Codex and Claude skill suite, graph Markdown ownership model, and browser build-status UI. Managed `graph` skill files are removed or replaced during update/repair, while graph synchronization remains a runtime/MCP capability used internally. Existing `.ss/workflows/play.json` records require a bounded migration or compatibility reader. No new runtime dependency or generated executable format is introduced, and authored diagrams and logical Markdown remain the source of implementation authority.
