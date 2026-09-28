## 1. Versioned Plan Protocol and Persistence

- [x] 1.1 Define versioned plan manifest, progress, evidence, work-item, claim, semantic-bump, generated-contract, and completion DTOs with strict validation and JSON-schema round-trip tests covering incompatible versions, bounds, forbidden executable content, and foreign projects.
- [x] 1.2 Replace singleton build persistence with atomic `.ss/workflows/builds/<plan-id>/{manifest,progress,evidence}.json` storage plus `current-build.json`; verify immutable manifests, compare-and-swap progress, claim recovery, non-terminal retention, and newest-50 terminal pruning in runtime tests.
- [x] 1.3 Implement scope-relative SemVer allocation and scope-contract fingerprints; verify first `1.0.0`, major/minor/fix advancement, duplicate versions across different scopes, stale latest-version rejection, and unchanged incomplete/failed-plan reuse.
- [x] 1.4 Add conservative legacy `play.json` migration and old-client projection; verify exact current records become executable `1.0.0` plans, obsolete or ambiguous records become non-executable stale plans, and authored schematic files remain unchanged.

## 2. Selected-Boundary Scope and Work Queue

- [x] 2.1 Implement exact browser-selection resolution from diagram path and source ID to a stable graph URN; verify duplicate XML IDs resolve only with their semantic owner and stale selections fail closed.
- [x] 2.2 Implement participant/pool scope traversal using outgoing containment and downward composition; verify a selected pool includes its eligible service descendants and compositions but no parent or sibling service.
- [x] 2.3 Implement node scope traversal with embedded descendants, attached events, incident edges, one direct connected layer, and eligible downward compositions; verify there is no incoming containment/composition expansion and no recursive adjacency walk across an entire process.
- [x] 2.4 Apply implementation-status authorization to every node, edge, event, and connected work target; verify only `new`/`modify` become work items while `open`/`locked` remain explicitly marked context or exclusions.
- [x] 2.5 Generate deterministic dependency-aware work items and phases; verify composed children precede parent integration, endpoints precede edge/event contract work, cycles use stable separate-item ordering without deadlock, and integration verification is last.
- [x] 2.6 Add scope-limit and physical-effect validation; verify plan creation fails rather than truncating authorized targets and completion accepts evidence-linked shared wiring/configuration/schema/migration/test effects while rejecting unrelated feature claims.

## 3. Unified Play and Natural-Language Plan Creation

- [x] 3.1 Implement one runtime `create_build_plan` operation for exact scope URN, expected graph/source revisions, and optional semantic bump; verify browser and skill callers produce equivalent immutable manifests for the same input.
- [x] 3.2 Add reachable-model natural-language scope resolution across types, labels, Names, Markdown, and composition breadcrumbs; verify confident unique matches, bounded deterministic duplicate-label candidates, and orphan exclusion.
- [x] 3.3 Make conversational plan creation verify durable source-manifest publication and browser dirty-document state; verify it synchronizes saved changes when safe, waits for publication, and refuses to plan from unsaved browser drafts.
- [x] 3.4 Add plan creation/resume behavior for unchanged and changed scopes; verify changed contracts require explicit major/minor/fix input while unchanged incomplete or failed plans resume without a new version.

## 4. MCP Build-Plan Workflow

- [x] 4.1 Add capability discovery and bounded tools to create, list, exactly retrieve, and wait on build plans; verify deterministic open-plan ordering, scope-plus-version lookup, opaque-ID lookup, and ambiguous bare-version rejection.
- [x] 4.2 Add compare-and-swap claim-next, record-result, fail, and complete operations; verify one active item per plan, competing-client conflict behavior, restart recovery, dependency readiness, and legal lifecycle transitions.
- [x] 4.3 Add claim-bound work-item context containing focused authority, selected-envelope support, downward/connected relationships, explicit exclusions, citations, and completed-item evidence; verify no ancestor becomes an implementation target and stale graph grants fail closed.
- [x] 4.4 Update MCP tool allowlisting and server instructions; verify no raw path, arbitrary file, graph mutation, executable payload, or proposal-approval authority is introduced by the plan tools.

## 5. Portable `design`, `plan`, and `build` Skills

