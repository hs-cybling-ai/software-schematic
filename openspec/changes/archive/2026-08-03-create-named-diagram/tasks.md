## 1. Format and File-Creation Foundation

- [x] 1.1 Add canonical filename extensions and minimal editor-compatible starter XML for every `DiagramFormat` case.
- [x] 1.2 Extend `WorkspaceFileService` with name normalization, format-extension validation, canonical destination validation, and collision-specific creation errors.
- [x] 1.3 Implement exclusive UTF-8 starter-document creation that cannot overwrite an existing filesystem item.
- [x] 1.4 Add file-service and format unit tests for valid root/nested placement, extension handling, invalid names, escaped or missing destinations, and existing/concurrently-created targets.

## 2. Workspace Creation Workflow

- [x] 2.1 Add a destination model and recursively derive the workspace root plus displayed directory nodes in recognizable hierarchy order.
- [x] 2.2 Add `WorkspaceStore` presentation state and validation APIs for opening, cancelling, and submitting the New Diagram flow.
- [x] 2.3 After successful creation, refresh discovery, resolve the created node, and reuse the canonical `open` path while preserving the created file if opening fails.
- [x] 2.4 Add workspace-store and integration tests covering displayed destinations, cancel behavior, successful creation/opening with a clean tab, stale folders, collisions, and post-write open failures.

## 3. Native Creation Interface

- [x] 3.1 Add a native New Diagram sheet with an accessible format picker, name field, workspace-relative destination picker, inline validation feedback, and Cancel/Create actions.
- [x] 3.2 Add File > New Diagram to the app commands, wire it to the shared sheet state, assign a standard shortcut, and disable it when no workspace is open.
- [x] 3.3 Add UI tests for command availability, every supported type, root and displayed-folder selection, invalid-name feedback, collision recovery, cancellation, and successful selected-tab opening.

## 4. Compatibility and Verification

- [x] 4.1 Add web-editor compatibility tests proving each starter XML document imports and exports through its matching adapter.
- [x] 4.2 Run Swift package tests and web-editor tests, then manually verify creation of BPMN and ArchiMate diagrams in the root and a displayed nested folder.
