# Project-local schematic MCP

Software Schematic compiles the diagrams reachable from `schematics/main.cmmn` and their Markdown Documentation into an in-memory Grafeo property graph. Diagram files are source artifacts; graph identity comes from the persisted project ID, semantic diagram owner Name, and each diagram element's unique XML ID.

`ss init` registers the server only for the project in `.codex/config.toml` and installs a managed block in `AGENTS.md`. Trust or activate the repository's Codex configuration, then start a fresh Codex task after registration changes. Do not add a global Software Schematic MCP entry. The generated wrapper always binds execution to its own repository:

```sh
./ssw mcp
```

Graph and MCP startup do not require ONNX Runtime. The editor/server loads it lazily only in its asynchronous Markdown embedding worker; `./ssw embeddings` uses the same generator for explicit backfill. On macOS, install the runtime with `brew install onnxruntime`. On Windows, install the `Microsoft.ML.OnnxRuntime` native package and place `onnxruntime.dll` beside `.ss\bin\ss.exe`. A missing runtime leaves authored Markdown and text retrieval available while vector readiness reports a diagnostic.

Before development, call `get_project_model` and verify that `projectName`, `projectId`, and the root diagram identify the active repository. Then pass the user's natural request to `resolve_development_scope`. A confident result supplies a root universal URN and an authorized scope containing only green `new` or orange `modify` entities. Ambiguous results require user selection; the service never treats `open` or `locked` entities as implementation authority.

The query-only tools are `get_project_model`, `get_entity`, `resolve_development_scope`, `get_neighbors`, and `search_model`. Graph construction always builds Grafeo text retrieval and imports validated precomputed vectors when available. A successful connected Markdown save queues owner-scoped embedding derivation and a path-aware graph refresh, then returns after durable body persistence. Rapid saves coalesce to the newest body hash; a compare-and-swap header write cannot overwrite a newer edit. Missing, stale, or invalid vectors degrade search to text retrieval without delaying MCP startup. Queries may briefly use the prior complete revision while refresh is queued or processing; a failed refresh keeps that revision and exposes a diagnostic in the editor. Codex cannot trigger refresh or mutate graph or source documents. Requests are bounded to 50 results, four graph hops, 512 reachable diagrams, 100,000 entities, and 2 MB per Markdown body by default. See [Markdown embedding headers](markdown-embedding-headers.md) for the generator contract.

The additive `guidedDesignWorkspace` delivery capability provides the zero-argument `open_design_workspace` entry tool. It reuses the project runtime started by the MCP adapter, reuses a fresh browser session when one exists, or asks the runtime to open its own authenticated loopback workspace when it is headless. The response contains only bounded project identity, revisions, root and process choices, active diagram/selection context, and resumable interview/proposal summaries. It never returns the daemon token, authenticated URL, absolute project path, or browser session identifier. If browser launch is unavailable, the response directs the developer to run `./ssw` or `ssw.cmd`.

`open_design_workspace` is orientation, not mutation authority. The design skill still saves bounded interview decisions, submits revision-bound typed proposals, and waits for browser approval. It guides the developer to an explicitly confirmed model, then offers Play or the natural-language `plan` skill. The operation cannot approve a proposal, replace XML, write arbitrary files, or start implementation.

The `versionedBuildPlans` tools create, list, retrieve, wait on, claim, and complete immutable scoped plans. Browser Play and conversational `create_build_plan` converge on the same runtime planner. Exact selection or natural-language resolution is confined to reachable entities; traversal goes down containment/composition and adds only the selected node's attached events, incident edges, and one direct connected layer. Only `new`/`modify` entities become work items. Compare-and-swap claims allow one active item per plan and provide claim-bound focused context, exclusions, citations, and completed evidence.

For plan-declared edge/event outputs, `publish_generated_contract` conditionally writes the derived `docs/<source-id>-contract.md` path with reserved plan/version/entity metadata. Callers cannot supply an arbitrary path. The logical Markdown and implemented contract are indexed separately and expose drift; expected self-output advances the plan's authorized graph revision, while unrelated graph changes stale the plan.

Keep proposals small. Detailed contracts belong in diagram structure and Documentation. A suitable proposal/task fragment is:

```yaml
intent: Improve checkout order submission
resolvedRoot: urn:ssw:project-123:node:acme.Build#Task_1
tasks:
  - text: Implement order validation
    nodeRefs:
      - urn:ssw:project-123:node:acme.Build#Task_1
sourceTrace: schematics/acme/Build/main.bpmn#Task_1
```

Source paths are citations, not graph IDs. Refresh an existing project with `./ssw update`; this preserves diagrams, Markdown, unrelated TOML, and content outside the managed `AGENTS.md` sentinels. To remove the guidance, delete the block from `<!-- software-schematic:begin -->` through `<!-- software-schematic:end -->` and remove only `[mcp_servers.software_schematic]` from `.codex/config.toml`.
