## Context

SSW is a self-contained browser editor in `software-schematic-web`. It creates one `bpmn-js` modeler and one tab button per canonical BPMN path, retains modelers while switching tabs, and automatically persists BPMN and Markdown changes. Tabs currently have no close control. The selected-item panel edits BPMN identity and documentation, but there is no LLM-oriented node status.

The requested status is forward-looking metadata for GrafeoDB. Because that storage integration does not yet exist, this change must define one stable vocabulary and UI behavior without inventing a temporary server persistence format that could conflict with the eventual database schema.

## Goals / Non-Goals

**Goals:**

- Give every non-root SSW tab a clear, keyboard-accessible `x` close control while keeping the project's root `main.bpmn` tab permanently open.
- Make the Markdown mode toggle visibly distinguish reading from source editing and announce the action it will perform.
- Make the existing maximize control enter and exit browser fullscreen reliably and keep the diagram fitted after viewport changes.
- Identify the active project in the browser title without exposing its filesystem path.
- Release a closed tab's modeler and DOM resources and choose a predictable remaining active tab.
- Let users set `new`, `locked`, `modify`, or `open` on selectable BPMN nodes.
- Make the four LLM hints visible through both color and textual/accessibility cues.
- Centralize the status vocabulary and semantic mapping so future GrafeoDB integration has one UI contract.

**Non-Goals:**

- Persisting node status to GrafeoDB, BPMN XML, Markdown, or a new local endpoint.
- Giving statuses to BPMN connections, labels, or diagram tabs.
- Making `locked` enforce editor permissions; it is guidance for a future LLM, not an edit lock in this change.
- Changing automatic-save behavior or adding dirty-tab confirmation UI.

## Decisions

### Treat status as client-side node metadata with `open` as the default

Define a single registry for the four exact values and meanings: `new` means the node represents work/content to create, `locked` means an LLM must not change it, `modify` means an LLM is allowed and expected to change it, and `open` supplies no hint. Status is held against the active modeler's stable BPMN element ID for the lifetime of that open tab. Missing or invalid values normalize to `open`.

The registry is deliberately isolated behind getter/setter functions so a later GrafeoDB adapter can replace the in-memory backing store without changing controls or rendering. Embedding the provisional value in BPMN extension elements was rejected because the user identified GrafeoDB as the eventual source of truth and no compatible schema exists yet.

### Put the status control in selected-node metadata

Add a labeled four-value selector to the existing selected-item metadata area and show it only for eligible BPMN shape nodes. Selecting a connection, label, diagram root, or nothing hides or disables it. A change updates the client-side status map and rerenders the selected element immediately; it does not schedule BPMN or Markdown autosave.

This reuses the established inspector instead of adding a floating palette, keeping the canvas clear and making the semantic labels available to keyboard and assistive-technology users.

### Apply status through a dedicated rendering class and accessible text

Decorate eligible node graphics with a status-specific class/data attribute, using green for `new`, grey for `locked`, orange for `modify`, and white for `open`. Styling targets the node's primary shape fill while retaining existing outlines, icons, labels, selection markers, and BPMN type cues. The inspector always displays the status name and its LLM meaning, so color is redundant rather than the only signal.

The colors will use centralized CSS variables and contrast-aware foreground/border treatment. Rewriting BPMN business-object type or using generic SVG inline styles was rejected because status is orthogonal to BPMN semantics and renderer classes are easier to update or remove.

### Split closable tabs into selection and close controls

Render a tab container with a tab-selection button and, for non-root diagrams, a sibling `x` button. This avoids invalid nested interactive controls and gives each action a distinct accessible name and focus target. Identify the non-removable navigation anchor by its normalized project-root path, `schematics/main.bpmn`, rather than by filename alone so a nested composition's own `main.bpmn` remains closable. The root tab exposes no actionable close control and cannot be removed through the close function even if that function is invoked programmatically.

Closing a non-root tab stops pending per-tab work safely, destroys the modeler, removes its canvas and tab DOM, and deletes it from the canonical-path map.

If the closed tab is active, activate the nearest remaining tab, preferring the next tab and otherwise the previous one. Because root `main.bpmn` remains open, a project with no auxiliary tabs returns to that navigation tab rather than an empty state. Closing an inactive tab does not disturb the active editor. Event propagation from the `x` control is contained so closing never activates the tab first.

### Make the Markdown mode icon represent the next action

