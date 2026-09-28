# Model-driven AI development

Software Schematic is a lightweight local design loop for one developer and an AI coding assistant:

```text
interview → diagram → plan (or ▶ Play) → versioned work items → code
```

Run `./ssw`. This starts one small project-local process and opens the diagram UI. Codex or Claude connects through the project-installed `./ssw mcp` entry and sees the same current diagram workflow.

## Design deeply, visualize early

Invoke the project `design` skill. The AI interviews you about goals, actors, boundaries, important data, states, integrations, trust, happy paths, alternatives, failures, recovery, and operational constraints.

The interview should not become a long prose specification. As soon as one slice is clear, the AI sends a small proposal to the open diagram. Approve it, reject it, or reshape the diagram yourself. The next interview turn uses the current saved diagram, so you never copy prompts or element IDs between the chat and UI.

Repeat until the model feels right.

## Create a scoped plan

The green **Play** button saves every pending open diagram/document edit, waits for graph publication, and creates an immutable build plan for the active diagram or selected element. The `plan` skill performs the same operation from a natural-language node or label.

Plans are stored under `.ss/workflows/builds/<plan-id>/` with an immutable manifest plus compare-and-swap progress and evidence. The UI shows the concise scope label, semantic version, and state. New scopes start at `1.0.0`; changed scopes require Major, Minor, or Fix; unchanged incomplete or failed plans resume.

The `build` skill then:

1. lists open plans or retrieves an exact plan ID or `scope@version`;
2. claims one dependency-ready item;
3. loads focused authority plus connected/downward context and explicit exclusions;
4. implements and checks that item while recording physical-effect evidence;
5. repeats sequentially before final integration verification.

If an unrelated authoritative diagram/document change is published during a build, the plan becomes stale. A plan-declared generated edge/event contract is expected self-output and does not stale its own plan.

## Small operational surface

- `./ssw` opens or reuses the project runtime.
- `./ssw mcp` is the AI client's stdio adapter.
- `./ssw doctor` reports whether the project integration is ready.
- `./ssw doctor --repair` restores managed wrapper, MCP, guidance, and skill files.
- `./ssw stop` stops the local runtime.

The runtime keeps bounded interview/proposal state and versioned build plans. It does not store provider credentials or full chat transcripts. Diagrams and owned Markdown remain the logical contract; claim-bound `docs/<element-id>-contract.md` files record the implemented edge/event contract separately when working code justifiably differs.
