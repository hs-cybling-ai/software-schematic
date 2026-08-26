## Why

Object-to-object drawing currently creates a direct connection even though ordinary relationships require a first-class intermediate `edgeNode`, producing an inconsistent and incomplete topology. The editor also needs a non-recursive way to state that one object is semantically linked to another object—such as a coworker or manager being the same kind of Person—so authors can reuse the linked object's connections without manually recreating loops.

## What Changes

- Route ordinary relationship gestures through creation of a scalar `edgeNode`: a gesture ending on empty canvas terminates at the edge node, while one reaching another object places the edge node between the objects and creates the corresponding domain and range edges.
- Add direct, dashed object-link edges between object nodes for semantic references that do not create an intermediate `edgeNode`.
- Add a link-type modifier with an initial controlled vocabulary including `sameAs` and `subclassOf`, designed to accept additional registered link types.
- Present exactly two object-originating connection choices to authors: Relationship and Object link; domain and range remain internal segments of a relationship rather than additional authoring choices.
- Define linked-object connection inheritance so consumers can resolve the linked object's existing relationships without copying topology or recursively expanding cycles.
- Persist, validate, render, edit, delete, undo, and redo typed object-link edges while keeping them distinct from collection relationships and dependent-property links.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `data-graph-diagram-editing`: Correct object-to-object relationship creation and add typed, dashed object-link semantics with cycle-safe connection inheritance.

## Impact

- Data Graph interaction rules and context-palette/connect behavior.
- Diagram rendering, type selection, validation, serialization, import/export, and undo/redo commands.
- The normalized Data Graph edge schema and fixtures for direct typed object links.
- Tests covering ordinary relationship creation, semantic object links, inherited connection resolution, and cyclic link graphs.
