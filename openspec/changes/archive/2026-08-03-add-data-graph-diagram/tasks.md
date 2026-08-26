## 1. Format Contract and Native Workspace Integration

- [x] 1.1 Confirm the Data Graph user-facing name, canonical extension, starter-document fields, and versioned JSON schema; add valid and invalid fixtures.
- [x] 1.2 Add the data-graph case to Swift diagram-format detection, display metadata, and editor routing with unit coverage.
- [x] 1.3 Extend workspace discovery and safe named creation to recognize data-graph files and write a valid starter document without overwriting existing items.
- [x] 1.4 Extend bridge load/export payload handling and integration tests for isolated data-graph tabs, failures, and safe saves.

## 2. Data-Graph Model and Persistence

- [x] 2.1 Add web-editor model types and an element factory for objects, properties, filters, relationships, intermediate controls, and extensible typed attachment ownership that can add mutation nodes in a later phase.
- [x] 2.2 Implement strict, non-mutating JSON import validation for versions, element types, unique IDs, references, ownership, and required intermediate controls.
- [x] 2.3 Implement deterministic JSON export preserving semantic records, movable label geometry, node geometry, edge routing, control identity, and attachments.
- [x] 2.4 Add parser, validator, round-trip, malformed-input, and deterministic-output tests using representative fixtures.

## 3. Rendering and Geometry

- [x] 3.1 Model every visible Data Graph item as a `diagram-js` shape or connection, including real half-size intermediate-control nodes and two-segment logical relationships.
- [x] 3.2 Add theme tokens and accessible styling for pastel-blue property fill, object nodes, labels, selections, and connection states.
- [x] 3.3 Render centered, movable labels as native diagram elements and support direct editing for node, filter, and edge labels.
- [x] 3.4 Implement circular-outline docking, direct object-property edges, plus half-size double-outline controls only for object-object relationships.
- [x] 3.5 Add rendering and geometry tests for centered movable labels, direct literal edges, outline docking, radial non-overlapping append placement, logical edge segments, and control repositioning.

## 4. Modeling Rules and Context Palette

- [x] 4.1 Implement Data Graph command handlers and rules through the native `diagram-js` command stack.
- [x] 4.2 Ensure only object-object relationships own an intermediate control while object-property literal links remain direct.
- [x] 4.3 Implement edge-owned property and filter attachment commands, validation, and single-command cascade deletion with undo and redo.
- [x] 4.4 Implement radial collision-aware native context-pad append actions for nodes, relationships, edge properties, and filters.
- [x] 4.5 Hide or disable invalid and deferred mutation actions, and add rule/provider tests for every eligible selection and invalid ownership case.

## 5. Editor Lifecycle and Application Assembly

- [x] 5.1 Add a data-graph editor adapter that imports sessions, retains per-tab modelers, reports command-stack mutations, exports JSON, and isolates diagnostics.
- [x] 5.2 Register the adapter with existing web editor format routing and native bridge correlation without changing BPMN behavior.
- [x] 5.3 Add BPMN-parity interaction tests for selection, dragging, panning, zooming, direct editing, connection creation, dirty state, undo/redo, tab switching, and failures.
- [x] 5.4 Rebuild and bundle production web assets into the macOS application and verify the app uses only local resources.

## 6. End-to-End Verification and Documentation

- [x] 6.1 Add an end-to-end workflow test that creates, discovers, opens, edits, saves, closes, and reopens a data-graph diagram.
- [x] 6.2 Add a visual interaction fixture covering object and property nodes, labeled arrows, edge controls, property attachments, and filter attachments.
- [x] 6.3 Update README and compatibility documentation with the Data Graph format, extension, first-phase notation, and the deferred thick green-outline mutation-node convention.
- [x] 6.4 Run Swift tests, web lint/tests/build, and the packaged-app smoke test; resolve regressions and record any platform-specific limitations.
