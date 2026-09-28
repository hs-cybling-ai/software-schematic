## 1. Workspace Entry Contracts and Model Projection

- [x] 1.1 Add credential-free design-workspace, launch-state, design-target, and proposal-summary protocol types with bounded validation, and verify serialization tests prove that tokens, authenticated URLs, absolute project paths, and browser-session IDs cannot appear in success or failure responses.
- [x] 1.2 Add a current-snapshot projection for the root anchor and at most 50 deterministic process candidates, including existing BPMN compositions and named CMMN Process Task anchors without a composition, and verify unit tests cover stable identity, anchored deduplication, ordering, missing compositions, and exclusion of orphan diagrams.
- [x] 1.3 Add current-interview and current-non-terminal-proposal summary accessors to the workflow store, and verify recovery tests return only the one resumable project-local record without exposing proposal operations through the summary.

## 2. Secure Project Runtime Browser Opening

- [x] 2.1 Introduce an injectable system-browser launcher and set-once loopback browser base URL in `ProjectRuntime`, wire it before MCP traffic is accepted, and verify runtime construction tests cover configured, missing, and duplicate base-URL initialization without launching a real browser.
- [x] 2.2 Implement the runtime workspace-entry method with session expiry, current-generation reuse, internal authenticated URL construction, a three-second bounded heartbeat wait, and credential-free fallback diagnostics; verify async tests cover already connected, opened-and-connected, still-opening, stale-session, launcher-failure, and wrong-project/authentication cases.
- [x] 2.3 Extend `WorkflowBackend` and its test doubles with the workspace-entry contract, and verify existing interview, proposal, and Play workflow tests continue to pass unchanged.

## 3. `open_design_workspace` MCP Tool

- [x] 3.1 Add the zero-argument `open_design_workspace` tool that merges runtime entry state with current project identity, revisions, root context, candidates, and resumable workflow state, and verify MCP tests cover complete bounded responses for new, existing, and in-progress designs.
- [x] 3.2 Advertise `guidedDesignWorkspace` through capability discovery and update the constrained tool-surface expectations, verifying tool-list/schema tests include the new operation and still exclude raw XML, arbitrary filesystem, launch-credential, and proposal-approval inputs.
- [x] 3.3 Add daemon/MCP integration coverage proving a headless MCP-first project reuses one runtime, requests the matching browser workspace through an injected launcher, and never attaches to or returns state from another project.

## 4. Guided Design Skill Experience

- [x] 4.1 Rewrite the generated `design` skill to discover the new capability, call `open_design_workspace`, verify project identity, resume a non-terminal proposal, prefer fresh visual context, resume an interview, or present root/existing/new process choices in that order; verify adapter snapshot tests assert the same workflow for Codex and Claude.
- [x] 4.2 Add outcome-first new-process guidance that establishes actors and boundary before proposing a need anchor and composition, and verify skill-content tests prohibit asking users for diagram paths, raw IDs, raw XML, or BPMN/CMMN expertise.
- [x] 4.3 Add adaptive completeness guidance using the existing interview decisions and unresolved fields for goals, paths, recovery, data/state, integrations, trust, operations, exclusions, documentation, and implementation status; verify skill-content tests require rereading after every approved/manual edit, explicit completion confirmation, and waiting for Play without implementation.
- [x] 4.4 Update initialization, update, and doctor/repair fixtures so refreshed projects receive the guided skill, and verify managed-file tests preserve diagrams, Markdown, workflow records, and unrelated project configuration.

## 5. Documentation and End-to-End Verification

- [x] 5.1 Update the diagram-delivery and project-local MCP documentation to describe automatic runtime reuse, secure browser opening, process orientation, launch fallback, and the completion-to-Play journey; verify documented commands and capability names match the generated skill and MCP schema.
- [x] 5.2 Run Rust formatting plus the focused delivery protocol, graph projection, runtime, MCP wrapper, and managed-skill tests, then run the complete Rust and web test suites; verify all commands succeed and no test opens a real browser.
- [x] 5.3 Initialize a temporary project with the built CLI and exercise `open_design_workspace` through MCP using a controlled launcher, verifying the response identifies the temporary project, presents the root/existing/new design paths, resumes persisted state, omits launch secrets, and leaves authored model files unchanged until browser-approved proposals occur.
