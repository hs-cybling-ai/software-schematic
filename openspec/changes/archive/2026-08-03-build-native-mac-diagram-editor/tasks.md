## 1. Project and Dependency Baseline

- [x] 1.1 Scaffold the sandboxed SwiftUI macOS application, native unit/UI test targets, and web-editor subproject, and record the chosen minimum macOS version.
- [x] 1.2 Add representative valid and invalid BPMN and ArchiMate fixtures with documented expected filename extensions and exchange XML variants.
- [x] 1.3 Query npm for current stable `bpmn-js`, `archimate-js`, and required `diagram-js` ecosystem versions; implement import/edit/export compatibility spikes for both fixtures.
- [x] 1.4 Resolve any ArchiMate dependency conflict with isolated bundles or a compatible transitive version, commit exact passing versions and the npm lockfile, and document the compatibility matrix.
- [x] 1.5 Configure repeatable web lint, unit test, bundle, and `npm ci` commands plus an Xcode build phase that embeds only local production assets and fails when they are missing.

## 2. Web Editor Host and Adapters

- [x] 2.1 Implement the local dark-themed editor host page and common format-adapter interface for load, export, destroy, and mutation notifications.
- [x] 2.2 Implement `BpmnAdapter` with the full `bpmn-js` modeler, bundled styles/assets, XML import/export, command-stack dirty notifications, warnings, and correlated errors.
- [x] 2.3 Implement `ArchimateAdapter` with `archimate-js`, bundled styles/fonts/assets, supported XML import/export, mutation notifications, warnings, and correlated errors.
- [x] 2.4 Implement and document the versioned JSON bridge envelopes for ready, load, changed, export, appearance, and failure messages with request correlation.
- [x] 2.5 Lock down the editor host to bundled navigation and named message handlers, prevent diagram data from being interpolated into executable JavaScript, and reject unsupported formats or bridge versions.
- [x] 2.6 Add JavaScript tests for bridge serialization, readiness ordering, both adapters, invalid imports, export failures, mutation tracking, and the BPMN/ArchiMate compatibility smoke fixtures.

## 3. Native Workspace and File Model

- [x] 3.1 Implement the supported-format registry with centralized extensions, file-size limits, format detection, and adapter selection.
- [x] 3.2 Implement `WorkspaceStore` state for the authorized root, recursive file nodes, canonical open-tab identity, selected tab, retained editor sessions, dirty state, diagnostics, and source file metadata.
- [x] 3.3 Implement `NSOpenPanel` folder selection, security-scoped bookmark persistence/restoration, stale-bookmark recovery, and balanced access lifetime management.
- [x] 3.4 Implement recursive discovery and refresh with folders-first localized sorting, unsupported-file filtering, selection preservation, and exclusion of symlinks resolving outside the authorized root.
- [x] 3.5 Implement safe file reads with root-boundary validation, duplicate-tab prevention, file-size enforcement, and non-destructive import error handling.
- [x] 3.6 Add Swift tests for authorization restoration, path containment, symlink escape rejection, mixed-content discovery, sorting, refresh behavior, format detection, and canonical tab identity.

## 4. Native Application Shell

- [x] 4.1 Build the system dark-mode split window with a resizable native outline/sidebar, accessible labels, minimum pane widths, and no-folder/no-selection empty states.
- [x] 4.2 Add File menu commands and shortcuts for Open Folder, Save, Save All, Close Tab, and relevant enablement derived from workspace state.
- [x] 4.3 Build the tab strip and editor area with filename/dirty indicators, selection, individual close actions, retained per-tab `WKWebView` sessions, and immediate cleanup on close.
- [x] 4.4 Implement the `WKWebView` representable/coordinator and typed native bridge client, including ready gating, serialized per-tab export requests, appearance updates, warnings, and isolated tab errors.
- [x] 4.5 Deny arbitrary web navigation, new windows, network editor assets, and unintended JavaScript-to-native surface area through WebKit delegates and configuration.
- [x] 4.6 Add native view-model and UI tests for launch/empty states, split resizing, menu actions, multi-tab selection, retained editor state, diagnostics, and web navigation restrictions.

## 5. Save, Conflict, and Lifecycle Safety

- [x] 5.1 Implement Save by requesting the active adapter's XML, validating the destination remains in the workspace, atomically replacing the source, and clearing dirty state only on success.
- [x] 5.2 Implement Save All with independent per-tab results and an aggregate report that preserves dirty state for every failed export or write.
- [x] 5.3 Detect source changes using captured file metadata and present explicit overwrite/reload/cancel conflict handling before a save can replace externally changed content.
- [x] 5.4 Add Save, Don’t Save, and Cancel flows for closing dirty tabs, switching workspace folders, and application termination, proceeding only after all requested saves succeed.
- [x] 5.5 Add Swift tests for atomic-write failure, external conflicts, partial Save All failure, dirty-state transitions, each close-prompt decision, and unchanged original bytes after import/export failure.

## 6. Integration and Release Verification

- [x] 6.1 Add end-to-end tests that open a folder, discover nested files, edit and save one BPMN fixture, edit and save one ArchiMate fixture, reopen both, and verify the persisted changes.
- [x] 6.2 Verify malformed/oversized files, missing authorization, stale bookmarks, externally renamed files, dependency warnings, and editor failures produce actionable non-destructive UI states.
- [x] 6.3 Profile representative multi-tab sessions, verify web views and security-scoped resources are released on close, and add a soft tab-count warning if measured memory use requires it.
- [x] 6.4 Run native and web test suites from a clean checkout, build the signed sandboxed application with network access unavailable, and confirm all editor assets load from the app bundle.
- [x] 6.5 Document developer setup, dependency-update and compatibility-test procedure, supported diagram extensions/variants, keyboard commands, known ArchiMate constraints, and manual acceptance steps.
