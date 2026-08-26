## Context

The Data Graph editor is a custom `diagram-js` modeler. It currently renders object and property nodes at one size, distinguishes properties and filters with fill colors, represents relationship controls as untyped double circles, and exposes compact icon-only palette/context-pad actions. The document format is version 1 and permits `object`, `property`, and `filter` nodes plus property/filter edge attachments.

This change crosses rendering, modeling, interaction, persistence, compatibility, styles, and tests. It must remain fully offline in the sandboxed macOS app and retain command-stack undo/redo semantics.

## Goals / Non-Goals

**Goals:**

- Make element purpose immediately readable without depending on color.
- Make property, mutation, and intermediate types quick to set through a familiar wrench interaction.
- Use compact type marks that remain clear at half-node size.
- Make every palette action deliberately aligned, bordered, labeled, keyboard reachable, and screen-reader understandable.
- Preserve selected types deterministically in the revised development document contract.
- Make user-positioned graph nodes remain at their committed drop coordinates and restore those coordinates on reopen.

**Non-Goals:**

- Execute queries, functions, transformations, or collection operations.
- Define database-specific scalar types or query languages.
- Change object-node size or relationship routing semantics.
- Download fonts or icons at runtime.
- Read, migrate, or preserve documents written against the original development graph definition.

## Decisions

### Use semantic type fields with a centralized registry

Add `dataType` to property nodes, `mutationType` to mutation nodes, and `collectionType` to intermediate controls. A repository-owned registry defines stable keys, display labels, compact marks, accessibility labels, defaults, and applicability. Initial values are:

- Property: the currently supported scalar set, including `string`/`STR`, `integer`/`INT`, `byte`/`BYTE`, and `date`/`DATE`; every rendered mark is four characters or fewer.
- Mutation: `query`, `function`, and `transformation` with simple symbolic SVG marks.
- Intermediate: `stack`, `queue`, `set`, and `map`, using three horizontal lines, three vertical lines, `{}`, and `{:}` respectively. `{:}` is preferred to `{;}` because the colon more conventionally suggests key/value association.

The registry is the single source for rendering and menus, preventing drift between persisted values and visible choices. Alternatives considered were free-form type strings and embedding glyphs directly in providers; both make validation and consistent presentation harder.

### Render compact marks with repository-owned SVG and text

Use inline SVG primitives for wrench, query, function, transformation, and collection symbols, and text only for short datatype abbreviations. Reuse the bundled BPMN icon font only if an existing glyph matches exactly. Do not introduce a network-loaded character set; if a third-party asset is ultimately required, vendor only the minimal licensed asset with attribution.

This keeps the application offline, scalable, themeable, and accessible. An external icon font was considered, but it adds licensing and glyph-alignment risk for little benefit.

### Separate semantic labels from type marks

Object nodes remain full-size. Property and mutation nodes use the existing half-size control diameter. Their compact type mark is centered inside the circle; the editable semantic label is placed as an external `diagram-js` label so longer names do not compete with the type mark. Intermediate controls retain their edge-derived position and display their collection mark in the center.

Color is no longer semantic: neutral fills and the standard graph stroke are used for all types. Mutation nodes use a deliberately thicker outline. Selection and hover may still use the shared interaction accent because that color communicates state, not type.

### Implement wrench menus as command-backed replace menus

Eligible shapes expose a wrench entry in their context pad. Invoking it opens one common type chooser anchored to the selected node. Entries use a fixed icon cell plus visible text label, center their content consistently, show a strong boundary and focus state, and support pointer and keyboard selection. Choosing a value runs a notation-specific command so type changes participate in dirty tracking, undo, and redo.

This mirrors BPMN's replace-menu mental model while allowing the Data Graph type registry and visuals to remain notation-specific. Independent bespoke popovers were considered but would duplicate positioning, dismissal, and accessibility behavior.

### Replace filters with typed mutation attachments

