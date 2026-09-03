## MODIFIED Requirements

### Requirement: Data-graph document persistence
The application SHALL load and export a current-version locally stored `.dgraph` ontology containing only `nodes`, `edges`, `properties`, and `contextStore`. Nodes SHALL use `objectNode`, `edgeNode`, or `mutationNode`; domain edges SHALL connect `objectNode` to `edgeNode`; range edges SHALL connect `edgeNode` to `objectNode`; modifier edges SHALL connect `mutationNode` to either `edgeNode` or `objectNode`; object-link edges SHALL connect distinct `objectNode` records and include a registered `linkType`; properties SHALL reference a valid node or edge owner; and `contextStore` SHALL identify a workspace-relative SQLite database, schema version, stable diagram ID, and authoritative captured revision. The application SHALL reject invalid or obsolete documents without modifying their source files and SHALL upgrade the immediately preceding valid topology-only version in memory by supplying a default sibling context-store reference without marking the document dirty.

#### Scenario: Valid multi-target document opens
- **WHEN** the user opens a current-version document containing a targetless or multi-target collection relationship
- **THEN** the editor reconstructs its complete typed editable topology and node context and leaves the tab clean

#### Scenario: Valid object-link document opens
- **WHEN** the user opens a current-version document containing `sameAs` or `subclassOf` object-link edges, including a cycle across multiple objects
- **THEN** the editor reconstructs the dashed typed links, accepts the cycle, restores node context, and leaves the tab clean

#### Scenario: Topology-only predecessor opens
- **WHEN** the user opens a valid document from the immediately preceding version containing only normalized nodes, edges, and properties
- **THEN** the editor upgrades it in memory with a default sibling SQLite context-store reference, preserves its topology, and leaves the tab clean

#### Scenario: Older obsolete shape is rejected
- **WHEN** the user opens an older Data Graph document using a replaced edge/control ownership shape
- **THEN** the editor reports that the obsolete development definition is invalid and leaves the source file unchanged

#### Scenario: Edited document exports
- **WHEN** the native layer requests export from a valid edited Data Graph session
- **THEN** the editor completes or explicitly fails pending context capture and returns a deterministic current-version document containing normalized nodes, typed stored edges including object links, owned properties, and the authoritative SQLite database reference, with no relationships, attachments, inferred topology aggregate, rendered HTML, or vector payloads in topology JSON

#### Scenario: Domain and range IDs are generated
- **WHEN** an edge node with ID `works_for` is connected from `person` and targets `company`
- **THEN** the exported domain edge is `works_for_domain` and the exported range edge is `works_for_range_company`

#### Scenario: Invalid relationship document opens
- **WHEN** a document has an intermediate relationship with a missing source, a non-object source or target, duplicate target IDs, more than one target for scalar, broken references, or an unknown subtype
- **THEN** the editor reports an actionable import failure, does not mark the tab dirty, and leaves the source file unchanged

#### Scenario: Invalid object-link document opens
- **WHEN** an object link has a missing or non-object endpoint, a self-reference, an unknown `linkType`, or duplicates an existing source-target-type tuple
- **THEN** the editor reports an actionable import failure, does not mark the tab dirty, and leaves the source file unchanged

#### Scenario: Invalid context store opens
- **WHEN** the context-store manifest escapes the authorized workspace, its referenced SQLite revision is missing, or its rows violate the current context schema or reference missing graph content
- **THEN** the editor reports an actionable import failure, does not mark the tab dirty, and leaves the source file unchanged

#### Scenario: Obsolete filter document is rejected
- **WHEN** the user opens a development Data Graph document containing an obsolete filter node or filter attachment
- **THEN** the editor reports that the obsolete graph definition is unsupported and leaves the source file unchanged
