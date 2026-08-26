## Why

The current Data Graph notation relies on color and an inconsistent, weakly presented palette, which makes node meaning slower to recognize and authoring less deliberate. The diagram should use compact, explicit type icons and BPMN-like wrench menus so users can think and draw with minimal mental translation.

## What Changes

- Replace color-based property differentiation with half-size property nodes whose datatype is shown by a compact, centered type icon.
- Add a BPMN-style wrench action to property nodes that opens a labeled datatype palette, including String (`STR`), Integer (`INT`), Byte (`BYTE`), Date (`DATE`), and the complete supported datatype set using icons of four characters or fewer.
- Consolidate filter and mutation concepts into one half-size, thick-outline mutation node.
- Add a wrench action to mutation nodes with labeled choices for query, function, and transformation, each represented by a recognizable icon.
- Add a wrench action to intermediate nodes with choices for stack, queue, set, and map, using compact symbolic icons that remain legible at node scale.
- Restyle global and contextual palettes so icons and labels are centered, spacing and borders are intentional, and every action is understandable without guessing.
- Make object, property, and mutation nodes freely movable, retain their dropped positions instead of snapping back, and persist those positions through save/reopen and undo/redo.
- Position object-relationship labels above intermediate controls so labels never block selecting or opening the control's wrench palette.
- Persist selected property, mutation, and intermediate types and restore them consistently on import, export, undo, and redo.
- Replace persisted `filter` attachment nodes with typed `mutation` nodes and replace development fixtures using the original graph definition; backward compatibility is intentionally out of scope.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `data-graph-diagram-editing`: Change the Data Graph node notation, type-selection interactions, attachment semantics, palette presentation, and persistence contract.

## Impact

- Affects the custom `diagram-js` Data Graph renderer, palette and context-pad providers, modeling commands, rules, and direct-editing behavior.
- Affects node move handling, relationship rerouting, derived intermediate-control placement, and serialized geometry.
- Changes the development Data Graph JSON schema; existing development fixtures and documents using `filter` records are not retained.
- Requires updates to editor styling, fixtures, unit tests, compatibility tests, bundled web assets, and relevant documentation.
- Prefer repository-owned SVG/CSS or the already bundled BPMN icon font; adding a downloaded icon dependency is only warranted if its license and offline bundling are verified.
