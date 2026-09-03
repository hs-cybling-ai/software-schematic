# Scoped process naming

Software Schematic keeps three values separate: **ID** identifies one BPMN node or edge occurrence, architectural **Name** identifies the reusable semantic concept, and **Label** is unrestricted presentation text. That separation also makes graph construction direct: graph occurrences use ID and reusable relationships use Name.

There is one stored Name field. A process Name is `package.Process`; a node or edge Name is `package.Process#nodeOrEdgeName`.

Qualified process Names determine composition folders. For example, `cybling.subscription.SelectAndOutfit` owns `schematics/cybling/subscription/SelectAndOutfit/`, including `main.bpmn`, `main.md`, `docs/`, and local assets. Two nodes with the same qualified process Name open this same composition regardless of their BPMN IDs or Labels. Members do not create composition folders.

Changing a task or event Name updates only its member symbol. Renaming a child process renames its composition folder and updates local qualified references. The root `schematics/main.cmmn` remains the project navigation diagram; `main.bpmn` is retained only as a legacy fallback when no CMMN anchor exists.

Process documentation is `main.md` in the process folder. Node and edge documentation is bound to the BPMN ID at `docs/<element-id>.md`. Two call activities can therefore keep distinct call-site documentation while both opening the same named subprocess and its shared `main.md`. A Name change never renames element documentation; an ID change does.

Implementation bindings—classes, methods, endpoints, and source files—remain ordinary Markdown content rather than architectural Name metadata. `calledElement` may remain in serialized BPMN as a derived copy of the qualified process Name, but it is not independently editable.

The inspector exposes exactly ID, Type, Label, Name, Implementation Status, and the Documentation editor. It does not expose separate Qualified Name, Documentation path, or External process controls.

Opening a call activity or subprocess by double-click or by its subprocess link icon requires a process Name. If no Name exists, the editor opens a modal for a valid `package.Process` value. It creates no fallback Name, folder, or diagram when the dialog is cancelled or the value is invalid.