- [x] 5.1 Replace generated Codex and Claude `graph` adapters with a `plan` skill that verifies project identity/publication, resolves natural-language reachable scope, presents short numbered candidates, obtains a version bump when required, calls shared plan creation, and reports the resulting scope/version; verify cross-host conformance fixtures match.
- [x] 5.2 Update the `design` skill's completion guidance to offer either clicking Play or invoking `plan` with a natural-language target, while preserving the rule that design does not start implementation; verify generated skill-content tests cover both paths.
- [x] 5.3 Rewrite the `build` skill to list numbered open builds when invoked without a selector, accept an invocation-local number or exact scope/version, and loop claim → context → implement → check → evidence one item at a time; verify Codex and Claude adapters expose equivalent behavior and never generate orchestration scripts.
- [x] 5.4 Update init, update, and doctor/repair managed-file handling to install `design`, `plan`, and `build`, remove only recognized managed legacy `graph` files, and preserve locally modified files with diagnostics; verify clean install, upgrade, repair, and user-modification fixtures.

## 6. Browser Play, Version, and Status Experience

- [x] 6.1 Make Play flush and await every pending relevant open diagram/document save before plan creation; verify a queued inactive-tab save or current Markdown edit cannot be omitted from the plan's source-manifest revision.
- [x] 6.2 Add compact Major, Minor, and Fix version controls shown only when a changed selected scope has prior plans; verify new scopes create `1.0.0`, unchanged retries resume, stale version bases are rejected, and keyboard/accessibility behavior is covered by browser tests.
- [x] 6.3 Replace the browser's direct legacy build request with shared plan creation and display the resulting concise scope label, semantic version, and ready/building/complete/failed/stale state; verify polling follows the correct plan while selection changes do not relabel it.
- [x] 6.4 Preserve a bounded compatibility display for a migrated legacy build and verify the UI never exposes scope URNs, filesystem paths, daemon credentials, or generated plan internals as its primary label.

## 7. Generated Edge and Event Implementation Contracts

- [x] 7.1 Define and validate the reserved generated-contract metadata envelope containing schema version, plan/scope identity, semantic version, entity URN, base graph revision, and body hash; verify malformed, foreign, stale, oversized, and non-edge/event submissions fail closed.
- [x] 7.2 Add plan- and claim-bound contract publication that derives `docs/<source-id>-contract.md` from the source map and performs conditional persistence without caller-supplied paths; verify edge/event authorization, expected-revision conflicts, scope-expanding content rejection, and successful writes.
- [x] 7.3 Extend graph loading and indexing to represent logical and implementation-contract Markdown separately on one owner, with distinct chunks, provenance, plan/version metadata, and normalized drift state; verify missing, matching, differing, orphan, and unsupported-owner cases.
- [x] 7.4 Route generated contracts through embedding and graph refresh, recognize only plan-declared output paths as expected self-output, and record a completion graph revision; verify own-output publication does not stale the plan while any unrelated authoritative change does.
- [x] 7.5 Add editor inspection for selected edge/event logical versus implemented contracts and a persistent drift warning; verify justified in-scope drift permits completion, neither document overwrites the other, and out-of-scope drift cannot be published.

## 8. Integration, Documentation, and Release Verification

- [x] 8.1 Add end-to-end fixtures for browser Play and conversational `plan` creating equivalent pool, node, composed-subprocess, edge/event, duplicate-label, unchanged-retry, and semver-advance plans, and verify all target/exclusion sets and versions.
- [x] 8.2 Add interrupted and cross-host build fixtures proving numbered discovery, exact lookup, sequential resume, physical-effect evidence, contract publication, integration checks, completion revision, and stale-model stopping behavior.
- [x] 8.3 Update README and workflow/MCP/daemon documentation to describe `design → plan → build`, Play equivalence, short open-build selection, semantic versions, selected-boundary rules, physical effects, and logical-versus-implemented contracts; verify documented tool and skill names match generated assets.
- [x] 8.4 Run web unit/browser tests, Rust formatting/lint/tests, managed clean-project init/update smoke tests, production bundle regeneration checks, and `openspec validate add-versioned-scoped-build-plans --strict`; record concise evidence and resolve every failure before marking the change complete.
