## Context

See `proposal.md` for the user-visible failure. Diagram import already reads each eligible element's `ssw:implementationStatus` into a per-tab `nodeStatuses` map. The CSS colors are driven by `node-status-*` canvas markers, but `applyNodeStatus` is currently reached only through inspector updates and status edits. Initial import therefore hydrates status data without hydrating its visual or accessible presentation. Several reload, rename, conflict-resolution, and revert paths also call `importXML` directly and can leave both the markers and the per-tab cache out of sync with the newly imported registry.

The solution must work through the shared diagram adapter so BPMN and CMMN receive identical lifecycle behavior, retain the existing persisted XML format, and avoid creating command-stack mutations or autosaves merely by rendering imported status.

## Goals / Non-Goals

**Goals:**

- Establish status markers and accessible labels for all eligible elements immediately after every successful import.
- Rebuild the per-tab status cache and presentation from the current element registry as one operation.
- Preserve immediate rendering when a user or assistant changes a status.
- Keep the behavior diagram-type-neutral and independently testable.

**Non-Goals:**

- Changing status names, colors, meanings, eligibility, or XML metadata.
- Replacing the existing marker-based CSS implementation.
- Changing selection styling, modeler command semantics, persistence timing, or graph behavior.

## Decisions

### Centralize post-import status hydration and rendering

Introduce one tab-scoped routine that clears `nodeStatuses`, scans the adapter's current element registry, normalizes every eligible element's persisted status, stores non-default statuses in the map, and applies the corresponding marker and accessible label. Call it after initial tab construction and after every successful XML reimport.

This treats import as the lifecycle boundary at which the model, cache, canvas, and accessibility metadata must become consistent. It also prevents individual reload workflows from duplicating only part of the restoration logic.

Alternative considered: trigger inspector updates for each element. That would incorrectly couple whole-diagram rendering to selection state and repeatedly perform documentation I/O.

### Keep single-element rendering as the shared primitive

Retain a single-element status renderer for interactive changes, and have the whole-tab hydration routine invoke the same primitive. The renderer will derive labels through the supplied tab's adapter rather than global `activeTab`, so inactive tabs and reimported tabs receive correct accessible descriptions.

Alternative considered: write fill colors directly to SVG during import. That would bypass the existing marker classes, duplicate the status palette, and make interaction-state preservation harder.

### Make all XML import paths use the same post-import contract

Audit initial open, conflict reload, process/package rename, assistant revert, and any other direct `importXML` call. Each successful import must be followed by cache and presentation hydration before the workflow returns control to the user. A small import wrapper may enforce this for existing tabs; initial tab creation can call the hydration routine after the tab object exists.

Alternative considered: listen only for the modeler's generic import-complete event. Explicit orchestration is preferred because the tab object and adapter context are required, and current imports are already awaited at clear application boundaries.

### Test observable lifecycle behavior with a focused modeler harness

Add Vitest coverage using a small fake registry/canvas/adapter or an extracted dependency-injected helper. Tests will verify that all eligible BPMN- and CMMN-shaped records receive normalized markers and accessible labels immediately, that unsupported elements are ignored, that rerendering removes stale marker state, and that all application import paths invoke post-import hydration. Existing status round-trip tests continue to cover XML persistence.

Alternative considered: rely only on source-text contract assertions. Those assertions can confirm wiring but cannot prove that multiple elements receive the correct initial marker, so behavioral helper coverage is required.

## Risks / Trade-offs

- [A missed direct `importXML` path can still leave stale colors] → Route all known imports through the shared post-import routine and add a contract test enumerating the expected wiring.
- [Marker application across large diagrams adds work during import] → Perform one linear registry scan and avoid commands, layout, or persistence; marker updates are proportional to visible elements.
- [Reimport replaces registry element instances] → Always rescan the current registry and clear the cache rather than retaining old element references or IDs without validation.
- [Accessible labels could use the wrong diagram adapter for inactive tabs] → Pass the tab explicitly through label and marker rendering instead of consulting global active state.

## Migration Plan

No data migration is required. Ship the web bundle with the lifecycle fix and tests; existing BPMN and CMMN files will display their current metadata correctly on the next open. Rollback consists of reverting the web code and bundled assets, with no persisted-data conversion needed.
