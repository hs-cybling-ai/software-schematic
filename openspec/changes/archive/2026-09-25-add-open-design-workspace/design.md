## Context

See `proposal.md` for motivation. The project MCP adapter already starts or attaches to one headless project runtime, and the runtime already owns the loopback URL, random token, daemon generation, browser heartbeat session, current durable interview, current proposal, and current Play request. The generated `design` skill currently assumes that a live browser is available but has no entry operation that can open it or return the orientation needed to resume/select a process.

`SchematicMcp` owns the current graph snapshot and delegates workflow state to an attached `WorkflowBackend`, implemented by `ProjectRuntime`. `ProjectRuntime` currently constructs authenticated launch URLs only in the top-level `serve` path; it does not retain a browser base URL or expose a credential-free workspace-opening method to the MCP surface. Parsed diagram snapshots already retain the information needed to derive named Process Task anchors and reachable BPMN designs without adding a second index.

## Goals / Non-Goals

**Goals:**

- Give the generated `design` skill one bounded entry call that securely opens or resumes the correct project workspace and returns enough context to guide the next conversational step.
- Keep the launch token and authenticated URL entirely inside the runtime while still supporting headless-first MCP startup.
- Prefer live browser context and durable workflow state, then offer deterministic root/existing/new process choices when no target is established.
- Reuse current graph, parsed-diagram, interview, proposal, and browser-session state rather than creating another handoff or requirements store.
- Make the skill adaptive but completion-aware, ending only with explicit developer confirmation and Play.

**Non-Goals:**

- Remote-controling browser UI gestures, guaranteeing operating-system window focus, or introducing a general-purpose browser automation tool.
- Deep-linking to arbitrary untrusted paths or accepting a project, URL, token, diagram path, or process ID from the tool caller.
- Changing browser-only proposal approval, conditional persistence, graph publication, or Play-to-build authority.
- Adding a second graph index, process catalog, transcript store, or mandatory fixed questionnaire.
- Implementing code as part of `design` or automatically clicking Play.

## Decisions

### 1. Make `open_design_workspace` a zero-argument project-bound MCP tool

The MCP adapter is already bound to a canonical initialized project and authenticated to its one daemon. The new tool therefore accepts no project root, URL, token, or target path. It combines a runtime workspace-entry snapshot with the current schematic snapshot and returns a `DesignWorkspace` response containing:

- protocol version and `ProjectIdentity`;
- graph and source-manifest revisions;
- a root-anchor target;
- credential-free browser state (`alreadyConnected`, `openedAndConnected`, `openRequested`, or `failed`), optional active project-relative diagram, and selected entity URNs;
- at most 50 deterministic process candidates;
- the current bounded `InterviewRecord`, when present; and
- a proposal resume summary containing only proposal ID, diagram, summary, state, and state revision.

The full proposal remains available through `get_diagram_proposal`. The workspace response never includes proposal operations, unrestricted document bodies, a browser session ID, an absolute project path, the daemon token, or the authenticated launch URL.

This is preferable to returning a Markdown URL because an authenticated URL would place the daemon bearer token in model context, task history, and potentially logs. It is preferable to making the skill run `./ssw` because the MCP tool already knows the exact project runtime and can report structured launch/session state consistently across Codex and Claude.

### 2. Keep browser launch authority inside `ProjectRuntime`

Add a small injectable browser-launch abstraction and an `Arc<OnceLock<String>>` browser base URL to `ProjectRuntime`. After binding the loopback HTTP listener, `serve` sets the base URL before attaching the workflow backend and accepting MCP traffic. The production launcher uses the existing system-browser mechanism; tests inject a recording or failing launcher.

Extend `WorkflowBackend` with a workspace-entry method returning a runtime-only snapshot of launch state, fresh active diagram/selection, current interview, and current non-terminal proposal summary. `SchematicMcp::open_design_workspace` delegates to that method, then adds graph-derived project/root/process context.

Before deciding whether to launch, the runtime expires a session using the existing heartbeat timeout. A fresh session is reused without another OS open request. Otherwise the runtime constructs the authenticated URL locally from the base URL, token, project ID, and generation, invokes the launcher, and waits up to three seconds for a matching browser heartbeat. The serialized result records only whether launch was requested and whether a session connected. An OS launch failure returns `failed`; a successful request without timely registration returns `openRequested` plus the wrapper fallback because the browser may still be loading.

The alternative of moving launch behavior into the MCP proxy was rejected because it would require sending discovery credentials out of the daemon and would duplicate runtime/session logic. Returning an MCP resource link was rejected because the authenticated target would still cross the model/client boundary and client handling is not portable.

### 3. Derive process choices from the current schematic snapshot

