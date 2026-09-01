## Why

SSW needs one stable architectural Name for every diagram element. Labels are presentation text and BPMN IDs are instance identifiers; neither should determine reusable process folders or documentation filenames.

## What Changes

- Add one editable `Name` field alongside ID, Type, Label, Implementation Status, and Documentation.
- Store the complete qualified Name directly: `package.SubProcess` for a process and `package.SubProcess#nodeOrEdgeName` for a node or edge.
- Remove separate local Name, Qualified Name, External process, and Documentation-path fields.
- Map a process Name directly to its folder and let equal process Names reuse the same `main.bpmn` and `main.md`.
- Keep element documentation bound to BPMN ID at `docs/<element-id>.md`.
- Rename a process folder when its process Name changes; Name changes do not rename element documentation.
- Include the single Name value in assistant context and Name-based operations.

## Capabilities

### New Capabilities

- `scoped-process-naming`: One qualified architectural Name per process, node, or edge.

### Modified Capabilities

- `software-schematic-composition`: Name-derived process folders and ID-bound element documentation files.
- `software-schematic-editor`: The compact ID, Type, Label, Name, Implementation Status, and Documentation experience.
- `ai-diagram-assistance`: Preserve and operate on the single qualified Name.

## Impact

- Affects BPMN extension metadata, composition lookup, documentation filenames, the metadata inspector, graph snapshots, and assistant operations.
