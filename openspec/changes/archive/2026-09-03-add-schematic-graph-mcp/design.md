## Context

Software Schematic stores its model as project-local CMMN and BPMN XML under `schematics/`, with sibling `main.md` and ID-bound `docs/<element-id>.md` files. `schematics/main.cmmn` is the new-project root and authored composition Names resolve to other diagrams by established folder conventions. The Rust CLI already owns project discovery, confinement, initialization, and local services, but it does not have a complete project reader or an MCP server.

This change is deliberately a local documentation/retrieval feature. The graph is a derived snapshot, not a new store. Correctness means deterministic discovery, useful provenance, bounded retrieval, and clear failure; it does not require durable indexing, background reconciliation, multi-user consistency, or automated graph-to-file mutation.

Grafeo provides an embeddable Rust `GrafeoDB`, in-memory labeled property graphs, vector/text search features, and an optional ONNX embedding feature. The official Rust MCP SDK (`rmcp`) provides typed tools and stdio transport. Both fit the existing single-process Tokio CLI.

## Goals / Non-Goals

**Goals:**

- Build one complete, deterministic in-memory graph by walking reachable compositions from the root CMMN.
- Preserve enough source identity and provenance for an LLM to cite the diagram or Markdown file behind every result.
- Carry diagram-authored Implementation Status into Grafeo and make `new`/`modify` the only authorized development scope.
- Resolve natural proposal language to a root diagram node and link every proposal task to in-scope diagram nodes.
- Represent diagrams, diagram nodes, and authored diagram edges as graph entities while also materializing containment, topology, documentation, and composition relationships.
- Attach Markdown and embeddings to bounded chunks and expose exact, graph, lexical, and semantic retrieval through MCP.
- Keep the service simple to launch from coding agents and safe to use in a project workspace.

**Non-Goals:**

- Persisting or incrementally synchronizing Grafeo state.
- Watching files, coordinating concurrent editors, compensating partial graph builds, or serving multiple tenants.
- Mutating schematic or source files through MCP, accepting arbitrary GQL, or asking an LLM to calculate file paths.
- Duplicating detailed requirements and design contracts in large prose proposals.
- Crawling every diagram merely because it exists; orphan files outside the root-reachable composition are diagnostics, not model content.
- Providing authentication or remote HTTP transport in the first version.

## Decisions

### Load a replacement snapshot transactionally in process memory

`load_schematic_graph(project_root)` resolves `schematics/main.cmmn`, parses the full reachable composition set into an intermediate typed model, validates it, generates document chunks, and then uses Grafeo to embed, index, and populate a new in-memory `GrafeoDB`. The caller swaps the completed snapshot into an `Arc<RwLock<Arc<GraphSnapshot>>>`; a failed startup has no server, and a failed explicit reload leaves the prior snapshot available.

Building an intermediate model before Grafeo makes validation and tests independent of query syntax and prevents clients from observing a partial graph. Persisting the graph was rejected because every record is derived cheaply from project files and persistence introduces staleness and migration behavior that this tool does not need.

### Traverse authored composition references from the root

The loader uses a queue seeded with `main.cmmn`, a visited set keyed by normalized workspace-relative diagram path, and the existing Name-to-path rules. It parses CMMN and BPMN through format-specific adapters into common `DiagramRecord`, `ElementRecord`, and `RelationRecord` values. A supported composable node with a valid Name adds a `COMPOSES_TO` relation and queues the resolved target only when that target exists. Cycles are valid and terminate through the visited set. Missing explicitly referenced targets, malformed XML, duplicate entity keys, path escape, and conflicting diagram identities are fatal with source context.

Only reachable compositions are loaded. This matches model-driven navigation and avoids silently granting unused or stale files equal authority. A summary diagnostic reports skipped/orphan diagram counts when discoverable without making them fatal.

### Use stable project-scoped URNs

Every graph entity has one universal `id` expressed as a stable SSW URN. Source-backed entities separately retain `sourceId`, which equals the exact persisted unique element ID in CMMN or BPMN. The compiler resolves each diagram's semantic owner Name using the existing CMMN package and BPMN process naming rules, then combines project identity, semantic owner Name, and the element's diagram ID. The terminal component is always the diagram element ID, never the element's optional Name or Label. File and folder names do not participate in graph identity:

- diagram: `urn:ssw:<projectId>:diagram:<qualified-owner-name>`; the root uses the reserved semantic owner `root`
- element: `urn:ssw:<projectId>:<qualified-owner-name>#<sourceId>` with labels `DiagramNode` or `DiagramEdge`
- document chunk: `urn:ssw:<projectId>:chunk:<owner-urn>#<content-hash>:<ordinal>`; it remains a retrieval record rather than a schematic entity

