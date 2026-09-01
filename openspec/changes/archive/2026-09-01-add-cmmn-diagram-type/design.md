## Context

Software Schematic is a project-local documentation tool. Its Rust CLI initializes a project, extracts a self-contained browser bundle beneath `.ss/`, and serves that bundle plus a small set of path-confined file operations. The browser currently uses `bpmn-js`, SSW moddle metadata, name-derived composition paths, retained tabs, an element inspector, Markdown companions, automatic persistence, and scoped assistant actions.

CMMN needs the same authoring experience without becoming a case-management runtime or introducing another backend. `schematics/main.cmmn` is the sole project anchor and defines the domain, actors, inputs, outputs, needs, and business services. BPMN describes the logical architecture and solution building blocks chosen to satisfy those needs. A Process Task in the root anchor may link to `cybling.sdk.Birth` at `schematics/cybling/sdk/Birth/main.bpmn`. Package folders remain simple organizational containers: the design does not require `cybling/sdk/main.cmmn` merely because an anchored BPMN design lives below that package.

## Goals / Non-Goals

**Goals:**

- Add CMMN as the browser-edited, sole project-anchor layer above BPMN by extending the existing web editor.
- Represent domain context, actors, inputs, outputs, needs, and business services in CMMN rather than using BPMN pools or lanes as an abstract-system catalog.
- Mirror the current BPMN tab, selection, metadata, documentation, save, composition, and magic-action experience where CMMN has an equivalent concept.
- Map a package-level CMMN business anchor's complete Name directly to its package folder and `main.cmmn` without making CMMN authoritative over folder creation.
- Make CMMN Process Task to BPMN composition the direct, visible need-to-design traceability mechanism.
- Allow one CMMN anchor to drive BPMN designs in descendant or otherwise named project packages without requiring intermediate CMMN documents.
- Keep all modeling logic in static browser assets generated from HTML, CSS, and TypeScript sources and extracted beneath `.ss/` by `ss init`.
- Keep the Rust server a small, notation-neutral, path-confined file service.
- Preserve existing BPMN process behavior and file formats while moving new-project startup to CMMN.

**Non-Goals:**

- Property-graph construction, graph persistence, vector embeddings, or retrieval changes.
- CMMN execution, planning engines, runtime case instances, compensation, deployment, or process orchestration.
- Enterprise repositories, governance, permissions, collaboration, tenancy, or integration frameworks.
- Automatically rewriting the content of legacy BPMN-only projects; they receive a startup fallback until explicitly migrated.
- Implementing every CMMN 1.1 semantic feature beyond what `cmmn-js` already imports, displays, edits, and serializes.
- Adding CMMN-specific application services to the Rust server.

## Decisions

### Add a diagram adapter, not a second application

The browser will route `.bpmn` to a BPMN adapter backed by the existing `bpmn-js` modeler and `.cmmn` to a CMMN adapter backed by `cmmn-js`. Each adapter supplies import, export, selection, modeling, registry, type checks, label/ID/SSW metadata updates, composable-element checks, and normalized assistant context. Shared code continues to own tabs, inspector controls, Markdown, save queues, breadcrumbs, dialogs, and assistant UI.

This is preferred over copying the current editor because duplicated tab, persistence, and documentation behavior would drift. A generic server-side diagram model was rejected because it would move browser modeling concerns into Rust and enlarge the change.

### Bundle CMMN locally through the existing web build

Pin `cmmn-js` at `0.19.2` and bundle its runtime, styles, icons/fonts, and an SSW CMMN moddle descriptor with the existing static web assets. That release carries its compatible `diagram-js` `4.0.3` runtime while the existing `bpmn-js` `18.22.1` editor retains `diagram-js` `15.24.1`; the CMMN modeler is therefore isolated in a lazy vendor chunk behind the diagram-adapter boundary. Browser source remains authored as TypeScript modules where introduced; the build emits browser-executable JavaScript because browsers do not execute TypeScript directly. `ss init` extracts the resulting HTML, CSS, JavaScript, and vendor assets beneath `.ss/web` exactly as it does today.

