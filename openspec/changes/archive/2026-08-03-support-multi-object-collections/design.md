## Context

The current Data Graph document treats an object-to-object relationship as one logical edge with a required control owned by that edge. In the canvas, that logical edge is expanded into a source segment, a derived midpoint control, and one target segment. The validator therefore requires two object endpoints and exactly one control per edge. This ownership model cannot represent a collection before it has a target or fan one collection out to multiple object types.

This change spans the persisted document, import/export validation, `diagram-js` business objects, connection rules, context-pad authoring, geometry, deletion, and undo/redo. The product and Data Graph format are still in development, so the new model replaces the version-1 definition in place without a compatibility path or version bump.

## Goals / Non-Goals

**Goals:**

- Represent a collection relationship as one source object, one intermediate collection node, and zero or more object targets.
- Let users create a targetless intermediate node from an object, connect it to an existing object, or create a new object from it.
- Permit multiple intermediate relationships from one source to converge on the same object target.
- Support scalar `0..1` relationships alongside multi-valued collection relationships.
- Preserve heterogeneous target membership, labels, collection types, geometry, and attachments through save/reopen and undo/redo.
- Replace existing development fixtures and documents with the new format.

**Non-Goals:**

- Defining runtime query or collection-evaluation semantics.
- Allowing property or mutation nodes to become collection members.
- Sharing one intermediate node between multiple source objects.
- Inferring or enforcing a common schema across heterogeneous member object types.

## Decisions

### Persist the ontology as a normalized property graph

The version-1 document contains only `nodes`, `edges`, and `properties`. `nodes` contains `objectNode` and `edgeNode` records with stable IDs, labels, coordinates, and applicable semantic type data. `edges` contains explicit typed topology. `properties` contains property and mutation definitions with geometry and an `ownerId` referencing either a node or an edge. There is no persisted `relationships`, `controls`, or `attachments` aggregate.

This keeps the ontology self-describing in the same property-graph form that it defines. A relationship aggregate was considered and rejected because it duplicated topology already represented by nodes and edges and required special reconstruction logic.

### Type and name relationship topology consistently

A `domain` edge connects `objectNode → edgeNode`; a `range` edge connects `edgeNode → objectNode`. Each edge node has exactly one incoming domain edge and zero or more outgoing range edges while being authored. It becomes a useful completed definition once at least one range exists. Scalar edge nodes permit at most one range; collection edge nodes permit multiple unique range targets.

Generated IDs use `<edge-node-id>_domain` for the domain and `<edge-node-id>_range_<target-node-id>` for each range. Including the target ID keeps range IDs unique during fan-out. User-visible labels remain independent from IDs.

### Render one semantic relationship as a small topology

The canvas renders the normalized graph directly: a domain edge into the edge node and zero or more range edges out. Like a BPMN gateway, the edge node is a routing node and only forms a useful completed definition once it has an outgoing range. Zero ranges remain a temporary authoring state. The edge node uses its own stored coordinates and is not derived from, or stored inside, edge waypoints.

For a newly created single-target relationship, place the edge node at a collision-safe initial location between the objects and allow it to move independently for readable fan-out layouts. This initial placement does not make it an edge waypoint.

All persisted visual node kinds are independently adjustable: `objectNode`, `edgeNode`, `mutationNode`, and property nodes. Movement uses the model command stack, reroutes attached connections, moves an edge node's external label with it, marks the document dirty, and persists the new center coordinates. Undo and redo restore both geometry and routing.

### Give each intermediate relationship exactly one source and independent identity

An object may own multiple relationships, and each relationship has its own intermediate node even when several relationships target the same object. Thus `works_at` and `worker_for` remain separately labeled and typed while both may include the same Company node. One shared control with multiple incoming labeled edges was considered, but it would conflate collection type, attachments, and deletion ownership across distinct relationships.

### Author targets from the intermediate context palette

The edge-node palette adds two prominent completion actions: Connect object, which starts a constrained connection to an existing object, and Create object, which creates a new object and adds it as a target in one undoable command. Creating an edge node from an object may initially create only the relationship source and edge node, but that state is visibly incomplete until a target is added. Duplicate membership to the same target within one relationship is rejected, while convergence from different relationships is allowed.

### Treat scalar as the default intermediate type

