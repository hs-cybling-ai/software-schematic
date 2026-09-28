# Diagram-driven development skills

`ss init` installs the same three small, project-local workflows for Codex and Claude Code:

- `design` conducts a deep interview and sends frequent, focused changes to the live diagram for review.
- `plan` resolves a diagram node or label from natural language and creates the same scoped, versioned plan as Play.
- `build` lists open plans and implements their dependency-aware work items one at a time.

After initialization, start the project-local `design` skill. Its first `open_design_workspace` call starts or reuses this project's SSW runtime through MCP and securely asks the system browser to open the modeling workspace. The authenticated local URL stays inside the runtime. If the operating system cannot open a browser, the skill gives the normal `./ssw` (macOS) or `ssw.cmd` (Windows) fallback instead of printing the authenticated URL.

The skill orients you before changing the model. It first resumes any proposal awaiting browser review, then prefers a fresh diagram selection or durable interview. When no target is established, it offers the root business model, named existing processes, and a new process as human-readable choices. Starting a new process begins with the outcome, actors, and system boundary; you do not need to provide diagram paths, element IDs, or BPMN/CMMN notation.

Answer one focused question at a time, approve or reject each small visual proposal in the browser, and freely reshape the diagram yourself. After every approved proposal or manual edit, the next design turn reads the saved model and reassesses material gaps across paths, recovery, data and state, integrations, trust, operations, exclusions, documentation, and implementation status. The assessment is adaptive rather than a fixed questionnaire.

When the diagram is right, the skill summarizes the modeled scope and asks you to confirm that it feels complete. Then either click **Play** for the current browser selection or invoke `plan` and describe the intended node or label. Both paths flush all open diagram/document saves, wait for graph publication, and call the same planner. A new scope starts at `1.0.0`; a changed prior scope requires an explicit Major, Minor, or Fix bump; an unchanged incomplete or failed plan resumes.

Scope stays inside the selected boundary. A pool includes its service descendants and downward compositions. A node includes contained items, attached events, incident edges, one connected layer, and eligible downward compositions. Only `new` and `modify` become work; `open` and `locked` are context. Necessary shared wiring, configuration, schemas, migrations, and tests are allowed, but unrelated features, replay behavior, alternate flows, speculative stubs, and duplicate services are not.

Invoking `build` without a selector shows a short numbered list of open plans. After selection it loops claim → focused context → implementation → checks → evidence for one item at a time. It does not create task-launching scripts. Built edges and events may publish `docs/<element-id>-contract.md`, preserving the logical Markdown while recording the actual implementation contract and visible drift.

The generated files are `.codex/skills/{design,plan,build}/SKILL.md` and `.claude/commands/{design,plan,build}.md`. Run `./ssw doctor --repair` to restore recognized managed files, the wrapper, or project MCP registration. Locally modified adapters are preserved with a diagnostic. Plans live under `.ss/workflows/builds/`; the saved diagram remains the logical contract.
