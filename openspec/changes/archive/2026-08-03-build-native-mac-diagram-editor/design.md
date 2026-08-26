## Context

The repository does not yet contain an application. This change establishes a sandboxed native macOS diagram workspace whose chrome and file interactions are native while each diagram editor runs from bundled HTML, CSS, and JavaScript in `WKWebView`. BPMN has a current, actively maintained `bpmn-js` modeler; `archimate-js` provides the requested ArchiMate modeler and exchange XML support but is older and may require a compatible `diagram-js` dependency graph. The application must work offline after installation and must never load executable editor assets from a CDN.

## Goals / Non-Goals

**Goals:**

- Deliver a dark-mode native macOS shell with a resizable folder navigator and a tabbed editing area.
- Let a user authorize a folder, discover supported files, edit BPMN and ArchiMate diagrams, and safely save changes.
- Define a small, typed, testable message contract between Swift and the web editors.
- Bundle reproducible web assets while selecting the newest package versions that pass compatibility and smoke tests at implementation time.
- Keep format-specific editor behavior behind adapters so additional diagram types can be added later.

**Non-Goals:**

- Creating new diagrams, project metadata, cloud sync, collaboration, version control, or multi-window document management.
- Supporting formats other than BPMN XML and the XML format accepted by `archimate-js`.
- Reimplementing diagram rendering or modeling natively.
- Automatic conversion between BPMN and ArchiMate.
- Full semantic linting beyond errors and warnings exposed by the selected libraries.

## Decisions

### Use SwiftUI for application composition with AppKit integration points

The application will use a SwiftUI app lifecycle, `NavigationSplitView` or an equivalent split layout, native commands, and native tab state. `NSOpenPanel`, security-scoped bookmarks, close confirmation, and `WKWebView` integration will use AppKit/WebKit adapters where SwiftUI does not expose sufficient control. This preserves native behavior and accessibility while avoiding a wholly AppKit application. A pure AppKit shell was considered but rejected because it adds boilerplate without improving the core web editor integration.

### Model an authorized folder as a workspace, not as independent documents

A `WorkspaceStore` will own the selected root URL, restored security-scoped bookmark, recursive tree, open tabs, selected tab, and per-tab dirty/error state. File nodes use root-relative paths as stable display identifiers, while canonical URLs are validated to remain inside the authorized root before every read or write. Symlink traversal outside the root is excluded. This central model makes menu enablement and close/save prompts deterministic. Treating every file as an `NSDocument` was considered, but it does not naturally represent a folder-first navigator and shared folder authorization.

### Use one `WKWebView` editor session per open tab

Each tab retains its modeler instance and viewport state. A local `EditorHost` page receives an initial format and source payload, instantiates the matching adapter, and reports readiness, mutations, export results, warnings, and failures. Retaining one web view per tab costs more memory than recreating editors on selection, but avoids losing undo history and canvas position. The first version can impose a documented soft warning if many tabs are opened; virtualization is deferred.

### Define a versioned Swift–JavaScript bridge

Messages will be JSON envelopes with a contract version, request identifier, event/command name, diagram format, and typed payload. Swift sends `load`, `requestExport`, `undo`, `redo`, and appearance changes; JavaScript sends `ready`, `changed`, `exported`, and `failed`. Export requests are asynchronous and correlated by identifier. Untrusted diagram content is data passed to the modeler and never interpolated into page scripts. Navigation is limited to bundled resources, arbitrary popup/navigation requests are denied, and only named message handlers are exposed.

### Isolate diagram libraries behind format adapters

The web bundle will expose a common adapter interface (`load`, `export`, `destroy`, change notification) implemented by `BpmnAdapter` and `ArchimateAdapter`. BPMN uses the full `bpmn-js` Modeler and its XML import/save APIs. ArchiMate uses `archimate-js` import/export APIs after a compatibility spike determines the supported package graph. Format detection is extension-first and verified by import; a failed import produces an editor error without overwriting the source file.

### Resolve latest compatible dependencies reproducibly

Implementation will query the npm registry, install the latest stable `bpmn-js` and `archimate-js`, and explicitly add `diagram-js` only when required by the resolved peer/dependency graph. A compatibility smoke test must load, edit, and export fixtures for both formats. The passing exact versions will be committed in the lockfile; routine builds use `npm ci`. This interprets “latest” at implementation time while ensuring later builds do not drift. If current `diagram-js` is incompatible with `archimate-js`, the ArchiMate bundle may use its own pinned compatible transitive version while BPMN uses its current graph, isolated into separate chunks. Forcing one global version was rejected because it could make one editor unusable.

### Save atomically and make dirty-state transitions explicit

On Save, Swift requests exported XML from the active editor, writes it atomically to the original URL, and clears dirty state only after the write succeeds. Save All exports and writes dirty tabs independently and reports partial failures without clearing failed tabs. Closing a dirty tab, replacing the workspace, or quitting offers Save, Don’t Save, and Cancel. External file changes detected before saving cause a conflict prompt rather than silent overwrite.

### Build and test the web editor as an application resource

A web subproject will use npm and a lightweight bundler to produce hashed or manifest-addressed assets copied into the application resources by an Xcode build phase. Development builds fail if required assets are absent. JavaScript unit tests cover adapters and bridge serialization; Swift tests cover discovery, authorization boundaries, state transitions, and atomic persistence; UI/smoke tests exercise opening and saving one fixture of each format.

## Risks / Trade-offs

- [Risk] `archimate-js` is lightly maintained and may not run against the newest `diagram-js`. → Mitigation: perform the compatibility spike first, isolate dependency graphs, lock the passing versions, and record the compatibility result before building the full UI.
- [Risk] Third-party file extensions or XML dialects may be ambiguous. → Mitigation: centralize extension configuration, validate through the selected importer, preserve original bytes until a successful export, and show actionable import errors.
- [Risk] A web view per tab increases memory use. → Mitigation: measure representative diagrams, release views immediately on tab close, and add a soft open-tab warning if testing shows a practical limit.
- [Risk] Native/web dirty state can diverge when events arrive out of order. → Mitigation: use versioned correlated messages, gate commands on `ready`, serialize export requests per tab, and test race and failure paths.
- [Risk] Folder access can be lost across launches in the macOS sandbox. → Mitigation: persist a security-scoped bookmark, handle stale bookmarks, and request authorization again without discarding unrelated preferences.
- [Risk] Malformed or hostile XML is processed by JavaScript dependencies. → Mitigation: keep the editor offline, disable arbitrary navigation, cap readable file size, catch parser failures, and review package advisories during implementation.

## Migration Plan

This is a greenfield application, so no user-data migration is required. Implement the dependency compatibility spike and fixtures first, then the web host and bridge, then native workspace/file behavior, and finally packaging and end-to-end verification. Rollback consists of reverting the new application and web-project files; user diagram files remain standard source files and are written atomically, so uninstalling the app does not strand data.

## Open Questions

- Confirm the exact ArchiMate filename extensions and exchange-model variants accepted by the selected `archimate-js` release during the compatibility spike and document them in the file type registry.
- Determine the minimum supported macOS version based on the SwiftUI split-view and tab implementation selected during project scaffolding.
- Decide whether restored folder access should reopen previously open tabs in the first release; the baseline requirement restores folder authorization but may start with no tabs.