Add `scalar` to the intermediate-node type registry and make it the default for newly created relationships. Scalar represents cardinality `0..1`: it permits a targetless relationship or exactly one target object, but rejects a second target until the user changes the intermediate type to a multi-valued collection type. Its center mark is intentionally blank; the double-outline node shape, accessible label, tooltip, and visible `Scalar (0..1)` palette row communicate its meaning without inventing another symbol.

Stack, queue, set, and map remain multi-valued types and continue to display their existing marks. Changing a multi-target relationship to scalar is disabled while it has more than one target, avoiding silent member deletion. Automatically converting scalar to a collection when a second target is added was considered, but explicit type selection better preserves author intent and keeps cardinality validation predictable.

### Store property definitions as properties

Property definitions persist in the top-level `properties` array. Each has a stable ID, label, datatype, coordinates, and `ownerId`. The owner may be an object node, edge node, mutation node, domain edge, range edge, or modifier edge. Ownership connections shown on the canvas are derived and are not duplicated in `edges`.

The property datatype registry includes `uri` as a first-class semantic type with the visible `URI` mark. It is used for identifiers and resource references such as `id` and `companyUrn`; a URN is represented as a URI rather than as a separate datatype. The diagram records the contract but does not resolve the URI or prescribe target-language validation and storage behavior.

### Model functions as mutation nodes and modifier edges

Functions persist as first-class `mutationNode` records with a stable ID, label, coordinates, and `mutationType` such as query, function, or transformation. A typed `modifier` edge connects a mutation node to an `edgeNode` or `objectNode`. An edge-node target indicates that the function modifies or queries that relationship definition and is applied in the context of its domain object. An object-node target indicates that the function operates directly on that object type, parallel to defining properties on the object. Generated modifier IDs use `<mutation-node-id>_modifier_<target-node-id>` for either target kind.

Mutation marks communicate data direction with Unicode arrow characters. A query that returns items from the collection uses `←ƒ`; a function that updates the collection uses `ƒ→`; and a transformation that modifies internal state of the domain object uses the non-directional `ƒ`. These marks use a larger mutation-specific font size than compact property datatype marks so the arrow and function glyph remain readable inside the half-size node. Accessible labels state the same semantics without relying on glyph interpretation.

Function parameters are ordinary property definitions whose `ownerId` is the mutation node. For example, an `add` mutation may own a `companyUrn` property, while a `searchCompanies` mutation may own a `name` property. The diagram defines these contracts but does not prescribe lookup, execution, or binding behavior.

A future runtime may implement each mutation as a standalone class, register instances in a mutation collection on the domain object, and bind them dynamically to generated relationship code. That runtime pattern is intentionally non-normative future work. Persisting mutation signatures explicitly now gives code-generating LLMs stable topology and parameter context without requiring them to infer behavior from prose or naming.

### Replace the version-1 development definition in place

The importer and exporter continue to identify documents as version 1, but version 1 now means the new relationship representation. Repository tests and fixtures are replaced rather than migrated. The prior edge/control ownership shape is invalid under the new definition. A version bump or migration adapter was considered but rejected because the product is still in development and has no release-compatibility requirement.

## Risks / Trade-offs

- [Replacing version-1 persistence touches most Data Graph tests and fixtures] → Keep the modeler operating on one canonical shape and replace all development fixtures together.
- [Branched routes can overlap labels or the control] → Store control position and target routes independently, provide deterministic defaults, and retain collision-safe relationship label placement.
- [Deleting a segment could accidentally delete the whole relationship] → Distinguish source segment, membership segment, and relationship-control deletion commands and test every cascade plus undo/redo.
- [Existing APIs may assume one target per edge] → Replace edge-centric lookup helpers with relationship-centric helpers throughout the development codebase.
- [A targetless relationship may look incomplete] → Render its labeled source segment and collection control normally and expose its target-creation actions immediately.
- [A blank scalar icon may be mistaken for an untyped node] → Keep the double outline, expose `Scalar (0..1)` in the chooser and accessible name, and make scalar a real persisted registry value rather than a missing subtype.

## Migration Plan

1. Replace version-1 validation and serialization tests with the new relationship definition.
2. Update the modeler to import, render, edit, and export canonical relationships while retaining direct dependent edges.
3. Add palette actions, connection constraints, deletion cascades, geometry, and undo/redo behavior.
4. Replace all version-1 development fixtures and remove old-shape compatibility expectations.
5. Roll back by reverting the feature and its development fixtures together.

## Open Questions

- None blocking. The implementation should use clear default labels for newly created targetless relationships while preserving the existing direct-editing workflow.