No CDN, package registry, Node runtime, or application server is required after initialization. The installed project's `.ss/` directory contains the complete browser application.

### Treat CMMN as a sparse business-anchor layer

A non-root CMMN diagram has one complete package Name composed of dot-separated lower-camel segments:

- `cybling` maps to `schematics/cybling/main.cmmn`
- `cybling.sdk` maps to `schematics/cybling/sdk/main.cmmn`

The Name is stored in the CMMN document through the SSW moddle extension and is displayed in the same Name inspector field used for BPMN. The folder is derived from the Name; no separate editable path or qualified-name field is introduced.

Within a CMMN diagram, ordinary selectable elements may store a short member Name such as `birthRequirements`; it resolves against the owning package as `cybling.sdk#birthRequirements`. A Process Task may store short process Name `Birth`; it resolves against the owning package as `cybling.sdk.Birth`. Fully qualified Names remain supported as explicit cross-package overrides. BPMN nodes and edges follow the same rule: a short member Name inherits the owning BPMN process, while a short reusable-process Name inherits its package. IDs remain occurrence identities and labels remain unrestricted presentation text.

The mapping locates a business-anchor document; it does not require every package folder to contain CMMN and does not prevent BPMN composition from creating folders as it does today. “Sparse” means CMMN is authored wherever a business need or capability needs a visual anchor, not mechanically repeated at every directory level. Using the CMMN label as the folder name was rejected because labels are presentation text. Inferring folders from graphical Stage nesting was rejected because this lightweight documentation tool should not impose a project architecture.

### Preserve need-to-design direction without hard enforcement

The intended composition direction is `CMMN business need → Process Task → BPMN design`. Opening a named CMMN Process Task creates or reuses the target BPMN composition and keeps the originating CMMN tab available as its business context. The link remains stored on the CMMN Process Task, so the need owns the design reference. One CMMN file may anchor many BPMN designs anywhere in the project namespace, including descendant packages that have no `main.cmmn`.

The tool does not block direct BPMN creation, require a CMMN document in every ancestor directory, or synthesize need definitions. Those rules would make a documentation tool overly opinionated and would break existing projects. The anchor-first path is made primary through navigation and creation affordances, while direct BPMN authoring remains available.

### Use Process Task as the BPMN composition link

- A `cmmn:ProcessTask` is the CMMN-to-BPMN composition link. Opening it first resolves a short Name against the CMMN package, then derives `<package-path>/<Process>/main.bpmn` through the existing qualified BPMN process Name mapping and creates or reuses the existing BPMN composition.

Other CMMN elements, including Case Tasks and Stages, remain ordinary documented diagram elements in this change. The custom SSW Name is the project-local Process Task link and therefore the durable authored trace from business need to design. Standard CMMN reference fields remain importable and serializable but do not become a second filesystem identity. This deliberately mirrors the current BPMN call-activity navigation instead of building general CMMN import or external-reference resolution.

### Reuse documentation ownership exactly

Every CMMN diagram uses sibling `main.md`. Every selectable CMMN shape or connection with an ID uses `docs/<element-id>.md` in the diagram's package folder. Renaming an element Name does not move its document; changing its ID uses the existing safe documentation rename operation. CMMN documents therefore require no new documentation API or storage format.

### Keep the server notation-neutral

The server will recognize `.cmmn` in its confined file allow-list and let the existing generic read/write operation persist it. Package-document creation and BPMN composition creation receive a validated kind and complete Name, derive paths server-side, and write the bundled `main.cmmn` or existing `main.bpmn` starter. Existing BPMN-driven folder creation remains unchanged. Rename support remains a confined filesystem operation driven by validated Names.

The server does not parse CMMN, build a CMMN object model, interpret sentries, resolve runtime references, or expose CMMN business endpoints. XML import, validation warnings, edits, and serialization remain responsibilities of `cmmn-js` in the browser.

