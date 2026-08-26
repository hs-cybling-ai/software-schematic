## 1. Type Model and Document Contract

- [x] 1.1 Add a centralized registry for property datatypes, mutation types, and intermediate collection types with stable keys, labels, compact marks, defaults, and accessibility text.
- [x] 1.2 Revise Data Graph validation and serialization to persist `dataType`, `mutationType`, and `collectionType`, remove `filter`, and reject unknown or obsolete values.
- [x] 1.3 Replace starter, valid, invalid, and test fixtures with documents conforming to the revised development graph definition.
- [x] 1.4 Add document tests for valid subtype round trips, deterministic output, defaults, unknown subtype rejection, and obsolete filter rejection.

## 2. Modeling and Type Commands

- [x] 2.1 Update creation and import paths so object nodes remain full-size while property and mutation nodes use the half-size geometry and default registered subtypes.
- [x] 2.2 Replace filter attachment modeling, connection rules, deletion behavior, and export logic with mutation attachment semantics.
- [x] 2.3 Persist collection types on intermediate controls without changing their edge-derived midpoint ownership and positioning.
- [x] 2.4 Implement one command-stack-backed subtype update command and verify dirty tracking, undo, and redo for all three subtype fields.
- [x] 2.5 Fix shape-move handling so object, property, and mutation drops remain at their committed coordinates while only connected routes and derived intermediate controls are recalculated.
- [x] 2.6 Serialize final registry geometry with size-aware center conversion and restore it without snap-back on import, save/reopen, undo, or redo.

## 3. Rendering and Wrench Interaction

- [x] 3.1 Render neutral property nodes with centered datatype abbreviations and external semantic labels, removing blue semantic styling.
- [x] 3.2 Render neutral, half-size mutation nodes with a thick outline, centered query/function/transformation marks, and external semantic labels.
- [x] 3.3 Render stack, queue, set, and map marks inside intermediate controls using repository-owned SVG/text primitives with consistent optical sizing.
- [x] 3.4 Add wrench context-pad actions for property, mutation, and intermediate nodes and an anchored registry-driven chooser with visible icon-and-label rows.
- [x] 3.5 Ensure the chooser supports keyboard focus, activation, dismissal, accessible names, tooltips, and selection-state announcements.

## 4. Palette Visual Refinement

- [x] 4.1 Replace filter actions with mutation actions in global and contextual authoring palettes and ensure invalid actions remain omitted or disabled.
- [x] 4.2 Introduce shared palette tokens and styles for strong borders, centered icon cells, aligned labels, deliberate spacing, and visible hover and focus states.
- [x] 4.3 Verify compact marks, labels, palettes, and context pads at representative zoom levels and in increased-contrast and monochrome conditions.

## 5. Verification and Delivery

- [x] 5.1 Extend modeler tests for persistent object/property/mutation movement, connected-edge rerouting, intermediate repositioning, geometry undo/redo, type marks, wrench availability, chooser contents, mutation attachments, and deletion cascades.
- [x] 5.2 Extend interaction tests for keyboard operation, accessible labeling, dirty state, subtype persistence, undo, redo, and invalid-type import isolation.
- [x] 5.3 Run the web-editor lint and test suites, rebuild bundled web assets, and run affected Swift integration tests.
- [x] 5.4 Update Data Graph documentation to describe the revised notation, complete supported datatype catalog, compact icon vocabulary, and no-compatibility development reset.
- [x] 5.5 Position controlled-edge labels above intermediate nodes, normalize overlapping imported positions, test click-area clearance and persistence, rebuild, and redeploy the application.
