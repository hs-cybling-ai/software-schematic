## Why

The studio supports established diagram standards but lacks a fast, purpose-built notation for defining property-graph schemas and the operations that act on them. A lightweight data-graph diagram will let people and AI describe graph-oriented software structures visually, while establishing an extensible foundation for later mutation semantics.

## What Changes

- Add a new editable data-graph diagram type centered on labeled circular nodes and directed, labeled edges.
- Distinguish object nodes from property nodes, with property nodes rendered using a pastel-blue fill.
- Add a half-size, double-outline edge-intermediate control on object-to-object relationships, positioned midway between source and target; connect literal property nodes directly to their owning object.
- Support attaching property and filter nodes to an edge through its intermediate control; mutation authoring remains out of scope for this phase, but the extension point is reserved for thick green-outline mutation nodes attached to the same control in the next phase.
- Add a context palette for quickly creating nodes, connecting them, and adding supported edge attachments, with interaction speed comparable to BPMN editing.
- Add local serialization, loading, validation, dirty tracking, and export for the new diagram format.
- Extend workspace discovery and new-diagram creation to recognize the data-graph format.
- Remove nonfunctional ArchiMate support and its legacy dependency so the application can focus on BPMN-quality Data Graph editing with one current `diagram-js` generation.

## Capabilities

### New Capabilities

- `data-graph-diagram-editing`: Defines the core data-graph notation, context-palette authoring workflow, edge-intermediate controls, supported edge attachments, and editable persistence behavior.

### Modified Capabilities

- `diagram-file-workspace`: Extends supported diagram discovery and safe named diagram creation to include data-graph documents.

## Impact

- The web editor gains a data-graph rendering and editing adapter, palette/provider behavior, visual styles, model validation, and serialization.
- The native-to-web bridge and tab/editor routing gain a new diagram format and canonical file extension.
- Workspace discovery, file-type metadata, starter-document creation, and fixtures/tests expand to cover data-graph files.
- No mutation-node creation, definition, or execution is included; the intermediate edge control and attachment model must remain extensible to thick green-outline mutation nodes in the next phase.
