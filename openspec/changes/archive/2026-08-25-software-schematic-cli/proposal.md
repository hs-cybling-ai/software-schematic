## Why

Software design artifacts need a lightweight, project-local home that developers and coding agents can open consistently without installing a desktop application or depending on a hosted service. A versioned Rust CLI and checked-in wrapper can provide the shortest path to drawing a BPMN flow, composing subprocesses, and documenting the model beside the code.

## What Changes

- Add a cross-platform Rust `ss` executable with an `init` command that installs a Software Schematic workspace into the current project.
- Have `ss init` create `.ss/` with the local runtime and bundled HTML, CSS, JavaScript, and BPMN editor assets; create `schematics/`; and place an `ssw` wrapper launcher at the project root.
- Have `ssw` start a loopback-only lightweight web server, serve the bundled application, and open it in the default browser.
- Add a sharp, professional two-column web application suitable for organization-wide daily use: tabbed BPMN editors on the left and selected-element metadata plus rendered/editable Markdown documentation on the right.
- Persist BPMN XML, metadata, and Markdown automatically as users edit them.
- Establish rigid diagram composition rooted in `schematics/main.bpmn`, with pools, lanes, and external subprocesses mapped to folders and each folder represented by `main.bpmn`.
- Open an external subprocess in a focused tab on double-click, creating its folder and base diagram when it does not yet exist.
- Keep this first increment deliberately narrow: initialize, open, compose, edit, document, and save valid project-local artifacts. MCP exposure, Graph DB indexing, recovery/compensation behavior, packaging/distribution automation, and broader modeling features are deferred.

## Capabilities

### New Capabilities

- `software-schematic-bootstrap`: Project-local installation, version metadata, generated workspace layout, wrapper launcher, local server startup, and browser opening.
- `software-schematic-editor`: Browser-based tabbed BPMN editing, selected-element metadata, Markdown rendering/source editing, and automatic persistence.
- `software-schematic-composition`: Rigid filesystem conventions and navigation/creation behavior for root diagrams, pools, lanes, and reusable external subprocesses.

### Modified Capabilities

None. Existing diagram capabilities describe a separate native macOS application and are not changed by this project-local CLI.

## Impact

- Adds a Rust CLI/server package and a browser application built with HTML, CSS, JavaScript, and selectively chosen UI or styling libraries where they materially improve the experience.
- Adds generated project conventions: `.ss/`, `schematics/`, and root-level `ssw` launchers.
- Introduces bundled browser dependencies for BPMN editing, Markdown rendering, Mermaid rendering, icons, and optional UI/styling support; every runtime dependency must be packaged in `.ss/` and the initialized tool must not require a package manager or network access.
- Defines local HTTP endpoints for workspace reads and writes, bound only to loopback and constrained to the initialized project.
- Targets Windows and macOS while keeping OS-specific behavior limited to wrapper launch and opening the default browser.
