## Context

The Data Graph model normalizes collection relationships into an `objectNode → edgeNode → objectNode` topology. The connection rule currently allows an object-to-object gesture to fall through as a direct diagram-js connection, so the resulting element lacks the required model semantics and is omitted or rejected by normalized persistence. Separately, authors need to express semantic reuse between object types without drawing duplicate relationship loops—for example, a `coworker` object should be able to reuse the modeled connections of `Person`.

This change affects the Data Graph modeler, renderer, document validator/serializer, fixtures, and tests. The stored document remains a normalized `nodes`, `edges`, and `properties` document, but gains a typed direct edge kind.

## Goals / Non-Goals

**Goals:**

- Make the default relationship gesture create a scalar `edgeNode` at its terminal point when no object is reached, or place it between the source and target and create domain/range edges atomically when an object is reached.
- Add a visually and structurally distinct dashed object-link edge with a registered semantic type.
- Let graph consumers resolve the linked target's existing relationships without copying stored nodes or edges.
- Keep resolution finite and deterministic when semantic links form cycles or diamonds.
- Preserve object links through validation, import/export, editing, deletion, undo, and redo.

**Non-Goals:**

- Materializing inherited relationships into the persisted document.
- Performing general OWL/RDFS reasoning or enforcing ontology consistency.
- Migrating unsupported development document shapes.
- Allowing semantic object links to target properties, mutation nodes, or edge nodes.

## Decisions

### Store semantic links as typed direct edges

Add an `objectLink` edge record whose source and target must both reference `objectNode` records and whose `linkType` is validated against the Data Graph type registry. The initial registry entries are `sameAs` and `subclassOf`; the registry owns display labels and future resolution metadata so new types do not require scattered conditionals.

This is preferable to inserting an `edgeNode`, because an object link describes identity/type reuse rather than a domain relationship. It is also preferable to copying the target's relationships, which would create divergent duplicates and make cyclic models unmanageable.

### Separate the two object-to-object authoring intents

An object exposes exactly two connection choices: `Relationship` and `Object link`. Relationship completion on empty canvas creates a scalar `edgeNode` at the gesture's terminal point, connected to the source and ready to receive a later target. Relationship completion on another object invokes one compound command that places the `edgeNode` at the midpoint between the two objects and creates the complete relationship. Its domain and range records are internal persisted segments, not separate authoring edge types. Object link creates a direct dashed `objectLink`, defaulting to `sameAs`, after which the existing anchored type chooser pattern changes its modifier (`linkType`).

Keeping the authoring surface to these two choices avoids inferring semantics from geometry and prevents internal domain/range segments from appearing as extra connection types. The compound relationship command ensures undo never leaves a partial topology.

### Render links as dashed direct connections

`objectLink` uses outline-docked endpoints and a dashed stroke, with an accessible label derived from `linkType`. It has no intermediate node and no collection relationship label. The selected link exposes a wrench/type action plus normal delete behavior. Dependent-property connections remain dashed but are distinguishable by endpoint types and accessible names.

### Resolve inherited relationships as a cycle-safe graph view

The persisted graph remains unchanged. A resolver computes an object's effective connections from its own collection relationships plus relationships reachable by following applicable object links from source to target. Traversal maintains a visited object-ID set and deduplicates results by the underlying relationship/edge-node ID, producing stable ordering by stored order and then ID. `sameAs` and `subclassOf` initially both allow the source object to assume the target object's connections; the registry can refine direction or behavior for later link types.

This virtual view supports loops such as `coworker sameAs Person` without recursively cloning `Person` or repeatedly returning the same relationship. Keeping the resolver separate from serialization also makes raw topology and inferred topology unambiguous.

### Validate links independently from collection topology

Import rejects object links with missing endpoints, non-object endpoints, unknown `linkType` values, self-links, or duplicate `(source, target, linkType)` tuples. Cycles across two or more valid links are accepted because resolution is explicitly cycle-safe. Export includes only stored object links, never inferred relationships.

## Risks / Trade-offs

- [Risk] Dashed object links may be confused with dependent-property edges → Use different endpoint eligibility, accessible labels, selection actions, and link-type metadata; add a modest dash-pattern distinction if visual testing shows ambiguity.
- [Risk] The meaning of future ontology link types may exceed simple target-to-source inheritance → Centralize link behavior in the type registry and constrain this change to the documented initial resolution rule.
- [Risk] Direct links can form dense cyclic graphs → Use visited-set traversal and edge-ID deduplication, and test cycles and diamond-shaped inheritance.
- [Risk] Correcting generic object-to-object gestures changes current accidental behavior → Treat the prior direct edge as invalid/unpersisted behavior and cover the new atomic result with interaction and serialization tests.

## Migration Plan

No compatibility migration is provided for invalid direct object-to-object development edges because they were not part of the persisted schema. Add the new edge kind to the current validator and fixtures, deploy the web assets with the native bundle, and retain rejection of all other unknown edge kinds. Rollback consists of reverting the modeler/document changes; documents containing `objectLink` edges will then fail validation rather than being silently altered.

## Open Questions

- Whether a later ontology-focused change should give `sameAs` symmetric resolution rather than the initial explicit source-to-target inheritance rule.