Diagram entities carry URN `id`, semantic owner Name, kind, Label, and diagram Markdown. Node and edge entities carry URN `id`, exact `sourceId`, type, Label, resolved Name, normalized Implementation Status, semantic owner Name, and element Markdown. Authored diagram edges are entities so their metadata and documentation remain queryable; separate `SOURCE` and `TARGET` relationships connect them to endpoint nodes. `CONTAINS`, `DOCUMENTED_BY`, and `COMPOSES_TO` relationships encode derived structure. Missing optional Markdown produces no chunk and is not an error.

Chunk identities are deterministic for unchanged normalized content. Each chunk records owner URN, heading path, ordinal, normalized Markdown, content hash, embedding model, dimensions, and vector. A separate `SourceMap` retained beside the compiled snapshot maps entity/chunk URNs to confined source files and XML IDs for diagnostics, citations, and future write-back; paths are not domain properties or query identity. This keeps the graph semantic while preserving exact source traceability.

### Persist diagram-authored Implementation Status

The existing four-value status becomes an SSW extension property on eligible CMMN and BPMN elements. `new`, `modify`, and `locked` are persisted explicitly; absent or unrecognized values normalize to `open`. Status edits use the modeler command stack and participate in dirty state, undo/redo, and automatic XML save. The loader copies normalized `implementationStatus` into Grafeo and derives `developmentScopeEligible=true` only for `new` and `modify`.

Color is redundant presentation, not input: green is `new`, orange is `modify`, gray is `locked`, and white is `open`. Agents may read `open` and `locked` nodes as context but must not treat them as implementation targets.

### Resolve proposal language to diagram-defined scope

Users describe a desired change naturally; they do not need to know or type a root ID in the prompt. The agent sends that proposal language to `resolve_development_scope`. Grafeo hybrid search is restricted to `new` and `modify` schematic entities, then graph structure ranks coherent root candidates and their reachable green/orange scope. Exact Name/Label/ID matches boost ranking but are not required.

When one candidate is sufficiently distinct, MCP returns it as the selected root and the agent records its universal URN `id` in the lightweight proposal. When candidates are close or absent, MCP returns bounded candidates and the agent asks the user to choose or mark the intended diagram node. Every proposal task records `nodeRefs` containing one or more authorized entity URNs. MCP can attach a source citation from the sidecar source map when humans need to locate the authored diagram, but proposal identity remains semantic. Detailed requirements and design stay in diagram structure and Markdown rather than being copied into OpenSpec.

### Use Grafeo's native embedding and hybrid retrieval pipeline

Markdown is split first on headings and then on paragraph/token boundaries to a configured maximum size with a small overlap. Empty content is ignored. Chunk text and provenance are inserted through Grafeo's opt-in native embedding capability using a pinned local ONNX model packaged or installed with the versioned runtime. Grafeo owns embedding generation, vector storage/indexing, text indexing, and hybrid candidate retrieval; SSW does not implement a parallel vector store, cosine engine, or ranking pipeline. A thin `GrafeoEmbeddingProfile` configuration/test seam supplies the pinned model and permits deterministic fixtures. Startup validates model availability, generated vectors, dimensions, and index readiness before serving.

Local Grafeo embeddings avoid credentials and keep project content off the network. The first release does not expose a separate embedding provider or remote-provider configuration. SSW applies only bounded filters, graph-distance context, result shaping, and deterministic final tie-breaking around Grafeo's native text/vector/hybrid results. Exact identity lookup and topology traversal remain graph operations independent of semantic ranking.

### Expose a narrow stdio MCP server

`ss mcp --project <root>` loads a snapshot and serves MCP over stdin/stdout using `rmcp`. Stderr is reserved for diagnostics so protocol output is never contaminated. The initial tools are:

- `get_project_model`: root identity, counts, reachable diagrams, load timestamp, and diagnostics.
- `get_entity`: one diagram/node/edge by universal URN, `{ownerName, sourceId}`, canonical Name, or unambiguous source ID, including properties, Markdown excerpts, and optional source-map provenance.
- `resolve_development_scope`: match natural proposal language against `new`/`modify` nodes, select a confident root or return ranked candidates, and return the authorized green/orange scope plus context-only neighbors.
- `get_neighbors`: bounded incoming/outgoing relationships around an entity, filtered by relationship type and hop limit.
- `search_model`: bounded Grafeo-native hybrid text/vector search with optional entity type, diagram, and neighborhood filters; results include Grafeo relevance plus SSW graph-distance context and citations.
- `reload_model`: rebuild and atomically replace the snapshot after file changes, or return diagnostics while retaining the old snapshot.

Inputs and outputs use explicit JSON schemas, conservative default/max limits, deterministic score tie-breaking, and no raw query language. MCP resources and write tools were considered but add little beyond these task-oriented operations in the first release.

### Bind Codex configuration and MCP execution to the project

