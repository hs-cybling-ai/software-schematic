## 1. Canonical Relationship Document Model

- [x] 1.1 Replace the version-1 collection relationship shape with a stable source object, first-class edge-node identity and coordinates, label geometry, edge-node type, unique object target list, per-edge routes, and relationship-owned attachments; the edge node must not be encoded as a waypoint. Add `scalar` as the registry default with `0..1` cardinality and a blank icon treatment.
- [x] 1.2 Implement strict version-1 validation for targetless and multi-target relationships, rejecting missing or non-object endpoints, duplicate memberships, more than one scalar target, broken references, unknown intermediate subtypes, and the obsolete edge/control ownership shape.
- [x] 1.3 Implement deterministic serialization for the replacement version-1 definition without a legacy parser or migration path.
- [x] 1.4 Add document tests and fixtures covering targetless, heterogeneous multi-target, converging relationships, obsolete-version rejection, canonical output, and invalid topology errors.

## 2. Relationship Rendering and Geometry

- [x] 2.1 Refactor Data Graph import and export to expand each canonical relationship into one source segment, one independently positioned intermediate node, zero-or-more membership segments, and one relationship label.
- [x] 2.2 Update rendering and connection notation so source and membership direction is clear, membership segments do not duplicate labels, and dependent property and mutation edges retain target-driven notation.
- [x] 2.3 Make intermediate nodes movable and persist their committed positions while rerouting every source, membership, label, and attachment connection belonging to the relationship.
- [x] 2.4 Add collision-safe default geometry for new relationships, including readable targetless and fan-out layouts.

## 3. Authoring and Modeling Commands

- [x] 3.1 Add an object context action that creates a targetless labeled relationship and intermediate node as one undoable, redoable command.
- [x] 3.2 Add Connect object and Create object actions to the intermediate context palette with centered icons, accessible labels, keyboard focus behavior, and constrained object-only targeting.
- [x] 3.3 Implement membership commands that add an existing or newly created object target atomically, reject duplicate membership within one relationship, and allow different relationships to share the same target.
- [x] 3.4 Update property and mutation attachment creation so ownership is recorded against the relationship rather than a rendered segment.
- [x] 3.5 Update the intermediate type chooser to show Scalar (`0..1`) with a blank mark, make it the creation default, enforce its one-target limit, and prevent switching a multi-target relationship to scalar without deleting members.

## 4. Deletion and Mutation Semantics

- [x] 4.1 Implement membership-segment deletion that removes only the selected target membership and retains the relationship, remaining targets, attachments, and object nodes.
- [x] 4.2 Implement source-segment or intermediate-node deletion that removes the complete relationship topology and owned attachments while retaining former target object nodes.
- [x] 4.3 Verify creation, connection, movement, label editing, membership changes, and deletion each mark the tab dirty and round-trip correctly through undo and redo.

## 5. Integration and Regression Coverage

- [x] 5.1 Add modeler interaction tests for default scalar creation, blank scalar rendering and accessibility, scalar cardinality enforcement, intermediate palette contents, heterogeneous targets, duplicate prevention, two relationships converging on one object, and attachment ownership.
- [x] 5.2 Add visual and geometry assertions for intermediate selection, label placement, independent movement, membership routing, and save/reopen restoration.
- [x] 5.3 Replace current version-1 Data Graph fixtures and integration tests with the new definition and remove old-shape compatibility coverage.
- [x] 5.4 Run the Data Graph unit, modeler, workspace integration, lint, and build suites and resolve regressions without weakening the new validation rules.

## 6. Normalized Ontology Graph

- [x] 6.1 Replace the persisted relationship model with top-level `nodes`, `edges`, and `properties`, using `objectNode` and `edgeNode` node kinds and no `relationships` or `attachments` aggregates.
- [x] 6.2 Validate domain edges as `objectNode → edgeNode`, range edges as `edgeNode → objectNode`, exactly one domain per edge node, unique range targets, and scalar range cardinality.
- [x] 6.3 Generate deterministic IDs using `<edge-node-id>_domain` and `<edge-node-id>_range_<target-node-id>` and preserve independent labels and geometry.
- [x] 6.4 Persist property and mutation definitions in `properties` with a valid node-or-edge `ownerId`, deriving their canvas ownership connections during import.
- [x] 6.5 Replace fixtures and document/modeler tests with normalized ontology examples and rejection coverage for obsolete relationship aggregates.
- [x] 6.6 Run strict OpenSpec validation, web lint/tests/build, Swift integration tests, and rebuild the deployed application web resources.

## 7. Mutation Nodes and Signatures

- [x] 7.1 Move mutations from `properties` into first-class `mutationNode` records with persisted `mutationType`, label, and geometry.
- [x] 7.2 Add and validate `modifier` edges from `mutationNode` to `edgeNode`, using `<mutation-node-id>_modifier_<edge-node-id>` IDs.
- [x] 7.3 Create mutations from the edge-node palette with a modifier edge and allow properties owned by mutation nodes to define parameters.
- [x] 7.4 Update rendering, import/export, deletion, fixtures, documentation, and tests for mutation nodes, modifiers, and parameter properties.
- [x] 7.5 Run strict OpenSpec validation and the full web and Swift verification suites.

## 8. URI Property Datatype

- [x] 8.1 Add `uri` as a first-class registered property datatype with a visible `URI` mark and version-1 validation support.
- [x] 8.2 Update the normalized fixture, documentation, and document/modeler tests to define identifiers and mutation parameters such as `companyUrn` as URI properties.
- [x] 8.3 Run strict OpenSpec validation and the full web and Swift verification suites, rebuild the app, and redeploy it for testing.

## 9. Directional Function Icons

- [x] 9.1 Replace mutation marks with larger Unicode `←ƒ`, `ƒ→`, and `ƒ` icons for query, collection-update, and internal-state functions, with descriptive accessible labels.
- [x] 9.2 Update documentation and tests for the directional function semantics.
- [x] 9.3 Run strict OpenSpec validation and the full web and Swift verification suites, rebuild the app, and redeploy it for testing.

## 10. Object Functions

- [x] 10.1 Generalize modifier validation and connection rules so mutation nodes may target either object nodes or edge nodes with deterministic target-suffixed IDs.
- [x] 10.2 Add direct function creation to the object-node context palette and cover persistence, invalid topology, and authoring with document/modeler tests and documentation.
- [x] 10.3 Run strict OpenSpec validation and the full web and Swift verification suites, rebuild the app, and redeploy it for testing.

## 11. Adjustable Nodes

- [x] 11.1 Verify object, edge, mutation, and property nodes are independently movable through command-stack modeling and persist their adjusted coordinates.
- [x] 11.2 Cover edge-node label movement, connection rerouting, dirty state, undo, and redo while documenting all-node adjustability.
- [x] 11.3 Include adjustable-node coverage in full validation before rebuilding and redeploying the app.
