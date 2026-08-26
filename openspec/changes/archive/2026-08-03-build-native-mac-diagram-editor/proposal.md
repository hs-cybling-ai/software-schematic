## Why

Architects need a focused native macOS workspace for organizing and editing diagram files without switching between a file browser and separate web tools. Establishing the application shell and two interoperable diagram formats now creates a usable foundation for future modeling capabilities.

## What Changes

- Add a native macOS application that uses the system dark appearance and presents a folder tree beside a tabbed editor workspace.
- Add File menu support for selecting a folder and recursively discovering supported diagrams within it.
- Embed a local web editor based on the latest compatible `diagram-js` ecosystem packages.
- Add ArchiMate editing through `archimate-js` and BPMN editing through `bpmn-js`.
- Allow supported diagrams to be opened in tabs, edited, saved to their original files, and protected from accidental loss when dirty.
- Add the build, bundling, and test structure needed to package the JavaScript editor assets inside the macOS application.

## Capabilities

### New Capabilities

- `macos-workspace-shell`: Native dark-mode window, folder sidebar, File menu commands, and tabbed editor workspace behavior.
- `diagram-file-workspace`: Folder authorization, supported-file discovery, file opening, tab lifecycle, dirty-state handling, and persistence.
- `bpmn-diagram-editing`: Loading, editing, validation/error reporting, and saving of BPMN diagram files using `bpmn-js`.
- `archimate-diagram-editing`: Loading, editing, validation/error reporting, and saving of ArchiMate diagram files using `archimate-js`.

### Modified Capabilities

None.

## Impact

- Introduces a macOS application target, native UI and document/workspace state, sandbox-aware folder access, and a Swift-to-JavaScript bridge.
- Introduces bundled web assets plus npm-managed `diagram-js`, `bpmn-js`, and `archimate-js` dependencies and their build pipeline.
- Reads and writes user-selected diagram files; no network service or external API is required at runtime.
- Adds native, web-editor, bridge, file-I/O, and end-to-end tests for supported workflows.
