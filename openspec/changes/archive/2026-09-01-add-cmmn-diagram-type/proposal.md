## Why

Software Schematic currently edits and composes only BPMN diagrams, so project design can exist without a visual statement of the business need it serves. Adding CMMN as the business anchor above BPMN lets a project model the need and context first, then trace CMMN Process Tasks to BPMN designs such as `cybling.sdk.Birth`, without introducing a second application architecture or requiring need models at every package level.

## What Changes

- Add `.cmmn` as the supported project-anchor diagram type using bundled `cmmn-js` browser assets.
- Extend `ss init` so the generated `.ss/` browser bundle contains the HTML, TypeScript-derived JavaScript, styles, CMMN dependencies, starter content, and metadata needed to open CMMN alongside BPMN through the existing lightweight local server.
- Initialize and open `schematics/main.cmmn` as the sole project anchor. CMMN defines the domain, actors, inputs, outputs, needs, and business services; BPMN defines logical architecture and solution building blocks reached through named Process Tasks.
- Give CMMN diagrams the same editor behaviors already available to BPMN: retained tabs, automatic save, selection, ID/Type/Label/Name/Implementation Status inspection, diagram and element Markdown, and diagram- and node-scoped magic actions.
- Let a CMMN business-anchor document use a Name such as `cybling` or `cybling.sdk` and live as `main.cmmn` in that package folder. Package folders remain neutral grouping containers and may contain nested packages and BPMN process folders without requiring another CMMN file at each level.
- Make CMMN Process Task → BPMN process the primary need-to-design composition path. For example, a Process Task in `cybling/main.cmmn` may anchor BPMN process `cybling.sdk.Birth` at `cybling/sdk/Birth/main.bpmn`, even when `cybling/sdk/main.cmmn` does not exist.
- Provide an anchor-first workflow that creates or opens BPMN design from the CMMN need while preserving existing direct BPMN creation for lightweight authoring and backward compatibility.
- Reuse the existing ID-bound `docs/<element-id>.md` convention for every supported CMMN node and edge, and `main.md` for the CMMN diagram document.
- Keep the Rust web server limited to confined generic file, composition, rename, and save operations; CMMN modeling and interaction behavior remains in the bundled browser files.
- Do not add property-graph generation, embedding, compensation-flow behavior, workflow execution, deployment, multi-user, governance, or other enterprise case-management features.
- Preserve existing BPMN process composition behavior, including BPMN-driven folder creation. CMMN is the sole project anchor for new projects but does not become mandatory at every package level or the authority for filesystem structure.

## Capabilities

### New Capabilities

- `cmmn-diagram-editing`: Loading, editing, inspecting, documenting, saving, and invoking magic actions on CMMN files with behavior parallel to BPMN.
- `cmmn-package-composition`: Name-based CMMN business anchors in neutral package folders and need-to-design navigation from Process Tasks to BPMN compositions across the package tree.

### Modified Capabilities

- `software-schematic-bootstrap`: Include CMMN starter and browser assets in newly initialized self-contained projects.
- `software-schematic-composition`: Generalize composition navigation and documentation ownership so CMMN package diagrams can coexist with BPMN process diagrams in the existing folder hierarchy.
- `software-schematic-editor`: Support retained CMMN tabs and apply the existing editor, inspector, Markdown, save, and magic-action experience to both diagram types.
- `scoped-process-naming`: Add CMMN package Names and simple parent-scoped node and edge Names for both notations, while retaining fully qualified Names as explicit cross-package overrides.
- `ai-diagram-assistance`: Let the existing diagram- and node-scoped magic actions receive CMMN context and propose a deliberately bounded set of CMMN edits and composition links.

## Impact

- Browser bundle: add `cmmn-js`, its local styles/assets, an SSW CMMN moddle extension, a CMMN editor adapter, and shared diagram-type routing while reusing the current UI and file APIs.
- CLI initialization: package and extract the CMMN-capable static browser bundle and starter CMMN asset beneath `.ss/`.
- Lightweight Rust server: extend existing allow-listed file extensions and generic composition requests only where needed; no CMMN domain model or graph builder is added server-side.
- Project files: permit `.cmmn` and their existing `main.md`/`docs/<id>.md` companions beneath `schematics/`.
- Dependencies: add a pinned, locally bundled `cmmn-js` dependency compatible with the existing `diagram-js`-based browser toolchain.
- Compatibility: legacy BPMN-only initialized projects may open `schematics/main.bpmn` as a fallback until migrated; new projects do not create a competing root BPMN.
