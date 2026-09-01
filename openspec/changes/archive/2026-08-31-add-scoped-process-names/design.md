## Context

Software Schematic is a project-local documentation wrapper. The naming model should be visible and literal, without hidden scope derivation or duplicate identity fields.

## Decisions

### One Name

Every named diagram element stores one complete architectural `Name` in BPMN extension metadata. There is no separate local Name or derived Qualified Name field.

- Process: `package.SubProcess`
- Node or edge: `package.SubProcess#nodeOrEdgeName`

ID remains the BPMN instance identifier, Type remains read-only BPMN type, and Label remains unrestricted display text.

This also keeps graph construction direct: ID identifies a node or edge occurrence in a diagram, while Name identifies the reusable architectural concept it represents. Graph consumers do not need to reconstruct qualified identity from folders, labels, or parent elements.

### Direct folder mapping

The process portion before `#` maps to nested directories. `cybling.subscription.SelectAndOutfit` maps to `schematics/cybling/subscription/SelectAndOutfit/`. Equal process Names open the same `main.bpmn` and `main.md`.

### ID-bound documentation

Process documentation is `main.md`. Node and edge documentation remains bound to the stable BPMN ID at `docs/<element-id>.md`. There is no editable or read-only Documentation-path metadata field; Documentation is the Markdown editor itself. Changing Name does not move documentation; changing ID renames its documentation file.

This deliberately separates call-site documentation from implementation documentation. Two call activities may share one process Name and therefore one subprocess `main.md`, while each caller retains its own `docs/<activity-id>.md` in the diagram that contains it.

### Straightforward rename

Changing a process Name moves its process folder and updates local references. Changing a node or edge Name changes only that stored Name. No additional migration, compensation, or integration system is part of this change.

### Assistant contract

Assistant context contains ID, Type, Label, Name, Implementation Status, and Documentation. Providers preserve these values unless an explicit operation changes one and never choose filesystem paths.

## Non-Goals

- A separate package or scope field.
- A separate Qualified Name field.
- A Documentation path field.
- Implementation bindings in structured metadata.
- A generalized migration, transaction, or integration framework.
