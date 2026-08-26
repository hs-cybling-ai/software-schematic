## Why

SSW currently conflates BPMN display labels, reusable process identity, and composition folder paths, allowing AI or manual edits to create shortened or conflicting composition names that no longer describe the architecture. Model-driven designs need stable, scoped symbols that remain easy to rename while their diagrams, documentation, and references move together.

## What Changes

- Add a distinct architectural `Name` to pools and eligible nodes while retaining the BPMN ID and human-readable Label.
- Use lowerCamelCase package names from pools, UpperCamelCase process names, and lowerCamelCase task/event member names.
- Resolve process symbols as `package.Process` and ordinary task/event symbols as `package.Process#member`.
- Replace the user-facing External process field and freely chosen composition paths with deterministic Name-based composition resolution.
- Let multiple nodes reference the same reusable process by using the same qualified Name, while allowing identical local names in different pool packages.
- Rename a process from diagram metadata as an atomic refactor that moves the complete composition folder and updates all references, tabs, breadcrumbs, and cached paths.
- Keep diagram and node Markdown links relative to the owning diagram folder so documentation and assets move without internal link rewrites.
- Require AI proposals to preserve IDs, Names, and Labels unless a change is explicitly previewed, and derive rather than invent composition paths.
- **BREAKING**: Existing user-facing External process configuration is replaced by scoped Name semantics; legacy `calledElement` and folder-only compositions require deterministic migration or compatibility resolution.

## Capabilities

### New Capabilities

- `scoped-process-naming`: Architectural Name syntax, pool package scope, qualified process/member resolution, deterministic composition mapping, and safe rename refactoring.

### Modified Capabilities

- `software-schematic-composition`: Resolve reusable compositions from qualified Names, create blank Name-derived folders, move complete composition units, and retain relative documentation links.
- `software-schematic-editor`: Present ID, Name, Label, and derived qualified identity while removing the user-facing External process field and supporting diagram-level rename.
- `ai-diagram-assistance`: Include scoped Names in context and constrain proposals to preserve identities, derive paths, and preview explicit Name/refactor changes.

## Impact

- Affects BPMN extension metadata, pool and node metadata controls, composition resolution, folder layout, tab registries, breadcrumbs, file confinement, automatic persistence, Markdown link resolution, and assistant schemas/validators/executors.
- Adds a transactional server operation for process rename/move and project-wide reference updates with collision detection and rollback.
- Requires migration tests for existing diagrams, `calledElement` values, nested compositions, case-insensitive filesystems, shared references, and open tabs.
