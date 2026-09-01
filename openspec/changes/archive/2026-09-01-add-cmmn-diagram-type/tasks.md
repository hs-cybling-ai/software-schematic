## 1. CMMN Assets and Compatibility

- [x] 1.1 Select and pin a `cmmn-js` version by testing it with the current Vite browser build and existing `bpmn-js` dependency, documenting any required vendor-bundle isolation.
- [x] 1.2 Add valid, invalid, and round-trip CMMN fixtures containing a Case Plan Model, Stage, Case Task, Process Task, Sentry/criteria connections, stable IDs, layout, and SSW metadata.
- [x] 1.3 Add a minimal bundled starter CMMN document suitable for a named package composition.
- [x] 1.4 Add an SSW CMMN moddle descriptor for package, member, and Process Task target Names, with fixture tests proving import/export preservation.

## 2. Shared Browser Diagram Adapters

- [x] 2.1 Introduce a typed diagram-adapter contract for import/export, modeler lifecycle, selection, registry/modeling access, metadata updates, composition targets, connections, and normalized assistant context.
- [x] 2.2 Move existing BPMN-specific calls behind a BPMN adapter and verify the current BPMN browser tests and real BPMN round-trip fixtures remain unchanged.
- [x] 2.3 Add a lazily loaded CMMN adapter backed by `cmmn-js`, including fatal-error handling, warning reporting, save serialization, viewport fitting, undo/redo where supported, and resource cleanup.
- [x] 2.4 Route `.bpmn` and `.cmmn` paths to the correct adapter while keeping tabs, save queues, breadcrumbs, and active-selection state keyed by complete canonical relative path.

## 3. CMMN Metadata and Documentation Parity

- [x] 3.1 Implement CMMN selection and the shared ID, Type, Label, Name, Implementation Status, and Documentation inspector behavior for supported shapes and connections.
- [x] 3.2 Implement CMMN package, member, and Process Task target Name validation using the specified complete-Name grammars.
- [x] 3.3 Reuse `main.md` for CMMN diagram Documentation and `docs/<element-id>.md` for CMMN element Documentation, including safe ID rename, automatic save, rendered/edit modes, and error state.
- [x] 3.4 Add UI and unit tests covering CMMN node and connection selection, metadata edits, status display, Markdown ownership, undo/redo, automatic save, tab switching, and tab closure.
- [x] 3.5 Keep CMMN Process Task Labels bidirectionally synchronized between the inspector and rendered node, show a blank Label when nothing is selected, and retain double-click child navigation.
- [x] 3.6 Allow short node and edge Names to inherit the parent CMMN package or BPMN process/package while retaining fully qualified Names as explicit overrides.

## 4. Mixed CMMN and BPMN Composition

- [x] 4.1 Extend shared path helpers so CMMN package Name `cybling.sdk` derives `cybling/sdk/main.cmmn` while existing BPMN Name `cybling.sdk.Birth` continues to derive `cybling/sdk/Birth/main.bpmn`.
- [x] 4.2 Add the lightweight action that creates or opens a `main.cmmn` business anchor for a named neutral package folder without requiring CMMN for its child or parent packages.
- [x] 4.3 Add Process Task composition navigation to create or reuse a named BPMN process through the existing BPMN composition path and starter.
- [x] 4.4 Retain the originating CMMN tab and provide direct return navigation when a BPMN design opens from its Process Task business anchor.
- [x] 4.5 Extend the confined generic server composition and rename requests with an allow-listed document kind, strict kind-specific Name validation, collision detection, and `.cmmn` read/write support without adding CMMN parsing to Rust or changing existing BPMN folder creation.
- [x] 4.6 Add Rust and browser integration tests for CMMN business anchors, BPMN-only nested packages, a higher-level CMMN linking to descendant-package BPMN, mixed `main.cmmn`/`main.bpmn` tabs, traversal rejection, collisions, and package rename behavior.

## 5. CMMN Magic Actions

- [x] 5.1 Produce bounded normalized CMMN diagram and element snapshots containing ID, Type, Label, Name, status, Documentation, owning package, supported connections, and no absolute project path.
- [x] 5.2 Extend the assistant schema and validation with the bounded CMMN operations for label/Name updates, Markdown replacement, supported plan-item creation and connection, and Process Task BPMN linkage.
- [x] 5.3 Apply approved CMMN mutations through supported `cmmn-js` modeling services and integrate them with the existing preview, approval, automatic save, rollback, and assistant-level revert flow.
- [x] 5.4 Add assistant tests for CMMN node and diagram scopes, valid composition links, unsupported operations, raw XML and path rejection, locked elements, failed mixed-file rollback, and successful revert.

## 6. CLI Initialization and Verification

- [x] 6.1 Update the web build to compile introduced TypeScript sources and bundle all CMMN JavaScript, CSS, icons/fonts, moddle metadata, and lazy chunks as local static assets.
- [x] 6.2 Embed and extract the CMMN-capable static browser bundle and starter asset through `ss init`.
- [x] 6.3 Verify a clean initialized project can create, edit, document, save, close, reopen, and rename `cybling/main.cmmn`, then use its Process Task to open `cybling/sdk/Birth/main.bpmn` without requiring `cybling/sdk/main.cmmn`, entirely offline.
- [x] 6.4 Run Rust and browser test suites, dependency audit, production asset build, and independent inspection confirming the generated `.ss/` bundle contains no CDN or runtime package-manager dependency.
- [x] 6.5 Confirm no property-graph, embedding, retrieval, execution, compensation, deployment, permission, tenancy, or collaboration code is introduced by the implementation.
- [x] 6.6 Make `schematics/main.cmmn` the sole initialized and retained project anchor, retain `main.bpmn` only as a legacy fallback, and reject competing root anchors.
- [x] 6.7 Migrate the Cybling Wallet acceptance project so its root CMMN defines domain, actors, inputs, outputs, needs, and business services while the former root BPMN exists only as a linked named process.
