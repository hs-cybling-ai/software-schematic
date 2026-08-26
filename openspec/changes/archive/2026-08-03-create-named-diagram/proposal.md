## Why

Users can open and edit existing diagrams, but they must leave the application to create a new BPMN or ArchiMate file. Adding an in-app creation flow removes that interruption and makes it straightforward to start a named diagram in any folder already visible in the workspace.

## What Changes

- Add a New Diagram command that is available when a workspace folder is open.
- Let the user choose any supported diagram type, enter a file name, and select the workspace root or a displayed folder as the destination.
- Validate the name and destination before creation, prevent accidental overwrite, and report actionable errors without disrupting the workspace.
- Create a valid starter document for the selected format, refresh the file tree, and open the new diagram in an editor tab.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `diagram-file-workspace`: Add safe creation of named, format-valid diagram files in the workspace root or a displayed folder, followed by discovery and opening.
- `macos-workspace-shell`: Add a native New Diagram command and a simple creation dialog for type, name, and displayed-folder placement.

## Impact

- Native SwiftUI workspace commands and creation UI.
- Workspace state and file-service APIs for validation, starter content, exclusive file creation, refresh, and open behavior.
- BPMN and ArchiMate starter-document definitions compatible with the embedded editors.
- Unit, integration, and UI coverage for creation, validation, placement, collision handling, and successful opening.