The Markdown toggle shows a pencil while rendered/read mode is active because its action is to enter source editing. After the user activates it, the source editor is displayed and the control changes to a book with an accessible label such as `Read Markdown`. Activating the book flushes/saves the current source, restores rendered/read mode, and changes the control back to the pencil with an `Edit Markdown` label.

Representing the next action, rather than merely the current state, matches conventional button behavior and keeps the icon, tooltip, and accessible label aligned. A pressed-state-only treatment was rejected because it would not make the return-to-reading action as immediately recognizable.

### Use the browser Fullscreen API for the top-bar control

Replace the current maximize-button handler, which only calls `canvas.zoom('fit-viewport')`, with a fullscreen toggle for the SSW app shell. When no element is fullscreen, the button requests fullscreen on `#app`; when the document is fullscreen, it calls `document.exitFullscreen()`. A `fullscreenchange` listener is the source of truth for the button icon, title, accessible label, and pressed state because fullscreen can also end through Escape or browser chrome.

Entering fullscreen shows a minimize/exit-fullscreen icon labeled `Exit full screen`; leaving it restores the maximize icon labeled `Enter full screen`. After each fullscreen transition, the active modeler receives a resize notification and fits its canvas after layout settles. Rejected fullscreen promises and `fullscreenerror` events surface an actionable toast. Keeping fit-to-viewport as the maximize action was rejected because it does not match the icon or user expectation.

### Expose the project basename as read-only startup metadata

Add a confined read-only metadata response from the Rust server containing only the final directory name of its already-canonicalized project root. During application startup, the browser uses that value to set `document.title` to `Software Schematic - <project name>`. The static HTML retains `Software Schematic` as the initial and failure fallback title.

The server, rather than browser URL parsing, is authoritative because the randomly assigned localhost URL contains no project identity. Returning only the basename avoids exposing an unnecessary absolute filesystem path. Empty, root-only, or otherwise unavailable basenames fall back to `Software Schematic` rather than producing a dangling dash.

## Risks / Trade-offs

- [Statuses disappear when a tab or browser session closes] → Label the feature as an authoring hint and keep the storage interface isolated for direct GrafeoDB integration; do not imply that this release persists it.
- [Fill overrides could obscure BPMN type styling] → Change only the primary node fill, preserve stroke and markers, and add representative visual tests for common node types and selected/hovered states.
- [`locked` may be mistaken for enforced permissions] → Describe it in the UI and tests as an LLM hint, while leaving manual editing behavior unchanged.
- [Closing during an automatic save could discard unsent work] → Flush queued per-tab BPMN/Markdown work and await or retain the existing serialized write before destroying UI resources.
- [Tab controls can become cramped] → Keep close buttons compact, preserve horizontal scrolling, and expose them on every closable tab without relying on hover.
- [A nested composition also uses the filename `main.bpmn`] → Protect only the normalized project-root path and test that nested `main.bpmn` tabs remain closable.
- [A changing icon could be ambiguous without text] → Keep the icon synchronized with an explicit tooltip and accessible action label in both modes.
- [Fullscreen may be denied by browser policy or unavailable] → Feature-detect the API, disable or clearly report unsupported use, catch request failures, and derive state only from `fullscreenElement`.
- [Canvas dimensions can be stale during a fullscreen transition] → Resize and fit the active modeler after the `fullscreenchange` layout has settled.
- [Project directory names can contain spaces or punctuation] → Assign the name through `document.title` as text and test representative Unicode names; never interpolate it as HTML.

## Migration Plan

1. Refactor tab markup and lifecycle cleanup while preserving canonical-path deduplication, automatic persistence, and the non-removable root `main.bpmn` invariant.
2. Add the centralized node-status registry, in-memory per-tab state, inspector control, and eligibility rules.
3. Add renderer decoration and accessible status explanations.
4. Add read-only project metadata and set the project-aware browser title during startup.
5. Replace the fit-only maximize handler with the fullscreen state machine and post-transition canvas resize/refit behavior.
6. Update the Markdown mode toggle and add unit/UI coverage for its icon, label, save, and mode transitions.
7. Add unit/UI coverage for project naming, tab closing, fullscreen behavior, cleanup, status transitions, colors, defaults, and non-color cues.

Rollback removes the new controls and in-memory status registry; no stored data migration is required.

## Open Questions

- Which GrafeoDB node property/schema and API will become authoritative for these values?
- Should a future persisted `locked` status also prevent manual edits, or remain exclusively an LLM instruction?
