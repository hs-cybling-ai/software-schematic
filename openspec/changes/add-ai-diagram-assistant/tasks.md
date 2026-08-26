## 1. Assistant Contracts

- [x] 1.1 Define the versioned node-scoped and diagram-scoped context snapshot types, normalization rules, revision hashes, size limits, and fixtures.
- [x] 1.2 Define the versioned assistant proposal schema with summaries, warnings, stable IDs, confined paths, and the initial allowed operation variants.
- [x] 1.3 Implement shared schema and semantic validators for request correlation, supported versions, operation limits, IDs, paths, BPMN compatibility, and node statuses.
- [x] 1.4 Add deterministic validation tests covering valid plans, malformed output, unsupported operations, duplicate IDs, path escape, locked nodes, and stale revisions.

## 2. Rust Provider Boundary

- [x] 2.1 Add an `AssistantProvider` trait and normalized request/result types that are independent of any provider SDK.
- [x] 2.2 Add a deterministic fake provider with success, invalid-output, delayed/cancelled, authentication-failure, and timeout fixtures.
- [x] 2.3 Add confined assistant configuration that selects an allowlisted HTTPS provider/model, reads credentials from the host environment, and redacts secrets from errors and logs.
- [x] 2.4 Add the assistant-proposal endpoint with request/response limits, concurrency control, timeouts, cancellation, correlation, and no mutation side effects.
- [x] 2.5 Implement the initial OpenAI Responses API adapter using the strict operation-plan contract without exposing credentials to the browser.
- [x] 2.6 Add Rust contract and endpoint tests for fake/OpenAI request translation, configuration failures, cancellation, redaction, limits, and invalid provider responses.

## 3. Magic Entry Points and Dialog

- [x] 3.1 Add an accessible magic action to eligible BPMN node context palettes and exclude connections, labels, roots, and unsupported elements.
- [x] 3.2 Add an accessible magic action to the persistent diagram palette that scopes assistance to the complete active diagram.
- [x] 3.3 Build one shared modal dialog with explicit scope/provider identification, prompt input, context disclosure, submit/cancel, progress, errors, preview, approve/reject, and focus restoration.
- [x] 3.4 Wire request cancellation and dialog teardown so late provider results cannot reopen or mutate dismissed UI.

## 4. Context Collection

- [x] 4.1 Build deterministic semantic graph extraction for active BPMN nodes, flows, labels, composition references, and live status hints.
- [x] 4.2 Collect active diagram Markdown and node Markdown according to invocation scope after flushing relevant pending edits.
- [x] 4.3 Compute source revisions, enforce context limits, disclose optional truncation, and reject requests that cannot retain required diagram structure.
- [x] 4.4 Add browser tests for node and full-diagram snapshots, pending-edit flushing, normalization, size boundaries, and revision changes.

## 5. Preview and Approval

- [x] 5.1 Convert validated operations into a human-readable preview grouped by diagram and documentation file with assumptions and warnings.
- [x] 5.2 Revalidate source revisions, live IDs, paths, BPMN rules, operation limits, and locked-node constraints immediately before approval.
- [x] 5.3 Ensure reject, close, invalid, stale, and failed proposals leave diagrams, Markdown, files, tabs, and selection unchanged.
- [x] 5.4 Add UI tests for preview contents, explicit approval, rejection, stale proposals, locked-node failures, and keyboard/focus behavior.

## 6. Diagram and Documentation Execution

- [x] 6.1 Implement supported `bpmn-js` executors for type replacement, label updates, external composition linkage, flow-node creation, and sequence-flow creation.
- [x] 6.2 Implement confined composition create/open and diagram/node Markdown replacement executors using existing canonical tabs and persistence operations.
- [x] 6.3 Capture complete before-state, preflight every operation, stage multi-diagram changes, and roll back all affected modelers and documents on any failure.
- [x] 6.4 Group successful mutations into coherent per-diagram undo steps and add an assistant-level revert action guarded against conflicting later revisions.
- [x] 6.5 Add integration tests for task-to-subprocess conversion, four-step child flow creation, coordinated Markdown updates, canonical tab behavior, persistence, undo, rollback, and revert.

## 7. End-to-End Verification

- [x] 7.1 Add a browser/Rust end-to-end scenario using the fake provider for both node-scoped and full-diagram modification suggestions.
- [x] 7.2 Verify no provider credential appears in browser assets, project metadata, `.ss` output, diagram or Markdown files, previews, errors, or captured logs.
- [x] 7.3 Run web and Rust test suites, production asset and release builds, and manual keyboard/browser verification of both magic entry points and the task-to-subprocess example.
- [x] 7.4 Document provider configuration, transmitted context, supported operations, limits, cancellation, approval, recovery, and the future Claude/Gemini/MCP extension boundary.

## 8. Local Agent Authentication

- [x] 8.1 Add wrapper and Rust CLI commands for local-agent login, status, logout, discovery, and explicit provider selection.
- [x] 8.2 Add Codex and Claude Code providers that reuse official cached account authentication and require the canonical structured plan with tools disabled.
- [x] 8.3 Persist only the selected provider, expose it in safe project metadata, and identify it in the assistant disclosure.
- [x] 8.4 Add configuration/security tests and document the account-authenticated workflow.
