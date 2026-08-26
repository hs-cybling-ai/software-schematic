## 1. Symbol Model and Persistence

- [ ] 1.1 Define pool package, process, and member Name extension metadata and round-trip fixtures alongside BPMN ID and Label.
- [ ] 1.2 Implement lowerCamelCase package/member and UpperCamelCase process validators with case-insensitive collision keys and actionable errors.
- [ ] 1.3 Implement qualified process and member symbol resolution from pool, owning diagram, and local Name context.
- [ ] 1.4 Implement deterministic bidirectional mapping between qualified process Names and confined composition folders.
- [ ] 1.5 Add unit tests for valid and invalid identifiers, nested packages, identical local names in different scopes, shared process symbols, and path confinement.

## 2. Legacy Migration and Compatibility

- [ ] 2.1 Inventory legacy folders, pool/lane ID paths, labels, and `calledElement` references into a non-mutating migration report.
- [ ] 2.2 Implement unambiguous legacy Name inference and compatibility aliases without moving existing content automatically.
- [ ] 2.3 Surface ambiguous migrations and require explicit user resolution before changing a folder or reference.
- [ ] 2.4 Mirror qualified process symbols into standard BPMN `calledElement` only as derived compatibility serialization.
- [ ] 2.5 Add migration tests for shared legacy references, nested folders, ambiguous names, Unicode labels, and projects reopened before migration completes.

## 3. Name-Based Composition Operations

- [ ] 3.1 Replace arbitrary-path composition creation/navigation requests with qualified process Name requests and blank deterministic composition creation.
- [ ] 3.2 Resolve multiple nodes with the same qualified process Name to one canonical diagram, documentation unit, and retained tab.
- [ ] 3.3 Ensure ordinary tasks and events derive `package.Process#member` symbols without creating composition folders.
- [ ] 3.4 Remove user-facing External process handling while retaining confined compatibility reads for migrated BPMN.
- [ ] 3.5 Add server and browser integration tests for creation, navigation, shared reuse, member ownership, and absent/invalid Names.

## 4. Transactional Process Rename

- [ ] 4.1 Build a project-wide process symbol/reference index covering open and closed BPMN diagrams.
- [ ] 4.2 Add a preflight API that derives the destination, affected references, folder contents, and case-insensitive collisions without mutation.
- [ ] 4.3 Implement atomic folder move and reference rewrite with complete before-state and rollback, including safe case-only renames.
- [ ] 4.4 Rekey retained modelers, tabs, breadcrumbs, documentation targets, queued saves, and assistant revisions after server commit.
- [ ] 4.5 Implement member Name edits as local symbol changes that never move the owning composition.
- [ ] 4.6 Protect the root navigation diagram from rename or move operations.
- [ ] 4.7 Add end-to-end tests for successful rename, shared references, nested content/assets, collisions, staged failure rollback, open tabs, and root protection.

## 5. Editor Naming Experience

- [ ] 5.1 Add accessible ID, Name, Label, and read-only Qualified Name controls appropriate to pool, process, task, and event selections.
- [ ] 5.2 Remove the External process control and make Open composition resolve exclusively from qualified process Name.
- [ ] 5.3 Add diagram-level process metadata with rename preview showing old/new symbols, folder move, and affected reference count.
- [ ] 5.4 Add keep-target versus rebind interaction when moving a named node across pool or process scope.
- [ ] 5.5 Add browser tests for independent Label editing, inline Name validation, derived symbol display, rename cancellation, keyboard/focus behavior, and scope-changing moves.

## 6. Portable Documentation

- [ ] 6.1 Make rendered diagram and node Markdown resolve relative links from the folder containing the owning `main.bpmn`.
- [ ] 6.2 Preserve `main.md`, `docs/`, assets, and internal relative link text unchanged during process rename.
- [ ] 6.3 Confine resolved Markdown and asset targets to `schematics/` and define qualified-Name handling for cross-process references.
- [ ] 6.4 Add tests for diagram Markdown, node Markdown under `docs/`, local assets, folder moves, path escape, and unchanged link content.

## 7. AI Assistant Contract

- [ ] 7.1 Extend semantic snapshots with local Name, qualified symbol, Label, kind, owning scope, and derived composition identity.
- [ ] 7.2 Replace provider-supplied composition paths with typed process/member Name and process-rename operations in the strict output schema.
- [ ] 7.3 Update Codex, Claude, OpenAI, and fake-provider prompts/fixtures to preserve existing ID, Name, and Label unless separately changed.
- [ ] 7.4 Validate Name syntax, scope, kind, deterministic destinations, implicit shortening, locked nodes, collisions, and stale identity before preview and approval.
- [ ] 7.5 Preview process rename as a symbol change, derived folder move, reference update set, and unchanged relative documentation links.
- [ ] 7.6 Route approved AI process renames through the same transactional refactor as manual rename and retain coordinated revert behavior.
- [ ] 7.7 Add contract and end-to-end tests for retained labels, full parent-derived names, same-named members in different scopes, shared processes, malicious paths, and rollback.

## 8. Release and Documentation

- [ ] 8.1 Document ID/Name/Label semantics, Java-style naming grammar, pool scope, qualified process/member notation, reuse, rename, and migration behavior.
- [ ] 8.2 Document diagram-relative Markdown links and confirm implementation bindings remain in Markdown rather than structured Name metadata.
- [ ] 8.3 Run strict OpenSpec validation, web and Rust suites, production asset build, release build, and manual browser scenarios for creation, reuse, rename, and migration.
- [ ] 8.4 Build the next SSW release and verify a fresh runtime update preserves project diagrams, documentation, assets, and assistant authentication selection.
