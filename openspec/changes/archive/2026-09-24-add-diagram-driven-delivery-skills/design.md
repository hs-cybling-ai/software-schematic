## Context

The repository already models software intent in CMMN/BPMN plus ID-bound Markdown, compiles those files into a queryable graph, resolves natural language to implementation-eligible nodes, and supports structured AI proposals in the browser. Save-triggered refresh and a separate optimization change are moving graph work off the save critical path, but agent guidance still treats diagram mutation as UI-only and current delivery depends on an additional proposal system.

The new skills must work in both Codex and Claude Code, preserve project confinement and atomic snapshots, tolerate either surface editing the same files, and make the diagram graph—not a copied prose plan—the authoritative contract.

## Goals / Non-Goals

**Goals:**

- Define one resumable `design` → `graph` → `build` protocol with portable skill text and thin host adapters.
- Share proposal validation and mutation semantics between chat and the browser.
- Prevent lost updates across UI and agent edits with durable revision preconditions.
- Make the common graph refresh proportional to the changed document set and provide proof of publication before build.
- Bind every implementation session to an explicit graph revision and authorized entity set.

**Non-Goals:**

- General collaborative real-time editing or CRDT synchronization.
- Allowing skills to issue arbitrary graph queries, mutations, shell commands through the MCP, or unrestricted file writes.
- Removing OpenSpec support in the same release; the skills must become credible before migration guidance deprecates it.
- Treating generated code as automatically accepted solely because it matches an eligible node.

## Decisions

### 1. Use a shared workflow protocol, not UI-only or skill-only diagramming

`design` is the canonical orchestration contract: interview state, context snapshot, typed proposal, preview, approval, application, and resume token. The web UI consumes the same protocol and provides a native visual entry point; Codex and Claude adapters translate their host conventions into it. Interview state contains stable entity references and revision tokens, not hidden mutable model state.

This keeps the no-friction chat path while preserving the diagram UI for spatial review and manual adjustment. Building only in the UI would interrupt chat-led work; building only as a skill would discard the strongest review surface and create divergent mutations.

The design interview's sole mutation output is a structured diagram change set. A versioned operation registry is generated from the shared schema and declares the BPMN/CMMN element/property combinations the editor can preview, execute through modeler services, undo, and roll back. Operations cover create, update/replace, move, connect/disconnect, remove, composition linkage, status, and diagram/node/edge Markdown. The skill may discuss any design freely, but it cannot lower an unsupported request into raw XML, arbitrary paths, or best-effort prose; it must keep interviewing, split the change, or report the capability gap.

The structured output follows a deliberate information hierarchy:

- Diagram labels optimize for human scanning and communicate business intent, process phase, function call, event, or transferred data in a few words.
- Activity Markdown owns operational detail. When one cohesive activity contains multiple steps, Markdown enumerates them; the label does not become a miniature procedure.
- Message-edge Markdown is the authoritative message contract. The edge's short name identifies the call or data, while Markdown carries endpoint or transport, producer/consumer, direction, payload, correlation, delivery, security, acknowledgement, retry/timeout, and failure semantics.
- Event Markdown owns trigger and event-contract details including data, timing, ordering, idempotency, correlation, delivery, security, and error handling.
- Diagram-level Markdown explains scope, purpose, actors, boundaries, and cross-cutting constraints rather than duplicating element contracts.

The interview coverage engine evaluates these ownership rules before declaring a design build-ready. Missing contract fields become focused questions or explicitly approved unresolved decisions. This keeps diagrams legible while ensuring the graph retains implementation-grade detail through element-owned Markdown.

### 2. Use content revisions and compare-and-swap for concurrency

Every authoritative diagram or Markdown read returns a digest-based revision over normalized complete bytes. Every write includes `expectedRevision`; the server performs atomic compare-and-swap and returns `newRevision`. A multi-document proposal captures the complete affected revision set, stages all operations, rechecks every precondition immediately before commit, and either commits the coordinated set or reports conflicts without overwriting.

Filesystem modification times may help discover candidates, but never establish equality or ordering: their resolution varies, clocks can differ, and file replacement can preserve or perturb timestamps. A persistent watcher may notify the UI that external content changed, but correctness rests on content revisions.

For conflicts, XML proposals are regenerated or semantically reapplied through modeler operations. Markdown may use a deterministic three-way merge from base/local/current; any overlap requires explicit preview and approval. There is no silent last-writer-wins path.

### 3. Make graph synchronization changed-document and receipt based

The graph service maintains a source manifest mapping canonical paths to content revisions and cached owner/dependency metadata. The `graph` skill compares the manifest to durable state, submits only changed identities, and waits on an operation ID. The refresh coordinator coalesces path updates, reads files itself, selects affected fragments using reverse dependencies, validates a complete staging snapshot, and atomically publishes it.