Initialization creates or updates a managed `software_schematic` MCP entry in the repository's `.codex/config.toml`. The entry launches the repository's pinned `ssw mcp` wrapper rather than a global `ss` binary. The wrapper derives its project root from its own location and passes that exact root to `.ss/bin/ss mcp --project`, so MCP remains bound to the repository even when Codex has a different process working directory.

The MCP server canonicalizes the supplied root, verifies `.ss/`, `schematics/`, and `schematics/main.cmmn`, and includes `projectName`, a non-secret deterministic `projectId`, root diagram identity, and snapshot revision in `get_project_model` and scope responses. Managed `AGENTS.md` guidance requires the agent to call `get_project_model` first and refuse to use results from a server whose project identity does not match the active repository. Project Codex configuration is effective only when that repository is trusted; documentation makes that activation step explicit.

Existing `.codex/config.toml` settings are preserved. Initialization/update owns only the named Software Schematic MCP table and replaces that table idempotently. A global MCP registration was rejected because it can accidentally load the wrong project and bypass the pinned project runtime.

### Install a managed agent-guidance block safely

Initialization creates or updates the root `AGENTS.md` with sentinel-delimited Software Schematic guidance. The block tells agents to treat diagrams and Markdown as the detailed contract; use OpenSpec or a lightweight equivalent only for intent, resolved root identity, and node-linked tasks; pass the user's natural proposal language to MCP; implement only returned `new`/`modify` scope; use exact returned paths and IDs; and reload after approved diagram edits. If MCP cannot confidently resolve scope, the agent asks the user rather than guessing.

If `AGENTS.md` exists, only the prior sentinel-delimited block is replaced; unrelated content and line endings are preserved. If no block exists, one is appended with spacing. This idempotent behavior is preferable to overwriting user instructions. Existing initialized projects receive the same update through an explicit project upgrade/update path rather than silently changing files whenever the MCP server starts.

## Risks / Trade-offs

- [A local embedding model increases binary/runtime size and startup time] → Pin one compact model, package it with the versioned runtime, batch chunk embedding, publish load counts/timing, and cap document/chunk sizes.
- [Grafeo or its embedding API is comparatively young] → Isolate Grafeo integration behind one graph-store module, pin compatible crate features and model identity, and contract-test the MCP layer against the intermediate model.
- [Strict failure on one bad reachable composition can reduce availability] → Return diagram path, element ID, and cause; keep `reload_model` transactional so a previously good snapshot survives reload failure.
- [A startup snapshot becomes stale after edits] → Include a snapshot revision/fingerprint in every response and provide explicit reload; avoid a watcher until real use demonstrates the need.
- [Hybrid ranking can hide why a result won] → Preserve the relevance components Grafeo exposes, add graph-distance context separately, and return deterministic ordering and citations.
- [Agent instructions can conflict with project policy] → Use a clearly scoped managed block, preserve surrounding instructions, and describe MCP as required context gathering rather than authority to mutate files.
- [Semantic root selection can choose the wrong green/orange node] → Return scores and evidence, require a confidence/margin threshold for automatic selection, and ask the user when ambiguous.
- [Necessary work is not marked `new` or `modify`] → Return no eligible scope and instruct the user to update the diagram rather than silently widening implementation.
- [Source IDs repeat across diagrams] → Namespace them by semantic owner Name in project-scoped URNs and retain source location only in the sidecar source map.
- [A diagram Name changes] → Treat it as a semantic rename that intentionally changes compiled URNs; report old/new identities during reload and require proposals to refresh against the current snapshot revision.
- [Codex launches the wrong MCP or from the wrong directory] → Register the pinned wrapper in project `.codex/config.toml`, derive root from the wrapper location, and verify project identity before scope calls.
- [Composition cycles or very large projects can consume resources] → Deduplicate visited paths and entity keys and enforce configurable but conservative diagram, entity, Markdown, hop, and result limits.

## Migration Plan

1. Persist Implementation Status in CMMN/BPMN XML and verify editor save/reopen behavior.
2. Add the intermediate model, parsers, confinement checks, exact-ID graph schema, fixtures, and loader tests.
3. Add Markdown chunking, pinned Grafeo native embeddings/indexes, hybrid retrieval, and proposal-language root ranking.
4. Add the `ss mcp` stdio command, scope resolver, and typed MCP tools with protocol integration tests.
5. Add idempotent project `.codex/config.toml` registration, `AGENTS.md` guidance, and an explicit update path for existing projects.
6. Add lightweight proposal examples, Codex trust/activation guidance, and packaged Windows/macOS verification.

Rollback removes the MCP configuration and reverts the CLI runtime. No graph data requires migration or recovery because the graph is in memory. The managed `AGENTS.md` block can be removed by its sentinels without touching user content.

## Open Questions

- Which compact Grafeo-supported embedding model gives the best size/quality trade-off for the first packaged runtime?
- Should the existing project update entry point be a new `ss update` command or an idempotent extension of the current installation workflow?
- What default chunk size and maximum reachable entity count fit representative Software Schematic projects?
