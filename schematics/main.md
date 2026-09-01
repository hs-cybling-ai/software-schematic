# Main flow

This is the project entry diagram (`designPro.Main`). It begins the reusable `cybling.Birth` process and then displays the home page.

## Identity and documentation

- BPMN IDs identify occurrences in this diagram and bind their documentation to `docs/<element-id>.md`.
- Names identify graph concepts and reusable processes. Nodes and edges use `designPro.Main#memberName`; the reusable subprocess uses `cybling.Birth`.
- The `Birth` call site can have caller-specific documentation here, while the shared implementation is documented in `cybling/Birth/main.md`.
