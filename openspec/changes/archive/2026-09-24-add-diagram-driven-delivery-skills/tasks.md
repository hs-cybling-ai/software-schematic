## 1. Protocol and Revision Foundations

- [x] 1.1 Define versioned schemas for project identity, strong document revisions, expected-revision writes, source manifests, synchronization operations, publication receipts, implementation-context grants, and skill handoffs; verify schema round-trip and incompatible-version tests pass.
- [x] 1.2 Add digest-based revisions to all typed BPMN, CMMN, and Markdown reads and verify identical bytes produce identical revisions independent of modification time.
- [x] 1.3 Implement atomic conditional writes for each supported document type and verify current revisions succeed while stale revisions preserve the newer durable file and return structured conflicts.
- [x] 1.4 Implement canonical-order locking, staging, precondition recheck, and recovery for coordinated multi-document proposals; verify an injected mid-commit failure leaves every affected document at its before-state.

## 2. Editor Concurrency and Shared Proposals

- [x] 2.1 Update browser persistence state to retain base revisions and send expected revisions, and verify normal autosave advances revisions without changing current pending/saved/failed behavior.
- [x] 2.2 Add external-change detection and a conflict UI that preserves local work and offers reload or previewed reapplication for diagram XML; verify a simulated agent edit cannot be silently overwritten.
- [x] 2.3 Add base/local/current comparison and previewed three-way merge for non-overlapping Markdown while requiring explicit resolution for overlaps; verify both automatic-candidate and overlapping-conflict cases.
- [x] 2.4 Extract the assistant context, typed plan, validator, preview, approval, executor, and revert behavior into one versioned shared design protocol; verify existing browser AI-assistance tests pass through the shared path.
- [x] 2.4a Define a versioned structured diagram-change schema and discoverable operation registry covering supported create, update/replace, move, connect/disconnect, remove, composition, status, and diagram/node/edge annotation operations; verify every advertised BPMN/CMMN operation has validator, preview, modeler executor, undo, and coordinated rollback conformance tests.
- [x] 2.5 Add resumable interview state and coverage checks for actors, triggers, outcomes, alternate/failure paths, contracts, boundaries, dependencies, non-functional constraints, acceptance evidence, and status; verify unresolved material decisions yield focused questions rather than mutations.
- [x] 2.5a Add endpoint- and system-boundary interview coverage for method/operation, route or topic, caller, receiver, payload purpose, trust boundary, success, timeout/retry, and failure path; verify confirmed details become activity/message-flow Markdown with concise Labels.
- [x] 2.5b Add annotation-ownership and completeness checks for activity operations, message contracts, and event contracts; verify concise Labels, enumerated activity steps in Markdown, short message-edge names, and edge/event-owned contract fields across BPMN fixtures.
- [x] 2.6 Expose the shared interview/proposal workflow to project-local skill clients and verify a chat-originated proposal can be reviewed in the UI and is rejected after a conflicting UI revision.

## 3. Incremental Graph Synchronization

- [x] 3.1 Extend graph build state with a canonical source manifest, content revisions, owner/dependency metadata, cached parsed fragments, and reusable annotation embeddings; verify a complete load produces a deterministic manifest and graph revision.
- [x] 3.2 Implement the confined changed-document synchronization request and operation-status API, including project identity and durable-revision validation; verify out-of-project paths, stale revisions, and caller-supplied graph content are rejected.
- [x] 3.3 Route editor and skill changes through one project-scoped coalescing coordinator and verify successive revisions of one path eventually publish the newest accepted durable revision.
- [x] 3.4 Implement owner-scoped Markdown refresh and verify unrelated fragments and embeddings are reused while updated search results cite the new annotation revision.
- [x] 3.5 Implement diagram and reverse-dependency-closure refresh with full-rebuild fallback for rename, delete, reachability, unsupported, or ambiguous changes; verify each fallback returns its reason in the publication receipt.
- [x] 3.6 Publish only validated complete snapshots and return correlated `unchanged`, `incremental`, or `fullFallback` receipts; verify failed builds retain the last-known-good graph and never issue a success receipt.
- [x] 3.7 Build incremental/full equivalence fixtures covering entities, relationships, annotations, citations, search, scope resolution, and deterministic revision, and verify all fixture comparisons pass.

## 4. Shared Skill Runtime and Host Adapters

