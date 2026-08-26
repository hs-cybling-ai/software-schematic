## Context

Diagram Studio currently discovers and opens existing BPMN and ArchiMate files beneath an authorized workspace. `WorkspaceStore` owns native commands and tab state, while `WorkspaceFileService` enforces canonical workspace containment and persistence safety. Creation crosses the SwiftUI command/dialog layer, workspace orchestration, file-system validation, format-specific starter XML, tree refresh, and editor loading.

The destination list must feel native and simple: users choose from the workspace root and folders already represented by the displayed file tree. Because the current discovery tree omits empty and unsupported-only folders, the creation UI is intentionally scoped to displayed destinations rather than becoming a general filesystem browser.

## Goals / Non-Goals

**Goals:**

- Create a valid starter document for every `DiagramFormat` case.
- Let users name the diagram and place it in the workspace root or a displayed folder.
- Keep creation within the authorized workspace and never overwrite an existing item.
- Refresh discovery and open the successfully created diagram immediately.
- Keep validation failures inside the creation flow and operational failures visible through existing workspace error handling.

**Non-Goals:**

- Creating folders, renaming files, templates, or diagrams outside the active workspace.
- Supporting additional diagram formats.
- Reserving a filename across multiple processes or resolving concurrent external creation beyond exclusive-write failure reporting.

## Decisions

### Use a native sheet backed by a small creation draft

`WorkspaceView` will present a SwiftUI sheet containing a format picker, name field, and destination picker. `WorkspaceStore` will expose the available destinations and perform creation, while transient form state stays in the sheet. The New Diagram command opens the same sheet through store-owned presentation state, making menu and future toolbar entry points consistent.

An `NSSavePanel` was considered, but it exposes arbitrary filesystem navigation and makes it harder to restrict choices to displayed workspace folders. Inline sidebar actions were also considered, but they make format selection and validation less clear.

### Derive destinations from displayed workspace nodes

The destination model will always include the workspace root and recursively flatten directory nodes from `fileTree`, using canonical URLs as identity and relative paths as labels. This guarantees every offered destination is already visible and within the active workspace. If a folder disappears before submission, the file service revalidates existence, directory status, and containment and fails without creating a file.

Maintaining a second unfiltered directory scan was considered, but would allow placement in folders the user cannot see and duplicate discovery logic.

### Centralize filename normalization and exclusive creation in the file service

The file service will trim surrounding whitespace, reject empty names, path separators, `.`/`..`, and names whose supplied extension conflicts with the selected format, then append the format's canonical extension when absent. It will reject an existing destination and create the UTF-8 starter document with exclusive semantics so creation cannot overwrite a file that appears after validation.

Keeping this logic in the view was considered, but would couple filesystem safety to one UI path and make behavior harder to test. Automatic numeric suffixes were rejected because they silently change the name the user chose.

### Define starter XML on `DiagramFormat`

Each supported format will provide its canonical extension and a minimal, deterministic, editor-compatible starter XML document. BPMN will contain definitions, an empty process, and a diagram plane; ArchiMate will contain a named model and an empty diagram view. Stable starter structure makes unit and adapter compatibility tests straightforward, while identifiers may be generated per creation when required for model validity.

Generating a new model through the web editor was considered, but would add a bridge round trip and require an editor session before a file exists. Bundled template files were considered, but the very small format-specific defaults are easier to keep type-safe beside format metadata.

### Reuse the existing open path after successful creation

After the write succeeds, `WorkspaceStore` refreshes the file tree, resolves the newly created file node, and calls the existing `open` behavior. This preserves canonical tab identity, metadata initialization, and editor loading. A creation failure leaves the sheet open with an actionable validation message; a post-write refresh/open failure reports the created path and keeps the file safely on disk.

## Risks / Trade-offs

- [Displayed folder disappears between selection and creation] → Revalidate the canonical destination immediately before the exclusive write and show a recoverable error.
- [Starter XML is accepted by one library version but not another] → Cover every starter document with adapter import/export compatibility tests.
- [A race creates the same filename after validation] → Use exclusive file creation and translate the collision into a specific user-facing error.
- [A file is created but cannot be opened] → Do not delete user data; refresh the tree and report that creation succeeded but opening failed.
- [The displayed tree excludes empty folders] → Document and test the deliberate displayed-folder scope; folder creation and broader browsing remain out of scope.
