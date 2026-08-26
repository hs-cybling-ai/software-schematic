## 1. Closable Tab Structure

- [x] 1.1 Refactor SSW tab markup into a non-interactive container with separate tab-selection and accessible `x` close buttons for non-root diagrams while preserving canonical-path deduplication and horizontal scrolling.
- [x] 1.2 Identify and protect normalized root `schematics/main.bpmn` in both rendering and close logic, while keeping nested composition paths ending in `main.bpmn` closable.
- [x] 1.3 Implement permitted-tab lifecycle cleanup for tab state, DOM, event bindings, and the `bpmn-js` modeler, without activating an inactive tab as a side effect.
- [x] 1.4 Flush queued automatic BPMN and Markdown persistence before cleanup and implement next-tab/previous-tab selection, returning to retained root `main.bpmn` after the final auxiliary tab closes.

## 2. Node Status Model and Controls

- [x] 2.1 Add a centralized registry for `new`, `locked`, `modify`, and `open`, including colors, LLM meanings, validation, and `open` fallback behavior.
- [x] 2.2 Add per-open-tab, element-ID-keyed client-side status storage with eligible BPMN node filtering and cleanup when a tab closes.
- [x] 2.3 Add a keyboard-accessible status selector and textual LLM-hint explanation to the selected-item metadata panel, updating it correctly as selection and active tabs change.
- [x] 2.4 Wire status changes to update UI state immediately without mutating BPMN business objects, blocking manual edits, or scheduling BPMN/Markdown persistence.

## 3. Project Identity

- [x] 3.1 Add a read-only Rust project-metadata route that returns only the canonical project root's non-empty directory basename.
- [x] 3.2 Load project metadata during web startup and set `document.title` to `Software Schematic - <project name>`, retaining `Software Schematic` as the unavailable/empty fallback.

## 4. Fullscreen Control

- [x] 4.1 Replace the maximize button's fit-only handler with feature-detected `requestFullscreen`/`exitFullscreen` behavior for the SSW app shell and actionable error reporting.
- [x] 4.2 Synchronize maximize/minimize icons, titles, accessible labels, and pressed state from `fullscreenchange`, including fullscreen exits initiated through Escape or browser chrome.
- [x] 4.3 Resize and fit the active BPMN canvas after fullscreen entry and exit layout settles.

## 5. Markdown Mode Toggle

- [x] 5.1 Show a pencil icon and `Edit Markdown` accessible label in rendered/read mode, then switch to a book icon and `Read Markdown` label when source editing opens.
- [x] 5.2 Make the book action flush/save current Markdown, render the updated content, return to read mode, and restore the pencil icon and label.

## 6. Node Status Presentation

- [x] 6.1 Add centralized CSS variables and rendering hooks that apply green, grey, orange, and white primary node fills for `new`, `locked`, `modify`, and `open` while preserving outlines, icons, labels, and selection/hover cues.
- [x] 6.2 Add status names and LLM meanings to accessible node descriptions so status is never communicated by color alone.

## 7. Verification

- [x] 7.1 Add server and browser tests for project basename metadata, absolute-path omission, dynamic titles, spaces/Unicode, and fallback behavior.
- [x] 7.2 Add tests for closing inactive, active, and final auxiliary tabs, root `schematics/main.bpmn` protection, nested `main.bpmn` closability, neighbor selection, queued-save flushing, canonical-path reopening, and modeler/DOM cleanup.
- [x] 7.3 Add tests for fullscreen entry, control-driven and external exit, actual-state synchronization, rejection handling, and post-transition diagram fitting.
- [x] 7.4 Add tests for both Markdown toggle transitions, icon and accessible-label synchronization, saving before read mode, and rendering the updated source.
- [x] 7.5 Add tests for status defaults, eligibility, all four transitions and colors, per-tab isolation, non-persistence, manual edit availability under `locked`, and accessible text.
- [x] 7.6 Run the SSW web and Rust test suites and production asset build, then manually verify the project title, keyboard tab closing, fullscreen entry/exit, Markdown mode toggling, and node-status presentation for representative BPMN node types.