### Extend magic actions through normalized adapter operations

The existing diagram- and node-scoped assistant entry points appear for CMMN. The CMMN adapter supplies normalized elements with ID, Type, Label, Name, Implementation Status, Documentation, and supported connections. The initial CMMN operation allow-list is intentionally small: update label/Name, replace Markdown, add a supported plan item, connect supported elements when `cmmn-js` permits it, and set/open a Process Task BPMN composition link.

Provider output still cannot choose paths, supply raw XML, execute code, or bypass the existing preview, approval, validation, and rollback flow. Supporting arbitrary CMMN XML or runtime expressions was rejected as unnecessary and unsafe for a documentation tool.

### Make CMMN the sole initialized project anchor

`ss init` writes and opens `schematics/main.cmmn` and does not create `schematics/main.bpmn`. The root CMMN owns project domain context and need-to-design navigation. What previously occupied the root BPMN becomes a named BPMN process referenced by a CMMN Process Task. BPMN pools and lanes therefore remain available for process design but are not needed to represent the abstract system landscape. The browser may fall back to `main.bpmn` only for a legacy project that has no `main.cmmn`; if both root files exist, startup rejects the competing anchors and asks for migration.

## Risks / Trade-offs

- [The older `cmmn-js` dependency may use different `diagram-js` internals than the current BPMN editor] → Isolate both libraries behind adapters, pin exact versions, split vendor bundles if necessary, and verify real import/edit/export fixtures for both notations.
- [“Feature parity” may invite unsupported CMMN semantics] → Define parity as shared SSW user experience and a tested subset of composable CMMN elements, not a case engine or complete semantic implementation.
- [Calling CMMN the anchor could imply one required CMMN per BPMN or package] → Define traceability through Process Task links and permit one anchor to cover many designs across descendant packages; do not enforce directory-by-directory need files.
- [Allowing direct BPMN creation can leave a design unanchored] → Keep it for simplicity and compatibility, but make Process Task navigation the primary creation affordance and preserve explicit CMMN-to-BPMN links for later validation or graph ingestion.
- [A Process Task may contain a standard reference that disagrees with its SSW Name] → Treat the SSW Name as the project navigation identity, preserve standard data during round-trip, and show a non-destructive warning rather than interpreting it as a path.
- [CMMN and BPMN files named `main.*` can coexist in one package folder] → Tabs and save queues use the complete canonical relative path, including extension, so the documents remain distinct.
- [Adding another modeler can increase the embedded binary and browser memory footprint] → Lazy-load the CMMN bundle only when a `.cmmn` tab opens and release its modeler resources when the tab closes.
- [Generic composition endpoints could accept invalid kind/name combinations] → Validate the complete Name against the selected CMMN-package or BPMN-process grammar before resolving a confined path.

## Migration Plan

1. Add pinned CMMN web dependencies, the moddle descriptor, fixtures, and a starter CMMN file.
2. Introduce the browser diagram adapter boundary while keeping BPMN behavior unchanged.
3. Add the CMMN adapter and route `.cmmn` tabs through it.
4. Extend generic confined file and composition operations to accept CMMN.
5. Add CMMN parity for metadata, Markdown, auto-save, navigation, and magic actions.
6. Rebuild the static bundle embedded by the CLI and verify a clean `ss init` project offline.

Existing projects require no file migration. Installing a new CLI version and re-running the project's supported asset-update flow replaces `.ss/` browser/runtime assets without modifying `schematics/`. Rollback restores the prior CLI-generated `.ss/` assets; authored `.cmmn` and Markdown files remain ordinary project files.

## Resolved Questions

- `cmmn-js` `0.19.2` is the pinned compatible release; its older `diagram-js` runtime remains isolated in the lazy CMMN bundle.
- The first UI uses a small **New CMMN anchor** action that accepts a package Name and creates missing neutral package folders through the existing confined composition service.
