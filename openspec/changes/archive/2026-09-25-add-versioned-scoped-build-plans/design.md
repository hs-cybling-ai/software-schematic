## Context

See `proposal.md` for motivation. The runtime currently persists one `.ss/workflows/play.json` `BuildRequest` containing an active diagram, optional selected XML ID, graph revision, one coarse lifecycle state, and final evidence. Scope resolution can walk the graph in both directions and implementation context can batch up to 100 eligible entities, so neither layer currently preserves the selected containment boundary or a sequential work queue.

The diagram graph is logical rather than a physical code map. A correct implementation may need to touch shared wiring, configuration, schemas, migrations, or tests, but only diagram entities marked `new` or `modify` may authorize product behavior. Authored `<element-id>.md` remains logical design. Generated `<element-id>-contract.md` will describe actual edge/event behavior and therefore becomes authoritative implementation evidence without replacing design intent.

## Goals / Non-Goals

**Goals:**

- Make Play produce a durable, inspectable, host-neutral plan that can resume in a later Codex or Claude invocation.
- Provide a natural-language `plan` skill that creates the identical plan without requiring the developer to navigate back to a diagram selection.
- Give each stable selected scope an independent semantic version while keeping plan discovery conversational and short.
- Preserve the selected service/function boundary through deterministic downward and connected graph traversal.
- Execute one revision-protected work item at a time and retain progress across host or process restarts.
- Allow necessary physical repository effects while preventing logical feature expansion.
- Publish generated edge/event implementation contracts and distinguish them from logical design in the graph and UI.

**Non-Goals:**

- Generating shell, Python, Codex, Claude, or other executable orchestration scripts.
- Retaining a separate user-facing `graph` skill; publication remains a runtime concern.
- Inferring major/minor/fix intent from status colors, code diffs, or LLM judgment.
- Creating a complete physical architecture model or requiring every changed code file to have its own diagram node.
- Automatically rewriting logical diagram Markdown to match implementation.
- Authorizing ancestors, sibling services, replay, alternate flows, or duplicate/stub services that are not in the selected eligible scope.
- Parallel shared-worktree implementation in the first version.

## Decisions

### 1. Replace the singleton request with an immutable plan and mutable journal

The runtime stores:

```text
.ss/workflows/
  current-build.json
  builds/
    <plan-id>/
      manifest.json
      progress.json
      evidence.json
```

`manifest.json` is immutable after atomic creation. It contains schema version, opaque plan ID, project ID, stable scope key and label, semantic version, scope-contract revision, base source-manifest and graph revisions, selection identity, ordered work items, dependency/context references, exclusions, and declared edge/event contract outputs. `progress.json` contains a compare-and-swap revision, plan state, work-item states, active claim, timestamps, and diagnostics. `evidence.json` contains bounded changed paths, checks, per-item physical effects, generated-contract revisions, and the completion graph revision.

`current-build.json` is only a small pointer used by the browser. The canonical history is the build directory. The runtime retains every non-terminal plan and the newest 50 terminal plans, pruning older terminal directories atomically. `.ss` remains ignored by source control and excluded from the schematic source manifest.

This expands the existing runtime-owned workflow store rather than putting plans under `schematics/`. A Markdown plan under `schematics/` would change the source-manifest revision from which it was derived and would duplicate the design contract. A generated executable was rejected because it would be provider-specific, difficult to resume, and unsafe to derive from model content.

### 2. Give Play and `plan` one server-owned creation path

The runtime exposes one `create_build_plan` operation whose normalized input is an exact scope URN, expected graph and source-manifest revisions, and optional semantic bump. The browser resolves its active selection directly. The `plan` skill first verifies project identity and publication state, then uses reachable-model search plus composition breadcrumbs to resolve natural-language intent. A confident match is confirmed conversationally; ambiguous matches become a short numbered list. Orphan diagrams never become candidates.

The implementation is organized as one shared domain path rather than two workflow implementations:

```text
browser HTTP adapter -- exact selection ----┐
                                             ├─ create_build_plan
MCP plan adapter -- natural language resolver┘        │
                                                       ├─ publication gate
                                                       ├─ scope envelope
                                                       ├─ SemVer allocation
                                                       ├─ work-queue ordering
                                                       └─ atomic plan store
```

The shared service reuses the current runtime graph snapshot, source manifest and refresh receipt, exact entity lookup, hybrid search, bounded neighbor traversal, workflow-store atomic JSON helpers, project/protocol validation, and event publication. The existing `request_build` path becomes a compatibility adapter over this service rather than a second creator. Browser JavaScript owns only edit flushing, selection capture, bump interaction, and status rendering. The `plan` skill owns only natural-language resolution/confirmation and calls MCP; both receive the same serialized plan summary.

Codex and Claude skill files continue to be generated from the same Rust constants and conformance fixtures. Their host-specific files differ only in packaging, not plan semantics. This minimizes the places where scope authority, versions, or lifecycle behavior could drift.

