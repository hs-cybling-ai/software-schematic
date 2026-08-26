## Context

SSW currently uses BPMN `id` for instance identity, BPMN `name` for the visible label, and `calledElement` or folder paths as a second user-managed identity for reusable compositions. The assistant demonstrated that these values drift: it preserved a useful activity label but invented a shortened folder name that conflicted with a child concept. In MDD, the diagram defines architectural symbols, so identity must be explicit, scoped, deterministic, and independently editable from presentation text.

The root `schematics/main.bpmn` remains the non-removable project navigation diagram. Named child processes own portable composition folders containing `main.bpmn`, `main.md`, `docs/`, and local assets. Implementation bindings such as Java classes and methods remain Markdown concerns.

## Goals / Non-Goals

**Goals:**

- Give pools, processes, tasks, and events a distinct architectural Name alongside BPMN ID and Label.
- Use familiar Java-style symbols: lowerCamelCase package segments, UpperCamelCase process names, and lowerCamelCase members.
- Resolve reusable processes by qualified Name and ordinary members by their owning process.
- Derive one canonical folder from each process Name and remove path choice from normal UI and AI operations.
- Make process rename a safe project-wide refactor that moves the entire composition unit and all references.
- Preserve internal documentation links when a composition moves.
- Migrate legacy `calledElement` and folder data without losing content.

**Non-Goals:**

- Mapping process symbols to implementation classes, methods, endpoints, or source files.
- Making labels valid identifiers or deriving identity continuously from labels after initial creation.
- Giving ordinary tasks or events their own composition folders.
- Renaming or removing the root navigation diagram.
- Persisting the model in GrafeoDB in this change.

## Decisions

### Separate instance identity, architectural identity, and display text

BPMN `id` remains the stable identity of a diagram instance. BPMN `name` remains the human-readable Label. SSW stores a distinct extension-backed local Name. Labels can change freely; Name changes invoke symbol-aware validation or refactoring. This avoids overloading BPMN `name` while presenting the simple UI vocabulary ID, Name, and Label.

### Use pool packages and process/member qualified symbols

A pool Name is a dotted sequence of lowerCamelCase package segments. A reusable process Name is UpperCamelCase and resolves under the containing pool package or inherited owning-process package as `package.Process`. A non-process task or event Name is lowerCamelCase and resolves as `package.Process#member`. Child diagrams retain their process qualified Name as diagram metadata, so ordinary members do not require a pool inside every child diagram.

Moving a named node across scopes does not silently change its target. SSW shows the newly derived symbol and requires an explicit keep-or-rebind choice when the resolved identity would change.

### Make qualified process Name the sole user-facing composition reference

The External process field is removed. Reusable process nodes expose Name and a read-only Qualified Name. Nodes with the same qualified process Name resolve to the same composition. SSW may mirror the qualified symbol into standard BPMN `calledElement` for interoperability, but users and providers never manage a separate path value.

This is preferable to retaining independent Name and path fields because two editable identities inevitably drift.

### Map symbols deterministically to portable folders

`cybling.subscription.SelectAndOutfit` maps to `schematics/cybling/subscription/SelectAndOutfit/`. The mapping preserves Java-style case, rejects case-insensitive collisions, and always places `main.bpmn`, `main.md`, `docs/`, and assets beneath the process folder. Ordinary members remain in that folder and have no derived composition path.

New compositions use the blank BPMN starter. Creation APIs accept qualified process Names rather than arbitrary folders and return the derived paths.

### Resolve Markdown links from the owning diagram folder

Relative links in diagram and node Markdown resolve from the composition folder containing `main.bpmn`, even when the node Markdown physically lives under `docs/`. Moving the complete folder therefore preserves links such as `./assets/model.png` and `./docs/Activity_A.md` without rewriting content. Escapes outside `schematics/` remain prohibited. Cross-process architectural references use qualified Names rather than filesystem-relative links.

### Implement process rename as a server-coordinated refactor

Renaming a diagram process preflights the new symbol, derived folder, case-insensitive collisions, affected references, open files, and confinement. It then stages the entire folder move and rewrites every stored qualified reference. The browser rekeys retained tabs, breadcrumbs, documentation targets, save queues, and assistant revisions only after the server succeeds. Any failure restores the old folder and references.

Label edits never invoke this refactor. Member Name edits update only that member and its derived display identity; process Name or pool package changes use the project-wide refactor because they affect paths and references.

### Treat AI naming as an explicit reviewed mutation

Assistant context includes ID, local Name, qualified Name, Label, kind, and owning scope. Providers cannot emit composition paths. Creating a process requires a valid Name; SSW derives its qualified Name and folder. Existing Names and Labels are preserved unless separate operations explicitly change them, and process rename appears in preview as a symbol change, folder move, and reference update.

## Risks / Trade-offs

- [Legacy folder names may not reveal the intended package or process] → Infer only unambiguous values, show migration diagnostics, and require user selection rather than moving ambiguous content automatically.
- [Case-only renames fail on common macOS and Windows filesystems] → Use a confined temporary sibling during the atomic move and test case-insensitive collision behavior.
- [Pool/package rename can affect many processes] → Preflight and preview the complete reference and folder impact as one refactor.
- [BPMN tools do not understand SSW Name extensions] → Retain BPMN IDs and labels and mirror reusable process symbols into `calledElement` where useful for interoperability.
- [Relative links from node Markdown normally resolve from `docs/`] → Set the renderer's explicit base to the owning composition folder and document this SSW convention.
- [Moving a node can unexpectedly change scope] → Never silently rebind; require an explicit keep or rebind decision.

## Migration Plan

1. Add extension metadata readers/writers and symbol/path utilities without changing existing resolution.
2. Inventory legacy compositions and build a migration preview from pools, labels, `calledElement`, and folders.
3. Persist unambiguous Names, preserve legacy references as compatibility aliases during the transition, and leave ambiguous items untouched with diagnostics.
4. Switch creation/navigation and assistant operations to qualified Names and deterministic paths.
5. Enable transactional process and package rename after reference indexing and rollback tests pass.
6. Remove the External process control once migrated projects resolve through Name.

Rollback retains migrated extension metadata but can restore compatibility resolution from `calledElement`; folder moves use captured before-state and can be reversed.

## Open Questions

- Should the root navigation diagram have a reserved qualified Name or remain outside the process symbol table?
- Should pool package rename be enabled in the first implementation or initially limited to process rename?
- What migration UI is best when one legacy folder is referenced by nodes in multiple candidate packages?
