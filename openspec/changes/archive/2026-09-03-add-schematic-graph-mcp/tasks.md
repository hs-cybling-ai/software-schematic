## 1. Dependencies and Module Boundaries

- [x] 1.1 Pin Grafeo's minimal LPG, AI, native text/vector/hybrid search, and local `embed` features; pin the official `rmcp` server/stdio features plus XML and hashing dependencies in `software-schematic-cli/Cargo.toml`.
- [x] 1.2 Add `schematic_graph` and `schematic_mcp` modules with store, parser, embedder, snapshot, retrieval, and protocol boundaries that can be tested independently.
- [x] 1.3 Define typed diagram, element, relationship, document-chunk, diagnostic, limits, and snapshot-summary records with stable serialized MCP response forms.

## 2. Confined Composition Discovery and Parsing

- [x] 2.1 Extend the BPMN and CMMN moddle descriptors and editor adapters to persist normalized Implementation Status through command stack, undo/redo, dirty state, autosave, close, and reopen.
- [x] 2.2 Add editor tests for `new`, `modify`, `locked`, and default `open` XML round trips, colors/text, invalid normalization, and status removal.
- [x] 2.3 Reuse or extract project-root and `schematics/` path confinement helpers for graph reads, including normalized workspace-relative paths and symlink/traversal rejection.
- [x] 2.4 Implement CMMN parsing into the intermediate model with exact IDs, node/connection properties, normalized status, endpoints, and composition references.
- [x] 2.5 Implement BPMN parsing into the intermediate model with exact IDs, flow-node/edge properties, normalized status, endpoints, and composition references.
- [x] 2.6 Implement breadth-first discovery from `schematics/main.cmmn` using Name-to-path mapping, visited-path cycle handling, reachable-only inclusion, and orphan diagnostics.
- [x] 2.7 Add fixture tests for nested compositions, duplicate IDs disambiguated by semantic owner Name, source moves, statuses, cycles, missing targets, malformed XML, conflicting identities, and confinement.

## 3. Documentation, Chunking, and Embeddings

- [x] 3.1 Load confined diagram `main.md` and element `docs/<element-id>.md` files with optional-absence behavior, owner association, normalized content hashes, and source provenance.
- [x] 3.2 Implement deterministic heading-aware Markdown chunking with paragraph/token fallback, ordering, overlap, stable chunk keys, and configured document/chunk limits.
- [x] 3.3 Define the thin Grafeo embedding-profile/test seam and deterministic fixtures, then test native batching, model provenance, dimensions, non-finite vector rejection, and error propagation without implementing an independent embedder.
- [x] 3.4 Integrate and package the pinned Grafeo local ONNX embedding model, including native text/vector index creation, readiness validation, and no-network execution tests.

## 4. Grafeo Snapshot and Retrieval

- [x] 4.1 Define deterministic project/owner-Name-scoped SSW URNs for diagram, element, and chunk `id`; preserve exact `sourceId`; and define Grafeo labels, relationships, status/scope properties, uniqueness rules, and indexes.
- [x] 4.2 Implement the snapshot sidecar source map from URNs to confined diagram/Markdown files and XML IDs, keeping paths out of graph identity and semantic properties.
- [x] 4.3 Populate a new in-memory `GrafeoDB` only after intermediate-model validation and embedding completion, and compute the deterministic source/model revision fingerprint and load summary.
- [x] 4.4 Implement exact lookup by universal URN, `{ownerName, sourceId}`, canonical Name, and unambiguous source ID with explicit ambiguity/not-found results and source-map citations.
- [x] 4.5 Implement deterministic bounded incoming/outgoing neighborhood traversal with direction, relationship-type, hop, and result filters.
- [x] 4.6 Implement bounded Grafeo-native text/vector/hybrid retrieval with semantic-owner/neighborhood filters, graph-distance context, citations, and deterministic tie-breaking.
- [x] 4.7 Add graph/retrieval tests covering semantic URN stability, source moves, authored edges, containment/endpoints/composition, documentation, filters, ranking, and bounds.
- [x] 4.8 Implement proposal-language root ranking restricted to `new`/`modify`, confidence and separation thresholds, graph-coherent scope expansion, and ambiguous/no-match results.

## 5. MCP Service

- [x] 5.1 Add the public `ss mcp --project <root>` command and route project wrappers to it without opening the browser.
- [x] 5.2 Implement the `rmcp` stdio server lifecycle so the complete initial snapshot loads before initialization, stdout remains protocol-only, and failures produce structured stderr diagnostics and unsuccessful exit status.
- [x] 5.3 Implement typed `get_project_model`, `get_entity`, `resolve_development_scope`, `get_neighbors`, and `search_model` tools with explicit schemas and enforced limits.
- [x] 5.4 Implement `reload_model` with build-then-swap concurrency so successful reloads publish one complete revision and failed reloads retain the prior snapshot.
- [x] 5.5 Add MCP tests for initialization, tool discovery, confident natural-language root selection, ambiguous candidates, no eligible match, explicit-root validation, limits, revisions, and reload rollback.
- [x] 5.6 Add tests launching the pinned wrapper from another working directory and verifying project name, project ID, root diagram identity, revision, invalid-root failure, and no ambient-directory inference.

## 6. Agent Guidance and Existing-Project Update

- [x] 6.1 Define managed guidance requiring diagram-owned contracts, natural-language MCP scope resolution, concise OpenSpec-style proposals, a recorded resolved root, `nodeRefs` on every task, and implementation limited to `new`/`modify` entities.
- [x] 6.2 Implement line-ending-preserving, idempotent create/append/replace logic for the managed root `AGENTS.md` block without modifying content outside its sentinels.
- [x] 6.3 Implement structural, idempotent merge of the managed `software_schematic` MCP entry into project `.codex/config.toml` while preserving unrelated valid TOML settings and servers.
- [x] 6.4 Install managed Codex registration, guidance, and embedding assets during `ss init`, preserving first-time collision guarantees.
- [x] 6.5 Add an idempotent update path that refreshes runtime assets, project MCP registration, wrapper routing, and guidance without rewriting diagrams or Markdown.
- [x] 6.6 Add initialization/update tests for existing `AGENTS.md` and `.codex/config.toml`, repeated updates, stale managed content, CRLF, collisions, and preservation of authored files/configuration.

## 7. Packaging, Documentation, and Verification

- [x] 7.1 Add a concise proposal/task example showing natural language resolved to a root-node URN, per-task URN `nodeRefs`, source traceability, and no duplicated detailed contract outside diagram Documentation.
- [x] 7.2 Document project Codex trust/activation, MCP registration, pinned wrapper invocation, project-identity verification, scope resolution, tools, reload behavior, limits, and managed-block removal.
- [x] 7.3 Verify release packaging includes MCP/runtime and pinned embedding assets on macOS and Windows and performs semantic root matching without network access.
- [x] 7.4 Run formatting, linting, unit/integration and regression tests, OpenSpec validation, and an MCP smoke test resolving and querying a green/orange development scope.