The authoring model exposes one `mutation` attachment node rather than separate filter and mutation concepts. `query` covers predicate/read concerns, `function` covers callable behavior, and `transformation` covers value or structure changes. Intermediate controls offer property and mutation attachment actions.

Revise the development document contract in place to persist semantic subtype fields and remove `filter` from the allowed node and attachment sets. Existing development fixtures are replaced rather than migrated. Invalid or obsolete node and subtype values are rejected before the live model changes.

Keeping both filter and mutation was considered, but it would preserve overlapping concepts and a larger palette—the opposite of the simplification goal.

### Keep palette presentation consistent across global and contextual tools

Introduce shared palette tokens for border, radius, icon-cell size, row height, spacing, typography, hover, and focus. Global creation entries and context-pad entries receive explicit accessible names and tooltips. Type chooser rows always include both a visual mark and visible label; creation tools use visible labels where space permits and otherwise retain a tooltip and accessible name.

### Treat node coordinates as command-owned state

Object, property, and mutation positions are authoritative user-authored geometry. Use the native `diagram-js` shape-move command as the sole owner of a drag transaction, and ensure relationship-maintenance behavior updates only connection waypoints and the derived intermediate control. No post-move listener may recreate, reimport, or recenter the moved node from stale business-object coordinates.

Serialization reads the final element bounds from the registry after the move command completes. Import converts persisted centers to top-left bounds using the correct size for each node kind. Undo and redo operate through the same command stack so both node coordinates and dependent edge/control geometry remain synchronized.

An alternative was to copy coordinates into the business object continuously during pointer movement. That duplicates framework state and can reintroduce stale-coordinate snap-back, so coordinates are captured at command completion and export boundaries instead.

### Reserve the intermediate control hit area

Place each controlled relationship's default label above the intermediate circle with a fixed visual gap. On import, preserve a manually positioned label only when its bounds do not overlap the control; normalize overlapping label positions to the above-control position. The label remains a native movable `diagram-js` label and its safe position remains persisted.

This preserves direct pointer access to the intermediate node and its wrench action. Raising the control above the label or relying on pointer-event pass-through was rejected because the visible label would still obscure the type icon and make the interaction ambiguous.

## Risks / Trade-offs

- [Four-character datatype marks can become dense at small zoom levels] → Use a minimum legible type size, test at representative zooms, and hide marks only below a documented extreme zoom threshold.
- [External labels can collide with edges or nearby nodes] → Use native label bounds and movement behavior, with sensible initial placement below compact nodes.
- [A large scalar list can make the chooser slow to scan] → Keep a deterministic, grouped order and derive future search behavior from the registry if the list grows.
- [Developers may retain stale local documents or generated assets] → Fail obsolete `filter` records with an actionable validation error, replace repository fixtures, and rebuild bundled assets in the same change.
- [Custom SVG marks may differ visually from BPMN icons] → Match the bundled BPMN stroke weight and optical size through shared CSS tokens and screenshot-level visual checks.
- [Relationship maintenance can accidentally overwrite a moved endpoint] → Restrict midpoint/reflow handlers to edges and intermediate controls and test object, property, and mutation moves independently.

## Migration Plan

1. Add the type registry and revise the development parser/serializer contract without a compatibility path.
2. Add command-backed type replacement and update node creation defaults.
3. Update rendering, context pads, global palette, and shared styles.
4. Correct move handling so node drops commit geometry while relationship controls and routes update without resetting endpoints.
5. Replace fixtures and update modeler, persistence, accessibility, movement, and undo/redo tests.
6. Rebuild bundled web assets and run the web and native integration suites.
7. Rollback is a source-level development rollback; generated test documents may be regenerated from the restored schema.

## Open Questions

- Confirm the complete scalar datatype catalog during implementation; it should be sourced from product requirements rather than guessed from a database vendor.
- Validate the exact mutation SVG metaphors in the running editor; suggested starting points are magnifier for query, `ƒ` for function, and opposing/flow arrows for transformation.
