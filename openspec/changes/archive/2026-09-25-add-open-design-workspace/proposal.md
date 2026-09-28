## Why

Invoking the project-local `design` skill can start the correct SSW runtime through MCP, but it does not reliably bring the developer into the browser modeling workspace or orient them to the processes already present. The design experience should hide runtime startup, make the diagram immediately available, and guide a developer from any current model state through process selection or creation to a complete, Play-ready model.

## What Changes

- Add an `open_design_workspace` MCP operation that starts or reuses the matching project runtime, reports whether a browser modeling session is connected, and provides a secure client action for opening or focusing the project workspace without placing the daemon token in ordinary assistant prose.
- Return bounded design-entry context containing project identity, the active browser diagram and selection when available, the root business anchor, resumable named process candidates, current interview/proposal state, and current graph/source revisions.
- Make the `design` skill call `open_design_workspace` first, verify the project, direct the developer into the visual model, and resume the active diagram or present meaningful choices to refine the business model, resume an existing process, or design a new process.
- Guide new process modeling from the desired outcome and actors rather than requiring BPMN/CMMN terminology, then use the existing revision-bound proposal and browser-only approval workflow to create the need anchor and composition incrementally.
- Maintain a bounded, adaptive completeness assessment across goals, actors, boundaries, paths, recovery, data/state, integrations, trust, operations, exclusions, documentation, and implementation statuses; finish by summarizing unresolved items and directing the developer to Play only when they confirm the model is ready.
- Preserve the existing security and authority boundaries: one loopback runtime per project, no raw XML or arbitrary writes, no AI proposal approval, no implementation before Play, and no launch credential persisted in interview state or exposed as normal chat content.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `diagram-delivery-skills`: Expand the `design` skill into a guided, resumable journey that opens the modeling workspace, orients the user to existing processes, and drives the selected model toward explicit completion.
- `schematic-mcp`: Add the bounded `open_design_workspace` entry operation and its project, browser-session, process-candidate, and workflow-state response contract.
- `project-runtime-daemon`: Allow an authenticated project-local MCP client to request a secure browser open/focus action against the existing daemon while preserving project isolation and launch-token confidentiality.

## Impact

- The generated Codex and Claude `design` skill content changes in `software-schematic-cli/src/delivery_runtime.rs`.
- The MCP router and delivery protocol gain request/response types and an `open_design_workspace` tool.
- The project runtime gains workspace-entry context assembly and a secure browser-launch path that reuses the current daemon.
- Graph/model queries must identify bounded resumable process candidates and root-anchor context without creating another index or specification store.
- Tests will cover headless-first launch, an already connected browser, process selection and creation guidance, persisted interview/proposal resumption, cross-project rejection, token non-disclosure, and completion-to-Play behavior.