The skill does not recreate the old graph workflow. When durable source differs from the active source manifest, it uses the existing constrained synchronization/status operations and waits for publication. If a fresh browser session reports dirty documents, it stops and asks the developer to finish or save them because an external skill cannot safely capture browser-only drafts. Once current, it calls the same `create_build_plan` operation as Play.

`ss init`, `ss update`, and `ss doctor --repair` install `design`, `plan`, and `build` adapters for Codex and Claude and remove only the managed legacy `graph` adapter. The underlying synchronization MCP operations remain for runtime and compatibility use. Keeping two creation implementations was rejected because their scope and version behavior would drift.

### 3. Key semantic versions by stable scope, not globally

The scope key is the selected entity's stable project-scoped URN. A plan is externally addressable as `<scope-key>@<semver>`, while normal users see a human label and version. The first plan is `1.0.0`. For later materially changed scope contracts, Play presents `Major`, `Minor`, and `Fix`; `Fix` maps to SemVer patch. The runtime computes a `scopeContractRevision` from the selected-boundary entities, their statuses and logical Markdown, relevant relationships, and composition identities. It rejects a bump computed from a stale latest version.

If the same scope-contract revision already has an incomplete or failed plan, Play resumes it rather than increasing the version. Versions are never inferred from `new` versus `modify`: those statuses authorize work but do not communicate compatibility. Different scopes may both have `1.0.0`, so MCP lookup by a bare version returns candidates rather than choosing.

The UI shows only the concise scope label, semantic version, and state. The build skill normally calls `list_build_plans`, assigns invocation-local numbers (`1`, `2`, ...), and asks the developer for a number. Exact automation can use plan ID or scope key plus version; the ordinal is never persisted as identity.

### 4. Build a directed scope envelope without ancestor expansion

Plan creation resolves the browser's diagram path and selected source ID to a universal graph identity before persisting anything. It then constructs a bounded scope envelope with explicit relation rules:

1. **Pool/participant selection:** traverse outgoing containment through that participant's process, then outgoing composition from contained composable items. This represents the selected service. The participant may be a boundary without itself becoming a work item.
2. **Node selection:** start with the selected component; traverse outgoing containment for embedded elements; include attached boundary events, incident authored edges, and directly connected endpoints; and follow outgoing composition only through a selected or contained composable node marked `new` or `modify`.
3. **Connected items:** include one direct graph adjacency layer within the current selected service/function boundary. Eligible directly connected items may become work items when required to realize the selected interaction. Their adjacency is not recursively expanded, preventing a sequence-flow walk from swallowing the whole parent process.
4. **No upward traversal:** incoming containment or composition may identify a boundary for diagnostics but can never add an ancestor or sibling as a target.
5. **Eligibility:** every work item, including edge and event work, must have normalized status `new` or `modify`. `open` and `locked` entities are retained only in `contextRefs` and `excludedRefs`.

The traversal has the existing diagram, entity, hop, and result bounds and fails plan creation rather than silently truncating authorized targets. This relation allowlist replaces the current generic both-direction neighborhood closure for Play-generated plans.

### 5. Separate logical authority from necessary physical effects

Each work item identifies the diagram entity whose behavior is authorized. During implementation the build skill may change physical code, configuration, database definitions, migrations, dependency wiring, and tests needed to make that behavior work. Those paths are recorded as `physicalEffects` against the work item; they do not become new logical work items.

The build skill prompt and completion validator require every claimed product behavior to trace to an eligible plan target. Changes that merely support an authorized target are allowed. Replay, unmodeled alternate paths, separate stub services, duplicated services, and adjacent product features are prohibited. When working software would require such behavior, the build stops and returns to design rather than broadening the plan.

This cannot prove semantic scope from arbitrary source diffs. Enforcement combines graph-derived target grants, item-focused context, explicit exclusions, evidence linkage, bounded generated-contract validation, and tests of host guidance. A general code-policy engine is intentionally out of scope.

### 6. Use a fixed sequential controller over a dependency-aware queue

The runtime, not an LLM-generated script, owns plan and item transitions. New MCP operations provide bounded list/get/wait, claim-next, record-result, publish-contract, and complete/fail behavior. Every mutation supplies the expected progress revision and active claim token.

The manifest keeps one work item per eligible graph entity. Ordering uses containment/composition depth and relationship dependencies:

- deeper composed implementation items precede their composable parent integration item;
- endpoint node implementation precedes its edge/event contract item;
- explicit graph dependencies precede dependents;
- cyclic groups retain separate items but use stable URN ordering within a shared cycle phase so a cycle cannot deadlock the queue;
- final integration verification follows all entity work.

Only one item may be active in the first version. The build skill loops: claim, revalidate, load focused context, inspect current repository state including completed-item effects, implement, check, record evidence, and claim again. Codex or Claude may internally delegate analysis, but the plan protocol does not launch provider tasks and does not permit concurrent claims.

