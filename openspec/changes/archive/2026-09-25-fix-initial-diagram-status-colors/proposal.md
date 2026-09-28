## Why

Persisted implementation-status colors are not rendered when a BPMN or CMMN diagram first opens. The status marker is currently applied only when an element is selected, so the canvas initially misrepresents development scope until the user clicks each node.

## What Changes

- Apply the normalized implementation-status marker to every eligible diagram element immediately after XML import and status hydration.
- Keep selection, inspector updates, status edits, and accessible labels consistent with the same shared status-rendering behavior.
- Reapply status presentation after any workflow that reimports XML into an existing tab.
- Add regression coverage proving persisted `new`, `modify`, `locked`, and default `open` presentation is established without element selection for both BPMN and CMMN diagrams.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `software-schematic-editor`: Clarify that persisted implementation statuses and their non-color accessible presentation are restored as part of initial diagram rendering, before user interaction.

## Impact

- Affects the web editor's diagram-open and XML-reimport lifecycle in `software-schematic-web/src/main.js`.
- Uses the existing adapter status APIs, canvas marker classes, and CSS colors; no schema, API, dependency, or persisted-format changes are required.
- Extends web editor tests to cover initial and repeated status rendering across BPMN and CMMN tabs.