The receipt binds operation ID, source-manifest revision, graph revision, processed paths, and outcome (`unchanged`, `incremental`, or `fullFallback`). Markdown-owner changes normally rebuild annotations/embeddings for one owner; diagram changes rebuild the diagram and reverse-dependent closure. Rename, delete, reachability change, ambiguity, validation failure, or unsupported formats fall back to the complete loader. Incremental/full equivalence tests are the safety oracle.

The skill-facing MCP call is a constrained synchronization request, not general graph mutation: callers identify durable documents and revisions, while the service owns reading, validation, compilation, and publication.

### 4. Treat build scope as an immutable capability grant

`build` first verifies project identity, invokes natural-language scope resolution, and either confirms one high-confidence eligible root or asks the user to select from candidates. It then requests a bounded implementation-context bundle for the chosen `new`/`modify` closure at a specific graph revision. The bundle includes annotations, relevant edges, citations, source-manifest revision, and exclusions (`open`/`locked`).

Before each material edit phase and at completion, the skill revalidates the grant. A graph change unrelated to the grant may proceed; a changed authorized contract pauses implementation. This prevents code from drifting from a diagram edited mid-build without requiring the whole repository to remain frozen.

### 5. Store small, inspectable handoffs rather than duplicate specifications

A project-local handoff file records schema version, project ID, phase, source-manifest and graph revisions, entity URNs, operation receipt, and verification summary. It contains no provider credentials, chat transcript, or copied full requirements. Skills can reconstruct current context from the graph and annotations, keeping the diagram authoritative and minimizing stale prose.

Host adapters package equivalent SKILL/command instructions and call the same local APIs. A conformance fixture runs identical scenarios against both adapters to catch drift.

### 6. Migrate incrementally from the current planning loop

Initially, `build` may emit an optional compatibility summary for existing OpenSpec consumers, but its native contract is the revision-bound node scope. Project guidance prefers the new skills once end-to-end conformance, conflict, and equivalence tests pass. OpenSpec files and commands remain available until projects can complete design, synchronization, implementation, and recovery without them.

## Risks / Trade-offs

- **[Multi-document atomicity is difficult across filesystem replacement]** → Stage complete files under confined paths, lock the affected document set in canonical order, recheck revisions, replace as one server-coordinated transaction, and restore captured before-state on failure.
- **[Incremental invalidation omits a dependent fragment]** → Use conservative dependency closure, common validation, full-build fallback, and normalized incremental/full equivalence tests.
- **[Host skill behavior drifts]** → Generate adapters from shared workflow material and run cross-host conformance fixtures.
- **[Structured output advertises more than the editor can perform]** → Derive discovery from the registered validators/executors and require preview/apply/undo/rollback conformance for every advertised operation and supported element type.
- **[Concise labels hide necessary implementation detail]** → Require owned Markdown coverage for activities, events, and message edges and expose that content through graph context, search, and citations.
- **[Contracts become duplicated and inconsistent]** → Assign each contract to the narrowest authoritative owner—edge, event, or activity—and keep diagram prose focused on scope and cross-cutting context.
- **[Long builds become stale]** → Revalidate only the granted contract and dependency neighborhood at phase boundaries, pausing when semantic inputs changed.
- **[Frequent conflict prompts create friction]** → Keep writes short, flush UI drafts before proposals, coalesce graph work, auto-merge only provably non-overlapping Markdown, and preserve user work when intervention is necessary.
- **[Skill-triggered refresh weakens the read-only MCP boundary]** → Accept only project-confined document identities and durable revisions; the service reads sources and controls all graph compilation and publication.

## Migration Plan

1. Add strong document revisions, conditional typed writes, external-change detection, and conflict UI without changing current graph behavior.
2. Refactor browser AI assistance behind the shared proposal/interview protocol and prove UI behavior remains compatible.
3. Add source manifests, constrained changed-document synchronization, publication receipts, and incremental/full equivalence coverage; coordinate with the existing background-refresh change rather than duplicating its scheduler.
4. Ship `graph`, then `design`, then `build` with shared fixtures and both host adapters; retain compatibility output during adoption.
5. Update bootstrap and managed guidance to prefer the three skills after end-to-end acceptance tests pass. Deprecate, but do not immediately remove, the separate proposal workflow.

Rollback can disable skill installation and incremental selection while retaining conditional writes and using the full background loader. Versioned handoffs and protocols allow older clients to fail closed with upgrade guidance.

## Open Questions

- Choose the final project-local handoff filename and retention count during implementation, provided it remains confined, inspectable, non-sensitive, and disposable.
- Establish performance budgets from representative repositories for changed-document publication and bounded implementation-context retrieval.
