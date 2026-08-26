## Context

The repository already specifies a native macOS diagram editor, but this change is a separate, project-local tool intended to travel with source code like a Gradle wrapper. Its first release must initialize a deterministic workspace, launch without a Node or package-manager dependency, and let a developer draw and document a composed BPMN model from a browser on Windows or macOS.

The implementation crosses four boundaries: a distributable Rust CLI, generated wrapper/runtime files, a loopback HTTP server with filesystem access, and a static browser editor. The browser cannot write project files directly, so all persistence and composition operations pass through the local Rust server.

## Goals / Non-Goals

**Goals:**

- Ship one native `ss` executable per supported OS/architecture and expose `ss init`.
- Generate a versioned, self-contained project tool that needs only a browser at runtime.
- Open, edit, compose, document, and automatically save BPMN and Markdown artifacts.
- Deliver a polished, cohesive interface suitable for routine use across a development organization.
- Keep the application self-contained by packaging all interaction, rendering, icon, and styling dependencies in `.ss/`.
- Use deterministic filesystem conventions that are readable to humans and coding agents.

**Non-Goals:**

- MCP tools, Graph DB indexing, LLM integration, semantic graph querying, or code generation.
- Recovery for deleted, malformed, inaccessible, conflicting, or externally modified files.
- General-purpose file browsing, arbitrary Markdown notebooks, non-BPMN diagram types, collaboration, authentication, or remote hosting.
- Installers, auto-update infrastructure, release signing, or a universal binary shared by Windows and macOS.
- Advanced BPMN validation or execution semantics.

## Decisions

### Use one Rust package for initialization and serving

`ss` will be compiled as a native executable for each supported target. The executable embeds the static application assets and starter BPMN/Markdown content. `ss init` copies a versioned runtime executable into `.ss/bin/`, writes `.ss/version`, materializes `.ss/web/`, creates `schematics/main.bpmn` and `schematics/main.md`, and writes root launchers (`ssw` for macOS and `ssw.cmd` for Windows). The launchers invoke the checked-in runtime with a `serve` command and the project root.

This mirrors wrapper-based build tools: projects pin the runtime and assets they were initialized with. Embedding a webview or requiring Node was rejected because both increase packaging and cross-platform complexity.

### Serve only a narrow project API on loopback

The runtime binds to `127.0.0.1` on an available port, serves `.ss/web/`, and exposes only typed operations needed by the UI: list diagrams, read a BPMN or Markdown file, write a BPMN or Markdown file, and resolve/create a composed diagram. Every requested path is normalized and must remain under `schematics/`. The server then opens the generated URL with the OS default-browser mechanism.

A general static file-write endpoint was rejected because it unnecessarily expands the write surface. Remote binding was rejected because this is a single-user local tool with no authentication requirement.

### Keep BPMN authoritative and derive documentation paths

Each composition folder has `main.bpmn` and diagram documentation in `main.md`. Element documentation uses `docs/<bpmn-element-id>.md` beside that folder's base diagram. The selected element's ID, name/label, BPMN type, and derived documentation path form the compact metadata header. ID and label edits update the BPMN model; changing an ID also renames its existing Markdown file in the same save operation.

This avoids a metadata database or sidecar manifest in the first release. Type is model-derived and read-only, while documentation links are deterministic and therefore do not need duplicated storage.

### Represent reusable external subprocesses as BPMN call activities

The editor treats a `bpmn:CallActivity` as the external-subprocess composition primitive. Its standard `calledElement` value stores a normalized project-relative composition folder beneath `schematics/`. Double-click resolves `<calledElement>/main.bpmn`; if the folder is absent, the server creates the folder plus starter `main.bpmn` and `main.md`, then the UI opens or focuses its canonical tab.

Pool folders resolve as `<parent>/<pool-id>/`, and lane folders as `<parent>/<pool-id>/<lane-id>/`; each contains a base `main.bpmn` and `main.md`. Stable BPMN IDs are used rather than labels so renaming display text does not silently move paths. A custom BPMN namespace and opaque internal graph IDs were rejected for this increment.

### Use retained tabs and debounced automatic saves

The browser retains one `bpmn-js` modeler session per canonical BPMN path. Selection drives the right column. Model changes are serialized with `bpmn-js` and sent after a short debounce; Markdown source edits are saved after a short debounce and on leaving edit mode. The server writes through a temporary sibling file followed by replacement, and the UI reports pending, saved, or failed state.

The tool intentionally has no manual-save workflow or compensation logic. A failed save remains visibly failed, but the first release does not reconstruct missing files, merge external changes, or retry indefinitely.

### Bundle browser libraries and optimize for product-quality interaction

The initialized assets include pinned browser distributions of `bpmn-js`, a Markdown renderer, Mermaid, an icon set, and any selected UI or styling library. A small UI library is acceptable when it materially improves accessibility, interaction quality, maintainability, or visual consistency. All production assets are built ahead of time, copied beneath `.ss/web/vendor/`, and served locally; no CDN, package installation, or network fetch is allowed at runtime.

The interface will use an explicit design-token layer for color, typography, spacing, radii, elevation, focus treatment, and motion. It will emphasize the modeling canvas, keep metadata dense without feeling cramped, use clear selected/hover/focus states, and make autosave status perceptible without becoming visually noisy. Generic unstyled controls and visually inconsistent library defaults are not acceptable. A framework-free shell remains a viable implementation choice, but is no longer a design constraint.

## Risks / Trade-offs

- [Native executables differ by OS and architecture despite the “single executable” distribution model] → Publish one self-contained `ss` binary per supported target and record its version in initialized projects.
- [A browser UI requires a local HTTP process and can leave it running] → Tie server lifetime to the wrapper process and handle normal termination signals.
- [Autosave can issue stale writes during rapid editing] → Serialize saves per file, debounce mutations, and attach a monotonically increasing client revision so only the latest acknowledged revision clears pending state.
- [Renaming element IDs changes documentation paths] → Perform the BPMN update and documentation rename through one typed server operation; reject collisions rather than overwrite.
- [BPMN call activities do not enforce valid filesystem references] → Constrain `calledElement` editing to normalized relative composition paths in the UI and server.
- [No missing-file or external-edit recovery can produce hard failures] → Surface the exact failing path and operation; defer repair workflows as explicitly out of scope.
- [Checking runtime binaries and vendor assets into projects increases repository size] → Favor small release builds and minified pinned assets; accept the cost for reproducibility and zero package-manager setup.
- [Visual polish can expand the first increment beyond its functional goal] → Establish a compact token system and a small set of reusable primitives, then require visual QA of the core workspace states instead of building a broad component library.

## Migration Plan

This is a new capability with no data migration. Implementation will first produce a development `ss` binary, then validate `ss init` and `ssw` in disposable projects on macOS and Windows. Re-running `ss init` over an existing `.ss/`, `schematics/`, or wrapper is not part of this proposal; initialization must stop on collisions. Rollback consists of removing the generated `.ss/`, `schematics/`, and wrapper files from a project that has not adopted their contents.

## Open Questions

- Which exact Rust HTTP and browser-opening crates should be selected after checking binary size and supported target matrices?
- Which pinned Markdown, Mermaid, icon, and optional UI/styling libraries provide the best balance of visual quality, accessibility, and bundle size?
- Should Windows ARM64 be included in the first release matrix or follow the initial macOS ARM64/x64 and Windows x64 targets?
