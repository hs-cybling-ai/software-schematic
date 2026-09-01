## 1. Simplify the Name model

- [x] 1.1 Replace local/derived naming with one stored qualified Name for processes, nodes, and edges.
- [x] 1.2 Validate `package.Process` and `package.Process#nodeOrEdgeName` forms.
- [x] 1.3 Map the process portion of a Name directly to a reusable composition folder.

## 2. Simplify metadata and documentation

- [x] 2.1 Show only ID, Type, Label, Name, and Implementation Status as metadata controls.
- [x] 2.2 Keep Documentation as the Markdown editor and remove Qualified Name and Documentation-path controls.
- [x] 2.3 Keep process documentation at `main.md` and element documentation bound to `docs/<element-id>.md`.
- [x] 2.4 Rename process folders for process Name changes and documentation files for ID changes only.

## 3. Align assistant and verification

- [x] 3.1 Send ID, Type, Label, Name, Implementation Status, and Documentation in assistant context.
- [x] 3.2 Update Name-based assistant operations to accept the single qualified Name.
- [x] 3.3 Test process reuse, node and edge Names, ID-bound documentation, and Name renames.
- [x] 3.4 Run web and Rust tests, production build, and strict OpenSpec validation.
- [x] 3.5 Prompt for a valid process Name before opening an unnamed subprocess from either navigation gesture.