Add a bounded snapshot helper that reads the retained parsed diagrams and compiled relationships to produce deterministic design targets. Candidates include reachable BPMN diagram entities and named CMMN Process Task anchors, including a named anchor whose BPMN composition has not yet been materialized. Each candidate carries a stable anchor or diagram URN, source ID when applicable, owner Name, human label/Name, project-relative diagram citation when one exists, and `existing` or `notCreated` composition state.

Candidates are deduplicated by semantic process Name, prefer an anchored process over an unanchored duplicate, sort by normalized Name and stable identity, and are capped at 50. The root CMMN anchor is returned separately. No orphan diagram becomes authoritative, and this read model does not alter graph publication or development-scope eligibility.

Using retained parsed snapshots is preferable to rescanning files in the MCP method because it preserves the current validated revision and Name/path confinement rules. A separate persisted process catalog was rejected because it could diverge from the diagram contract.

### 4. Make resume and orientation order explicit in the skill

The generated Codex and Claude skill text will require this sequence:

1. Discover capabilities and require the additive `guidedDesignWorkspace` feature.
2. Call `open_design_workspace` and verify project identity.
3. Resume a non-terminal proposal first because the runtime permits only one active proposal.
4. If a fresh browser selection exists, offer to continue there; when it differs from a durable interview, briefly offer the choice between the selected context and the interview target.
5. Otherwise resume the current interview.
6. Otherwise present human-readable choices: refine the root business model, resume one of the returned processes, or design a new process.
7. For a new process, ask first for desired outcome, actors, and boundary; use supported typed proposals to establish the need anchor and composition rather than asking for a file path or diagram notation.

The skill must not dump IDs or all candidates into prose. When more than a small conversational set exists, it narrows by the user's natural-language goal using existing model search and then presents bounded matches.

### 5. Store adaptive completeness in the existing interview record

The skill uses the existing bounded `decisions` and `unresolved` fields rather than a new persistence format. Stable decision keys cover goal/outcome, actors, boundary, happy path, alternatives, failure/recovery, data/state, integrations, trust, operations, exclusions, documentation, and implementation statuses. A category may be recorded as not applicable when the developer confirms it does not materially affect the process.

These keys guide attention; they are not a mandatory linear questionnaire. After each approved proposal or direct browser edit, the skill rereads the current model, updates only settled decisions and consequential unresolved items, and selects the next question based on structural impact. Completion requires no known material gap plus explicit developer confirmation. The skill then summarizes scope, asks the developer to click Play, and waits through the existing build-request tool.

The alternative of adding a separate checklist record was rejected because `InterviewRecord` already provides bounded resumable state and the saved diagram remains the contract.

### 6. Preserve additive compatibility through capability discovery

Advertise `guidedDesignWorkspace` in `get_delivery_capabilities`. Keep the current protocol version because the new tool and response are additive and no persisted record format changes. Updated skills require the feature before using the entry workflow and return repair/update guidance when it is absent. Older clients continue to use the existing tools.

## Risks / Trade-offs

- **[A system browser request may open a new tab rather than focus an existing window]** → Never launch when a fresh session exists, describe the behavior as open/resume rather than guaranteed focus, and return session state so the skill can guide the user accurately.
- **[Browser startup can exceed the three-second readiness wait]** → Return `openRequested` rather than treating the model as unavailable; allow the skill to proceed with orientation while asking the developer to use the opening workspace, and retain the `./ssw`/`ssw.cmd` fallback.
- **[Parsed anchor candidates and compiled diagram candidates can duplicate a process]** → Deduplicate by validated complete process Name with deterministic anchored-target precedence and cover missing/existing composition cases in tests.
- **[A stale browser heartbeat could misdirect the conversation]** → Apply the current session timeout before reading active context and require the current daemon generation.
- **[A richer skill prompt can become rigid or verbose]** → Persist only settled decisions and material gaps, ask one focused structural question at a time, and treat the completeness categories as adaptive coverage rather than a script.
- **[Launch secrets could leak through errors or debug formatting]** → Keep authenticated URL construction in a non-serializable runtime path, use explicit response DTOs without secret fields, redact launcher errors, and assert token absence in success/error serialization tests.

## Migration Plan

1. Add the response DTOs, process-target projection, runtime launcher abstraction, and workflow-backend entry method behind the new capability feature.
2. Add the MCP tool and tests while leaving existing tools unchanged.
3. Update the generated Codex and Claude design-skill adapters and managed-file repair/update tests.
4. Existing initialized projects receive the new binary, skill content, and MCP registration through the normal `./ssw update` or `./ssw doctor --repair` path; authored diagrams, Markdown, interviews, proposals, and Play state require no migration.
5. Rollback removes the additive tool and restores the prior generated skill text; no authored or persisted schema conversion is required.