### 7. Return focused context with the rest of the envelope as support

`get_work_item_context` is bound to plan ID, work-item ID, claim token, and base graph revision. It returns:

- the focused eligible entity and its logical Markdown;
- other eligible targets in the envelope as read-only supporting context unless currently claimed;
- downward containment and composition context;
- incident edges, events, and direct endpoints;
- `open` and `locked` context with explicit exclusion markers;
- completed-item physical effects and check summaries;
- source citations and declared implementation-contract outputs.

The context grant authorizes implementation only for the focused item. This lets each item use the whole selected service/function for coherence without batching the whole plan into one unconstrained LLM edit.

### 8. Store generated implementation contracts beside logical documents

For an implemented edge or event, the build skill submits Markdown plus entity URN, plan ID, version, claim token, and expected existing generated-document revision. The runtime resolves the source map and derives `docs/<source-id>-contract.md`; callers cannot provide a path. The document starts with a reserved generated metadata envelope containing schema version, plan ID, scope key/version, entity URN, base graph revision, and body hash, followed by human-readable actual contract content.

The graph loader recognizes this suffix and envelope as `implementationContract` content linked to the existing owner. It indexes the body independently from logical Markdown, computes a drift flag from normalized body hashes, and never merges the two documents. A generated contract with an unreachable, non-edge, non-event, foreign-plan, out-of-scope, or stale owner is rejected or diagnosed.

Contract persistence uses the existing conditional-save and refresh coordinator. The plan records its immutable base revisions plus an allowlist of expected generated paths. Publications affecting only those paths are recognized as self-output and produce `completionGraphRevision`; any other authoritative change during the build makes the plan stale. This avoids both circular plan generation and false staleness at completion.

### 9. Migrate legacy Play state and managed skills conservatively

On first startup with `play.json` and no plan directory, the runtime attempts to resolve its diagram and selected entity against its recorded graph revision. A compatible ready/building/failed request becomes a `1.0.0` plan with an immutable manifest and equivalent lifecycle evidence. If exact scope cannot be reconstructed or the graph revision is obsolete, it becomes a visible stale legacy plan that cannot authorize writes. The original file is retained as migration evidence until the converted plan is durably written and validated.

Older clients continue to see a projection of the newest current plan through the existing build request operations during one compatibility release. New skills require the versioned-build-plan capability before using the new workflow.

Managed-file migration replaces only generated `graph` skill/command files whose content matches a known managed version. A locally modified or unrelated file is preserved with an actionable diagnostic rather than overwritten or deleted.

## Risks / Trade-offs

- **[Logical adjacency still over-selects behavior]** → Limit connected expansion to one layer, stay inside the selected service/function boundary, require `new`/`modify`, show the resolved plan summary in the UI, and never traverse ancestors.
- **[Logical adjacency under-selects physical work]** → Permit evidence-linked physical effects while keeping feature behavior bound to plan targets.
- **[Semantic version choice becomes friction]** → Ask only when the scope contract changed, use three compact buttons, display `Fix` to users, and resume unchanged plans automatically.
- **[Natural-language scope selects the wrong duplicate label]** → Search only reachable content, include composition breadcrumbs, require confidence separation, and show numbered candidates when ambiguous.
- **[Plan skill observes stale browser drafts]** → Check the browser session's dirty-document state and refuse plan creation until those edits are durably saved and published.
- **[Two hosts race on the same build]** → Use progress revisions and claim tokens; only one work item can be active.
- **[Generated contracts normalize accidental implementation drift]** → Keep logical and actual documents separate, expose drift, require plan linkage, and reject behavior outside authorized scope.
- **[Generated contract publication changes the graph revision]** → Allowlist declared output paths and record a distinct completion graph revision while retaining the immutable base grant.
- **[Legacy request migration guesses scope incorrectly]** → Convert only when identity and revision resolve exactly; otherwise expose a non-executable stale record.
- **[Plan history grows indefinitely]** → Preserve all open work and retain only the newest 50 terminal plans.

## Migration Plan

1. Add versioned plan/progress/evidence protocol types, validation, atomic storage, semantic-version ledger behavior, and legacy `play.json` compatibility projection.
2. Add selected-boundary graph resolution and deterministic work-item ordering, then create plans server-side after graph publication.
3. Add MCP reachable natural-language scope resolution plus plan creation, discovery, exact lookup, wait, claim, context, result, contract-publication, and completion tools behind a capability flag.
4. Replace generated Codex and Claude `graph` adapters with `plan`, and update build adapters to use the numbered open-plan menu and sequential controller loop.
5. Add generated implementation-contract persistence, graph representation, drift reporting, and self-output completion revision handling.
6. Update the browser Play/version interaction and plan status display, then update documentation and managed assets.
7. Retain old build-request projection for one compatibility release; rollback can disable new plan creation while leaving immutable plan directories inspectable and using the legacy reader.