- [x] 4.1 Create the shared skill runtime contract and project-local handoff store with schema version, project identity, workflow phase, revisions, entity references, receipts, and verification metadata; verify foreign-project and stale handoffs fail closed.
- [x] 4.2 Implement project capability discovery and compatibility diagnostics and verify unsupported MCP or document protocol versions cause no writes and return upgrade guidance.
- [x] 4.3 Create Codex and Claude Code adapters from the shared workflow material and verify cross-host conformance fixtures produce equivalent requests, decisions, and handoffs.
- [x] 4.4 Add CLI/bootstrap installation and update support for all three skills and both hosts, and verify a clean initialized fixture contains valid host-specific skill packages without credentials or machine-specific absolute paths.

## 5. Design Skill

- [x] 5.1 Implement bounded interview orchestration that resumes from current diagram annotations and asks only material unresolved questions; verify multi-round state survives a host restart through the handoff record.
- [x] 5.2 Translate interview outcomes into the shared typed CMMN/BPMN and Markdown proposal schema with stable entity references and appropriate annotation ownership; verify fixtures cover diagram-, node-, and edge-level documentation.
- [x] 5.2a Constrain design output to the discovered diagram operation registry and verify unsupported requests yield an explicit capability gap instead of raw XML, unrestricted paths, prose-only mutations, or approximate operations.
- [x] 5.2b Add composition recommendations and structured parent/child operations for cohesive subflows or mixed abstraction levels; verify traversal changes conversational scope to the child while retaining parent navigation.
- [x] 5.3 Add semantic preview, explicit approval, conditional coordinated application, graph synchronization, and resume from the publication receipt; verify reject, stale approval, successful apply, revert, and refresh-failure paths.
- [x] 5.4 Add a browser entry point for continuing the same design interview and verify users can move from chat to visual review and back without copying prompts or identifiers.
- [x] 5.5 Replace the one-shot diagram prompt with a persistent multi-turn interview that separates explanatory answers from mutation proposals; verify bulk simple node naming with per-node Markdown, targeted `save data` documentation, and clarified S3 pool/message-flow scenarios.
- [x] 5.6 Add interview guidance that rejects procedure-like Labels and routes detailed steps and contract data to the narrowest activity, message-edge, or event Markdown owner; verify a four-step activity and S3 message contract produce readable diagrams with implementation-grade annotations.

## 6. Graph Skill

- [x] 6.1 Implement manifest comparison that selects changed durable documents by strong revision rather than modification time and verify touched-but-identical files produce an `unchanged` result.
- [x] 6.2 Implement synchronization submission, bounded status waiting, diagnostic reporting, and publication receipt persistence; verify one Markdown change requests only that canonical document and returns the including graph revision.
- [x] 6.3 Add recovery guidance for unavailable service, stale source revisions, failed publication, and full-rebuild fallback and verify each failure leaves authoritative files and the last valid graph unchanged.

## 7. Build Skill

- [x] 7.1 Implement natural-language development-scope resolution with confidence thresholds, explicit candidate selection, and `new`/`modify` eligibility enforcement; verify clear, ambiguous, and no-eligible-node fixtures.
- [x] 7.2 Implement revision-bound bounded implementation-context retrieval containing annotations, topology, citations, authorized targets, and exclusions; verify obsolete revisions and `open`/`locked` targets are rejected.
- [x] 7.3 Implement the build execution loop that plans edits against node references, authors only in authorized scope, runs repository-relevant checks, and records code and verification evidence per entity; verify an end-to-end fixture produces traceable output without a separate prose spec.
- [x] 7.4 Revalidate the scope grant before material edit phases and completion and verify an unrelated graph update can continue while an authorized contract change pauses further writes.
- [x] 7.5 Add optional OpenSpec compatibility summary output for migration and verify the native workflow remains operable when compatibility output is disabled.

## 8. End-to-End Adoption and Documentation

- [x] 8.1 Add an end-to-end scenario that starts with a chat interview, reviews and manually adjusts the proposal in the UI, resolves the resulting stale revision, incrementally publishes the graph, selects a node in natural language, and builds verified code; verify no identifier or requirement text is manually copied between phases.
- [x] 8.2 Add load and latency tests for a large project and verify changed-document synchronization and save acknowledgement remain within established budgets while a deliberately slow full builder runs off the interaction path.
- [x] 8.3 Document the three-skill workflow, concurrency model, conflict recovery, graph fallback behavior, host installation, and migration from OpenSpec/Speckit; verify commands and screenshots/examples against a clean fixture.
- [x] 8.4 Update managed agent guidance to prefer `design`, `graph`, and `build` only after conformance and end-to-end gates pass, and verify legacy OpenSpec commands remain available for rollback during the migration window.
- [x] 8.5 Run the complete Rust, web, protocol, skill-conformance, strict OpenSpec, and end-to-end test suites and record passing commands and any accepted performance baselines in the change evidence.
