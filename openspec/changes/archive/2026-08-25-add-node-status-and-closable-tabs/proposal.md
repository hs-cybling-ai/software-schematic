## Why

The SSW editor needs clearer, direct controls for managing open work and communicating which graph nodes an LLM may create, change, or leave untouched. Visible tab close buttons and color-coded node statuses make both actions understandable at a glance while establishing the UI contract for future GrafeoDB persistence.

## What Changes

- Add an `x` close button to each non-root editor tab, while keeping the project's root `main.bpmn` permanently open because it is the navigation entry point to other diagrams.
- Make the Markdown mode button communicate its next action: show a pencil in rendered/read mode and a book in source-edit mode; the book saves and returns to rendered/read mode.
- Repair the top-bar fullscreen button so it enters and exits browser fullscreen, reflects the current fullscreen state, and reports failures instead of acting only as a fit-to-viewport control.
- Include the initialized project directory name in the browser title using `Software Schematic - <project name>`.
- Let users assign one of four statuses to an SSW diagram node: `new`, `locked`, `modify`, or `open`.
- Treat node status as an LLM authoring hint: create this node, do not touch it, changes are allowed/expected, or no hint, respectively.
- Render node status with a consistent fill color: green for `new`, grey for `locked`, orange for `modify`, and white for `open`.
- Keep the UI and model boundary ready for node status to be stored in GrafeoDB when persistence is implemented, without adding that backend integration in this change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `software-schematic-editor`: Add explicit close controls to SSW web tabs, project-aware browser titles, a working fullscreen toggle, an unambiguous Markdown read/edit toggle, and editable node authoring statuses with their LLM semantics and accessible color-coded presentation.

## Impact

- Affects the SSW browser tab strip, fullscreen and Markdown mode controls, BPMN modeler overlays/rendering, selected-item metadata controls, in-memory status state, and tests in `software-schematic-web`.
- May extend SSW local-server payloads when GrafeoDB persistence is added later; this change keeps status client-side.
- Adds a read-only project-metadata response to the local Rust server so the browser can obtain the authoritative project directory name without receiving its absolute path.
- Establishes a future GrafeoDB field contract; no GrafeoDB dependency or persistence implementation is introduced now.
