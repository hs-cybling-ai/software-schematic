## Why

Property and mutation connections currently share generic edge rendering, so their visual meaning can vary with connection source and property ownership is obscured by redundant edge labels. A single endpoint-driven notation will make dependent properties and mutation flows immediately recognizable throughout Data Graph diagrams.

## What Changes

- Render every edge whose target is a property as a dashed line, regardless of whether it originates at an object, property, or intermediate control.
- Suppress relationship labels on property-targeting edges and rely on the property node's semantic label displayed below the node.
- Render every edge whose target is a mutation as a solid line ending in a black circular target marker, regardless of source.
- Preserve the existing notation for edges whose target is neither a property nor a mutation.
- Apply the same classification to newly created connections and connections reconstructed from persisted documents.
- Add coverage for property and mutation edge notation across each supported source type and save/reopen behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `data-graph-diagram-editing`: Standardize edge line style, target marker, and label behavior according to whether the target is a property or mutation node.

## Impact

- Data Graph relationship creation, import reconstruction, rendering, direct label editing, and serialization in `web-editor/src/data-graph`.
- Data Graph CSS and SVG marker definitions in the web editor.
- Data Graph modeler tests and user-facing notation documentation.
- Existing stored documents remain readable; property-edge labels may remain in persisted data for compatibility but are no longer rendered as relationship labels.
