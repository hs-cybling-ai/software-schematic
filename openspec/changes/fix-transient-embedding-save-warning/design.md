## Context

See `proposal.md` for motivation. The Markdown write path persists the body, schedules embedding derivation, and independently queues a graph refresh. That first refresh is intentionally allowed to publish a text-only snapshot. `collect_artifact_vectors` records missing, stale, and invalid embedding artifacts in the graph summary's generic warning list; `refresh_from_documents` then concatenates every warning into `GraphRefreshOutcome.diagnostic`. The browser treats any diagnostic on an `updated` or `unchanged` outcome as a warning and shows a toast, even while `/api/embedding-status` correctly reports `queued` or `processing`.

The implementation must retain early text-only publication, asynchronous compare-and-swap header generation, and visibility of real graph or embedding failures. It must also remain compatible with existing headerless projects and external Markdown writers.

## Goals / Non-Goals

**Goals:**

- Represent graph publication and vector readiness as independent typed state.
- Prevent expected missing/stale-header readiness from becoming an actionable graph warning during normal derivation.
- Preserve source-specific diagnostics for corrupt artifacts and operational failures.
- Cover the body-save and derived-header publications as one observable workflow.

**Non-Goals:**

- Making Markdown saves wait for embedding generation.
- Removing text-only retrieval or the two-stage publication sequence.
- Hiding embedding failures, malformed artifacts, structural graph warnings, or refresh failures.
- Changing the embedding envelope, hashing, chunking, or compare-and-swap contract.

## Decisions

### 1. Classify graph diagnostics at their source

Graph diagnostics will carry a stable machine-readable category/code in addition to their human-readable message and source. Embedding artifact states such as missing, stale, malformed, or incompatible will be distinguishable from structural graph warnings.

Refresh construction will use the typed category rather than message matching. Expected vector unavailability will populate vector-readiness detail and will not be copied into the refresh's actionable `diagnostic`; unrelated structural warnings and build failures retain their current warning/failure behavior.

String matching in the browser was rejected because wording changes, localization, or multiple joined warnings would make it fragile and could accidentally suppress a real failure.

### 2. Put retrieval readiness on the refresh outcome

Successful `updated` and `unchanged` outcomes will include the published snapshot's retrieval mode, embedding revision, and vector readiness derived from the replacement graph summary. The existing lifecycle status continues to describe graph work, while these fields describe what the published graph can currently retrieve.

The existing `/api/embedding-status` response remains the authority for derivation-job lifecycle (`queued`, `processing`, `current`, `stale`, or `failed`). The browser correlates both responses: a text-ready graph plus a queued/processing job is normal progress; a failed job is degraded and actionable.

Overloading `GraphRefreshStatus` with embedding states was rejected because a graph can be successfully updated and text-ready at the same time that vectors are pending.

### 3. Toast only actionable outcomes

The save-status control will continue to show concise pending/current/degraded labels. A successful graph outcome will show a toast only for an actionable graph diagnostic, not merely because retrieval mode is `text` or vector readiness is pending. Embedding failure and graph-refresh failure retain actionable status text, title details, and toast behavior where currently provided.

The UI will consume structured fields rather than infer state from the diagnostic message. This keeps a headerless legacy project and a just-saved Markdown document behaviorally consistent without silencing unrelated warnings.

### 4. Test the ordered state transition

Rust coverage will verify that a body save first publishes an `updated` or `unchanged` text-ready outcome with vectors pending and no actionable missing-header diagnostic, then that derived-header publication produces current/hybrid readiness. Separate cases will assert that structural graph warnings, corrupt artifact diagnostics, derivation failures, and refresh failures remain observable.

Web tests will exercise save-state rendering with paired graph-refresh and embedding-status payloads and assert both the visible label and whether a toast is emitted. Packaged assets will be rebuilt after source tests pass so the distributed editor uses the same behavior.

## Risks / Trade-offs

- **[Adding fields can leave older clients unaware of readiness]** → Keep existing refresh lifecycle and diagnostic fields compatible; new fields are additive and the bundled client handles absent values conservatively.
- **[Over-classifying vector diagnostics could hide corrupt external artifacts]** → Preserve corrupt/incompatible details in structured vector diagnostics and show them when no active derivation is expected or when derivation fails; suppress only the expected pending transition.
- **[Polling can observe graph and embedding endpoints between transitions]** → Render each response pair conservatively, keep polling while either subsystem is pending, and rely on the next poll to converge without emitting a transient error.
- **[Generated web assets can drift from source]** → Include the normal web build/package verification in the implementation tasks and test the packaged contract.

## Migration Plan

1. Add typed diagnostic classification and additive retrieval-readiness fields to graph summaries/refresh outcomes.
2. Update the bundled web client to render combined graph and embedding state and gate toasts on actionable diagnostics.
3. Add Rust and web regression tests, rebuild packaged assets, and run focused plus release verification.

Rollback restores the prior response consumption and diagnostic aggregation; no project files, Markdown envelopes, or persisted revisions require migration.
