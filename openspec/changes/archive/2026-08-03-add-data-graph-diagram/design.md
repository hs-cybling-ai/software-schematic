## Context

Diagram Studio is a sandboxed macOS application whose Swift workspace shell discovers local diagram files and hosts format-specific editors in a bundled web application. BPMN establishes the target interaction model plus the adapter, bridge, tab-lifecycle, dirty-state, and safe-save patterns. ArchiMate support is removed because it is nonfunctional and its legacy toolkit conflicts with the product's focus on Data Graph editing.

The first phase must make graph schemas quick to sketch: object and literal-property circles, labeled directed edges, and an edge-owned midpoint control on object relationships that can carry properties and filters. Literal properties connect directly to their object because they describe the object rather than the relationship. The relationship midpoint is also the planned attachment point for thick green-outline mutation nodes in the next phase, so its identity and typed attachment interface must be explicit in the model even though its screen position is derived.

## Goals / Non-Goals

**Goals:**

- Deliver a recognizable, editable property-graph notation with object nodes, pastel-blue property nodes, and labeled directed edges.
- Make creation flow from the current selection through a context palette, comparable in speed to BPMN append/connect interactions.
- Give qualifying edges a stable intermediate control that owns supported edge attachments.
- Integrate data-graph documents with current tab, bridge, workspace, save, diagnostics, undo, and test patterns.
- Define a versioned document contract suitable for human tooling and later AI generation.

**Non-Goals:**

- Creating, defining, or executing graph mutations, queries, or runtime transformations; the future mutation node's thick green-outline visual contract is reserved but not rendered in this phase.
- Mapping diagrams directly to a particular graph database or query language.
- Automatic layout, schema inference, collaborative editing, or code generation.
- Replacing BPMN notation or changing its editor behavior.

## Decisions

### Build the editor as a dedicated `diagram-js` modeler

Compose the Data Graph editor from the same current `diagram-js` generation used by BPMN, with custom element factory, renderer, rules, modeling handlers, palette/context-pad providers, direct editing, import/export validation, and diagnostics. This is required to inherit BPMN-quality canvas interaction, selection, dragging, connections, keyboard behavior, zooming, panning, and command-stack semantics.

Alternative considered: a repository-owned SVG modeler. It proved able to render the notation but did not provide the mature, intuitive BPMN-like interaction behavior that is central to the product goal.

### Use a versioned JSON document with semantic and presentation sections

Use a canonical extension selected during implementation (provisionally `.dgraph.json`) and a top-level format identifier plus integer schema version. Store semantic records for object nodes, property nodes, filters, relationships, and attachment ownership; store node positions and edge waypoints as presentation data. Use stable string IDs and explicit references.

Alternative considered: GraphML or an existing property-graph exchange format. Those formats do not naturally encode this notation's edge-owned controls, filter attachments, and editor geometry without vendor extensions. A small JSON contract is easier for both local tooling and AI to produce and validate.

### Model the intermediate circle explicitly but derive its geometry

Every visible item is modeled as a `diagram-js` shape or connection. Each object-to-object logical relationship owns one real, half-size intermediate-control shape and is rendered as two docked connection segments: source-to-control and control-to-target. Object-to-property literal links are single direct connections with no control. The control uses the same selection, context-pad, layout, and command infrastructure as other nodes while retaining its distinct double-circle rendering and serialized edge ownership.

Each relationship label is a native `diagram-js` label shape linked to its owning connection or control. It is centered on the logical edge by default, participates in selection and move behavior, supports direct editing, and persists an explicit position after movement.

The control is constrained by relationship behavior and recenters when endpoints move, preventing it from becoming an unrelated free-floating node.

### Use radial collision-aware append placement and outline docking

Context-pad append actions evaluate candidate positions around the selected node at a consistent radius and select the first location that does not overlap an existing shape. A custom `diagram-js` layouter docks every segment on the circular outlines of its source and target rather than using center-to-center endpoints.

### Represent edge properties and filters as typed attachments

Property and filter nodes attached through the intermediate control have explicit `ownerEdgeId`/attachment semantics, not ordinary graph-edge endpoints. The attachment type is extensible so the next phase can add mutation nodes—rendered as thick green-outline circles—without changing edge-control ownership. Deleting the owning relationship cascades through the control and attachments in one command-stack transaction. Connection rules prevent invalid cross-ownership and dangling references.

Alternative considered: model all visible connectors uniformly as graph relationships. This simplifies rendering, but loses the distinction between domain relationships and metadata or constraints applied to those relationships.

### Make the context palette selection-sensitive

Object and property nodes expose valid append/connect actions; intermediate controls expose edge-property and filter actions. Actions are hidden or disabled through modeling rules when invalid. New elements receive default labels and immediately enter direct editing so a graph can be built with short select-and-append sequences.

Alternative considered: a persistent global palette only. It is familiar, but it adds pointer travel and mode switching that conflicts with the speed goal. A minimal global entry point may remain for the first node and general tools.

### Reuse the existing native bridge and workspace lifecycle

Add a data-graph case to format detection, tab routing, starter-document creation, bridge load/export messages, and supported-file discovery. The web adapter reports changes only for command-stack model mutations and returns deterministic JSON on export. Native atomic-write and conflict behavior remain format-neutral.

Alternative considered: manage files entirely in JavaScript. This would duplicate security-scoped access, conflict detection, and safe persistence already owned by the Swift shell.

## Risks / Trade-offs

- [The provisional file extension may conflict with existing conventions] → Confirm the canonical extension during implementation before shipping fixtures, then keep format detection centralized.
- [Labels may not fit inside BPMN-sized circular nodes] → Apply bounded wrapping or ellipsis in the circle and retain full text for editing and accessibility.
- [Path-midpoint placement can jump when routing changes] → Calculate against actual polyline length and cover straight and multi-segment routes with geometry tests.
- [Custom `diagram-js` modules can accumulate notation-specific complexity] → Keep semantic rules and persistence separate from framework services, and test providers, commands, and rendering independently.
- [Cascade deletion could remove attachments unexpectedly] → Treat the cascade as a single undoable command and make ownership visually unambiguous.
- [AI-produced JSON may be structurally plausible but invalid] → Validate version, types, IDs, references, ownership, and required controls before import; never partially mutate a live session.

## Migration Plan

1. Add the format model, parser/validator, starter document, and Swift workspace recognition behind the new extension.
2. Add the web modeler and bridge adapter, then bundle rebuilt web assets.
3. Add unit, integration, compatibility, and representative fixture coverage before exposing the format in the creation UI.
4. Existing workspaces require no data migration; newly recognized files appear on the next workspace refresh.
5. Rollback removes the new creation option and format routing. Existing data-graph files remain untouched on disk and become undiscoverable by older builds.

## Open Questions

- Confirm the user-facing name and canonical extension; the design currently uses “Data Graph” and `.dgraph.json` provisionally.
- Decide whether filter attachments use the same circular silhouette as nodes with a distinct style, or a different basic shape while preserving the circle-first visual language.
- Decide the exact pastel-blue token and object-node fill token within the existing editor theme and accessibility contrast constraints.
