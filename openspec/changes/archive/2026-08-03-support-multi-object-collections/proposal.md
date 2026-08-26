## Why

Data Graph collection relationships currently assume exactly one source object, one intermediate collection control, and one target object. That prevents authors from modeling heterogeneous collections, adding another relationship into an existing collection target, or representing a collection relationship before its range is known.

## What Changes

- Allow one intermediate collection node to connect to more than one object node, so a collection can contain heterogeneous object types such as companies and persons.
- Add edge and object-node creation actions to the intermediate node palette.
- Allow an intermediate node to be added from an object without immediately creating or selecting a target object.
- Add `scalar` to the intermediate-node type palette as the default type for `0..1` cardinality, represented by a blank or missing center icon.
- Allow a new relationship edge, such as `worker_for`, to connect an object to an existing intermediate collection that already reaches the same target object as another relationship such as `works_at`.
- Preserve collection membership, labels, routing, editing, undo/redo, and persistence for branched and temporarily targetless relationships.
- Replace the development-only version-1 relationship definition in place; existing Data Graph files and tests are not migrated.
- Normalize the saved ontology as a property graph containing only `nodes`, `edges`, and `properties`; remove persisted `relationships` and `attachments` aggregates.
- Name node kinds consistently as `objectNode` and `edgeNode`, and type the two halves of an edge-node relationship as `domain` (`objectNode` to `edgeNode`) and `range` (`edgeNode` to `objectNode`).
- Generate deterministic topology IDs using `<edge-node-id>_domain` and `<edge-node-id>_range_<target-node-id>`.
- Represent functions as first-class `mutationNode` nodes connected to edge nodes by typed `modifier` edges, with function parameters represented as properties owned by the mutation node.
- Add `uri` as a first-class property datatype for identifiers and resource references such as `id` and `companyUrn`.
- Give mutation nodes directional function marks: `←ƒ` for queries returning collection items, `ƒ→` for collection updates, and `ƒ` for internal domain-state changes.
- Allow a mutation node to attach directly to an `objectNode` through a modifier edge, just as an object can own properties, for functions that operate on the object type rather than one of its relationships.
- Make every persisted node kind independently movable, including edge nodes, object nodes, mutation nodes, and property nodes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `data-graph-diagram-editing`: Extend collection relationship topology, palette authoring, validation, persistence, and interaction behavior to support multiple object members, shared targets, and targetless intermediate nodes.

## Impact

- Affects the Data Graph document model and validation rules for edges and intermediate controls.
- Affects custom `diagram-js` element creation, connection rules, routing, rendering, context-pad actions, deletion behavior, and command-stack integration.
- Replaces the version-1 development JSON definition and fixtures for collection relationships without backward compatibility.
- Requires modeler, document-validation, interaction, persistence, and regression tests for branched and targetless collection topologies.
