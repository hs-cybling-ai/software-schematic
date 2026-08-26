## 1. Rust CLI Foundation

- [x] 1.1 Create the Rust package and command parser with `ss init` and the internal `serve` command
- [x] 1.2 Select, pin, and embed starter BPMN/Markdown plus all HTML, CSS, JavaScript, font, icon, renderer, editor, and optional UI/styling assets needed for offline use
- [x] 1.3 Implement `ss init` to validate collisions and generate `.ss/`, `.ss/version`, `schematics/main.bpmn`, `schematics/main.md`, `ssw`, and `ssw.cmd`
- [x] 1.4 Add initialization tests covering the complete layout, recorded version, executable wrapper permissions, and collision refusal

## 2. Local Server and File Boundary

- [x] 2.1 Implement loopback available-port binding, static application serving, normal signal shutdown, and default-browser opening
- [x] 2.2 Implement confined path normalization that rejects absolute paths, traversal, symlink escapes, and paths outside `schematics/`
- [x] 2.3 Implement typed endpoints for diagram listing and BPMN/Markdown reads and atomic writes
- [x] 2.4 Implement the typed element-ID/documentation rename operation with collision refusal
- [x] 2.5 Add server tests for endpoint behavior, complete-content replacement, path confinement, and rename collisions

## 3. Browser Workspace and BPMN Editing

- [x] 3.1 Define design tokens and reusable UI primitives for typography, color, spacing, icons, controls, elevation, focus, motion, and application states
- [x] 3.2 Build the sharp responsive two-column shell with tab strip, dominant diagram area, compact metadata header, documentation area, and unobtrusive save-state indicator
- [x] 3.3 Apply consistent hover, focus, selection, disabled, pending, and failure treatments across keyboard and pointer interaction
- [x] 3.4 Integrate bundled `bpmn-js` and implement one retained modeler session per canonical BPMN path
- [x] 3.5 Load and focus `schematics/main.bpmn` on startup and implement tab opening, switching, and canonical-path deduplication
- [x] 3.6 Bind BPMN selection to ID, label, type, and derived documentation path and implement valid ID and label updates
- [x] 3.7 Implement serialized, revision-aware, debounced BPMN autosave with pending, saved, and failed states
- [x] 3.8 Perform visual QA at representative supported desktop viewport sizes and correct overlap, clipping, hierarchy, density, and inconsistent styling defects

## 4. Markdown Documentation

- [x] 4.1 Load diagram Markdown from `main.md` and element Markdown from `docs/<element-id>.md`
- [x] 4.2 Integrate the bundled Markdown and Mermaid renderers for the default documentation view
- [x] 4.3 Implement the edit-icon toggle between rendered Markdown and source editing
- [x] 4.4 Implement serialized, revision-aware, debounced Markdown autosave and save-on-leaving-edit-mode behavior
- [x] 4.5 Add browser tests for documentation target selection, rendering/edit toggling, and autosave state transitions

## 5. Composition

- [x] 5.1 Implement normalized call-activity `calledElement` path editing and validation
- [x] 5.2 Implement the server operation that resolves or creates a composition folder with starter `main.bpmn` and `main.md`
- [x] 5.3 Handle call-activity double-click by resolving its external subprocess and opening or focusing its canonical diagram tab
- [x] 5.4 Implement pool and lane folder derivation from stable BPMN IDs and the action that resolves, creates, and focuses their base diagrams
- [x] 5.5 Add integration tests for new subprocess creation, existing subprocess navigation, shared-process tab reuse, and pool/lane paths

## 6. Cross-Platform Acceptance

- [x] 6.1 Verify the generated `ssw` launcher initializes the pinned runtime and opens an editable root flow on supported macOS targets
- [ ] 6.2 Verify the generated `ssw.cmd` launcher initializes the pinned runtime and opens an editable root flow on supported Windows targets
- [x] 6.3 Run an end-to-end acceptance flow that initializes a disposable project, edits and autosaves BPMN and Markdown, creates an external subprocess, and reopens the saved artifacts
- [x] 6.4 Verify the complete application loads and remains fully styled with network access disabled and no external asset requests
- [x] 6.5 Document the supported target matrix, CLI usage, generated layout, composition conventions, packaged web dependencies, and explicitly deferred MCP/Graph DB scope
