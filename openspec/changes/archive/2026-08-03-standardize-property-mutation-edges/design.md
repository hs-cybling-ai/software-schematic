## Context

The Data Graph renderer currently assigns one class and arrow marker to both logical relationship segments and attachment connections. Relationship creation also creates an edge-label shape unconditionally, including when the target is a property. Because the renderer does not classify a connection by its semantic target, property and mutation connections cannot maintain consistent notation across direct node connections, control attachments, creation, and import.

The persisted document already identifies node types and attachment ownership. The change therefore belongs in the projection from the semantic model to diagram-js elements and SVG, without requiring a document-version migration.

## Goals / Non-Goals

**Goals:**

- Derive property and mutation edge notation solely from the target node type, independent of source type.
- Render property-targeting connections dashed and without an edge-label shape.
- Keep the property semantic label below its node as the visible relationship description.
- Render mutation-targeting connections solid with a filled black circular target marker.
- Apply the rules identically during interactive creation and document import.
- Keep unaffected relationship rendering and persisted graph semantics stable.

**Non-Goals:**

- Changing property, mutation, or object node geometry and subtype marks.
- Changing which source/target combinations the authoring rules permit.
- Removing edge label fields from the persisted document schema or incrementing its version.
- Reinterpreting edges based on source type, ownership, or creation path.

## Decisions

### Classify rendered connections by target node type

Introduce a small shared edge-notation classifier that inspects the semantic type of the connection target and returns `property`, `mutation`, or `relationship`. Use it in SVG rendering, marker selection, and label creation/import decisions. Target-based classification directly implements the invariant “regardless of source” and avoids duplicating source-target matrices across code paths.

Alternative considered: assign styles when each connection is created. This was rejected because imported connections and future creation paths could omit or disagree on presentation metadata.

### Express line and endpoint notation in SVG/CSS

Give property-targeting connections a dedicated class with a dash pattern and no target marker. Give mutation-targeting connections a solid-line class and a dedicated SVG marker containing a filled black circle. Leave ordinary relationship connections on the existing solid line and arrow marker.

Alternative considered: draw endpoint circles as independent diagram shapes. This was rejected because SVG markers naturally track rerouting, orientation, zoom, and connection endpoints without adding model elements.

### Omit property-edge label shapes at projection boundaries

Interactive creation and import SHALL not create an edge-label diagram element when the logical target is a property. The property node's existing compact label remains below the node and remains directly editable. Serialization may retain the relationship label value already present in the document for backward compatibility, but no label position is emitted for a nonexistent property-edge label.

Alternative considered: create the label and hide it with CSS. This was rejected because an invisible shape could still participate in selection, direct editing, layout, and serialization.

### Keep persistence backward compatible

Do not alter the current JSON schema in this change. Existing required edge label values can remain as compatibility data even when the visual projection suppresses the property-edge label. Mutation edges continue to use the ordinary relationship label behavior unless otherwise constrained by their ownership model.

Alternative considered: make property-edge labels optional and migrate stored documents. That creates a format change disproportionate to a notation cleanup and can be proposed separately if semantic redundancy becomes undesirable.

## Risks / Trade-offs

- [Hidden legacy property-edge labels remain in JSON and could surprise downstream consumers] → Document that the field is retained for compatibility and test that save/reopen does not recreate a visible label.
- [Marker geometry can overlap the mutation outline or stop short of the node] → Tune marker `refX`, radius, and docking with focused rendering assertions and visual inspection.
- [diagram-js connection targets may be controls in segmented relationships] → Classify the actual rendered segment target; attachment connections terminate at property or mutation nodes, while ordinary relationship segments retain their existing notation.
- [A property used as a source will not make its outgoing edge dashed] → This is intentional: notation communicates the dependent target, exactly matching the target-driven rule.

## Migration Plan

Ship the renderer and projection changes together with regression tests. Existing documents require no data migration; opening them applies the new notation immediately. Rollback restores the former projection without changing stored content.

## Open Questions

None. The requested rules define target classification, line style, marker, and property-label placement explicitly.
