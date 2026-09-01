# Software Schematic Editor

## Purpose

Provide a polished, self-contained browser workspace for editing, documenting, and automatically saving CMMN business anchors and BPMN compositions.

## Requirements

### Requirement: Two-column schematic workspace
The web application SHALL present a polished, responsive diagram column on the left and a metadata/documentation column on the right, with the diagram column occupying the larger share of the available workspace.

#### Scenario: Application opens
- **WHEN** the initialized application loads successfully
- **THEN** it displays the project's sole root anchor in the left editor and the current diagram documentation in the right column

#### Scenario: Workspace is resized
- **WHEN** the browser viewport changes within the supported desktop range
- **THEN** both columns remain usable without overlapping controls, clipped primary actions, or unreadable documentation

### Requirement: Organization-grade visual system
The web application SHALL use a cohesive visual system for typography, color, spacing, icons, controls, elevation, focus, and interaction states, and SHALL present a production-quality interface appropriate for daily use across a development organization.

#### Scenario: User navigates the primary workspace
- **WHEN** the user views tabs, canvas controls, metadata fields, documentation actions, and save status
- **THEN** the elements use consistent hierarchy, alignment, density, and interaction treatment

#### Scenario: Interactive control changes state
- **WHEN** a control is hovered, focused, selected, disabled, pending, or failed
- **THEN** its state is visually distinct and does not rely on color alone where that would obscure meaning

### Requirement: Self-contained web assets
The initialized application SHALL load all browser code, styles, fonts, icons, and third-party libraries from packaged files beneath `.ss/` and SHALL make no runtime dependency request to a CDN or package registry.

#### Scenario: Project is opened without internet access
- **WHEN** the wrapper launches in an offline environment
- **THEN** the complete styled interface and all editing and rendering libraries load from the initialized project

### Requirement: Tabbed diagram editing
The diagram column SHALL provide one retained, selectable editor tab per complete canonical `.cmmn` or `.bpmn` path and SHALL use the corresponding bundled modeler to render and edit each diagram. The project's normalized sole root anchor tab SHALL remain open as the navigation entry point and SHALL NOT expose an actionable close control or be removable through the tab-close operation. Every other diagram tab SHALL display a keyboard-accessible `x` close control. Closing a permitted tab SHALL remove that retained editor session and release its UI/modeler resources without changing other open sessions; when an active auxiliary tab closes, the application SHALL select the nearest remaining tab, preferring the next tab and otherwise the previous tab. The application SHALL safely flush queued automatic persistence for the closing tab before releasing it.

#### Scenario: Multiple diagrams are opened
- **WHEN** the user opens supported diagram files with different canonical paths
- **THEN** each file has one retained tab, switching tabs preserves each editor session, and every tab except the sole root anchor has a visible `x` close control

#### Scenario: An open diagram is requested again
- **WHEN** the application requests a BPMN path that already has a tab
- **THEN** it focuses the existing tab without creating a duplicate

#### Scenario: Inactive tab is closed
- **WHEN** the user activates the `x` on an inactive tab
- **THEN** the application closes and releases that tab without activating it or changing the current tab

#### Scenario: Active tab is closed
- **WHEN** the user activates the `x` on the active tab while other tabs remain
- **THEN** the application flushes its queued persistence, releases its editor, and activates the next tab or the previous tab when no next tab exists

#### Scenario: Root main diagram is displayed
- **WHEN** the project root anchor tab is open
- **THEN** it has no actionable `x` control and the tab-close operation refuses to remove its editor session

#### Scenario: Last auxiliary tab is closed
- **WHEN** the user closes the only auxiliary tab
- **THEN** the application releases that editor session and returns to the retained root anchor tab

#### Scenario: Nested main diagram is displayed
- **WHEN** a composition opens a nested path such as `schematics/orders/main.bpmn`
- **THEN** that tab has an actionable `x` control because protection is based on the normalized root path rather than the filename

### Requirement: Compact selected-item metadata
The inspector SHALL expose exactly ID, Type, Label, Name, Implementation Status, and Documentation for a selected diagram element. ID, Label, Name, Implementation Status, and Documentation SHALL be editable when applicable; Type SHALL be read-only. Documentation SHALL be the Markdown editing surface, not a path field. The editor SHALL NOT expose Qualified Name, Documentation path, or External process fields.

#### Scenario: User selects a BPMN node or edge
- **WHEN** a BPMN node or edge is selected in the active editor
- **THEN** the inspector shows ID, Type, Label, Name, Implementation Status, and its Documentation editor

#### Scenario: User edits Name
- **WHEN** the user enters a valid short parent-scoped Name or fully qualified override
- **THEN** the authored Name is stored and its resolved identity determines the associated composition while Documentation remains ID-bound

#### Scenario: User updates an element label
- **WHEN** the user changes the selected element's label in the metadata header
- **THEN** the active BPMN model updates its name and schedules the diagram for automatic save

#### Scenario: User updates an element ID
- **WHEN** the user submits a valid unused BPMN ID
- **THEN** the model adopts the ID and any existing documentation file is renamed from the old ID to the new ID without overwriting another file

### Requirement: Name required before subprocess navigation
When a user opens a call activity, subprocess, or CMMN Process Task by double-clicking it or selecting its composition link and the element has no Name, the editor SHALL show a modal dialog that collects a valid short process Name or fully qualified `package.Process` override. A short Name SHALL inherit the parent diagram package. The editor SHALL NOT generate a fallback Name or create a composition until the user submits a valid Name.

#### Scenario: Unnamed subprocess is opened
- **WHEN** the user double-clicks an unnamed subprocess or selects its subprocess link icon
- **THEN** a Name dialog opens and the composition remains unopened

#### Scenario: Valid Name is submitted
- **WHEN** the user submits `sales.checkout.PlaceOrder`
- **THEN** the exact Name is stored on the element and the shared subprocess composition opens

#### Scenario: Name dialog is cancelled
- **WHEN** the user cancels the Name dialog
- **THEN** the subprocess remains unnamed and no composition is created

### Requirement: Markdown display and source editing
The remainder of the right column SHALL render the selected item's Markdown by default and SHALL provide an edit icon that switches between rendered content and Markdown source.

#### Scenario: Documentation is viewed
- **WHEN** a diagram or supported BPMN or CMMN element becomes the current documentation target
- **THEN** its Markdown is rendered, including Mermaid fenced blocks rendered as diagrams

#### Scenario: User enters documentation edit mode
- **WHEN** the user activates the edit icon
- **THEN** the rendered view is replaced by an editable Markdown source control containing the same content

#### Scenario: User leaves documentation edit mode
- **WHEN** the user activates the edit icon while source is visible
- **THEN** the current source is saved and the column returns to rendered Markdown

### Requirement: Project-aware browser title
The local server SHALL expose the initialized project's directory basename as read-only project metadata without exposing its absolute filesystem path. After loading that metadata, the web application SHALL set the browser document title to `Software Schematic - <project name>`. If a non-empty project name cannot be resolved or loaded, the title SHALL remain `Software Schematic` without a trailing separator.

#### Scenario: Named project opens
- **WHEN** SSW is served for a project rooted at `/work/order-service`
- **THEN** the browser title becomes `Software Schematic - order-service`

#### Scenario: Project name contains spaces or Unicode
- **WHEN** the initialized project directory is named `Café Platform`
- **THEN** the browser title displays `Software Schematic - Café Platform` as plain text

#### Scenario: Project metadata is unavailable
- **WHEN** project metadata cannot provide a non-empty directory basename
- **THEN** the browser title remains `Software Schematic`

### Requirement: Fullscreen workspace control
The SSW top bar SHALL provide a fullscreen control that uses the browser Fullscreen API to enter fullscreen for the application shell and exit the document's fullscreen state. The control SHALL derive its state from the document's actual fullscreen element, SHALL show a maximize icon labeled `Enter full screen` when windowed, and SHALL show a minimize icon labeled `Exit full screen` while fullscreen. Following each fullscreen transition, the application SHALL resize and fit the active diagram canvas to its changed viewport. Unsupported or rejected fullscreen operations SHALL leave the current state intact and report an actionable failure.

#### Scenario: User enters fullscreen
- **WHEN** the application is windowed and the user activates `Enter full screen`
- **THEN** the application shell requests browser fullscreen and, after success, the control shows `Exit full screen` and the active diagram is resized and fitted

#### Scenario: User exits fullscreen with the control
- **WHEN** the application is fullscreen and the user activates `Exit full screen`
- **THEN** the document exits fullscreen, the control restores `Enter full screen`, and the active diagram is resized and fitted

#### Scenario: User exits fullscreen externally
- **WHEN** fullscreen ends through Escape or browser chrome
- **THEN** the fullscreen-change event restores the maximize icon, `Enter full screen` label, and unpressed state

#### Scenario: Fullscreen request fails
- **WHEN** the browser does not support or rejects the requested fullscreen operation
- **THEN** the application remains in its actual current state and displays an actionable error without disrupting the active diagram

### Requirement: Markdown read and edit mode control
The Markdown panel SHALL show rendered Markdown in read mode and editable Markdown source in edit mode. Its mode button SHALL represent the action it will perform: a pencil icon with an accessible `Edit Markdown` label in read mode and a book icon with an accessible `Read Markdown` label in edit mode. Activating the pencil SHALL enter edit mode; activating the book SHALL save the current source, render it, return to read mode, and restore the pencil icon.

#### Scenario: User enters Markdown edit mode
- **WHEN** the Markdown panel is in rendered read mode and the user activates the pencil button
- **THEN** the panel displays editable Markdown source and changes the button to a book labeled `Read Markdown`

#### Scenario: User returns to Markdown read mode
- **WHEN** the Markdown panel is in source edit mode and the user activates the book button
- **THEN** the application saves the current source, displays its rendered Markdown, and changes the button to a pencil labeled `Edit Markdown`

#### Scenario: Mode control is interpreted without its icon
- **WHEN** a user navigates the Markdown mode control with assistive technology or cannot identify the icon
- **THEN** the control announces `Edit Markdown` in read mode or `Read Markdown` in edit mode

### Requirement: Implementation Status
The SSW editor SHALL let a user assign exactly one Implementation Status of `new`, `locked`, `modify`, or `open` to each eligible BPMN or CMMN node or edge in an open tab. The statuses SHALL mean, respectively, that an assistant is being asked to create the represented work/content, must not change the element, is allowed and expected to change the element, or receives no hint. An element with no assigned or recognized status SHALL be treated as `open`. Status SHALL remain an authoring hint and SHALL NOT prevent manual editing. Until persistence is implemented, status SHALL be client-side state scoped to the lifetime of the open tab and SHALL NOT be written to diagram XML or Markdown.

#### Scenario: User selects an eligible element
- **WHEN** the user selects a BPMN node or edge in the active editor
- **THEN** the selected-item metadata displays its current status and a textual explanation of the corresponding LLM hint

#### Scenario: User changes node status
- **WHEN** the user changes a selected node from `open` to `locked`, `modify`, or `new`
- **THEN** the application updates that node's client-side status immediately without preventing manual edits or scheduling BPMN or Markdown persistence

#### Scenario: Unsupported element is selected
- **WHEN** the current selection is a label, diagram root, or no element
- **THEN** the status control is unavailable and no element status changes

#### Scenario: Tab is reopened before persistence exists
- **WHEN** a tab containing client-side node statuses is closed and its diagram is opened again
- **THEN** its nodes default to `open` because GrafeoDB persistence is outside this change

### Requirement: Status color and non-color presentation
The editor SHALL render the primary fill of an eligible node green for `new`, grey for `locked`, orange for `modify`, and white for `open`, while preserving BPMN outlines, icons, labels, selection cues, and type semantics. The editor SHALL also expose the status name and LLM meaning in text and accessible labeling so status is not communicated by color alone.

#### Scenario: Node status changes color
- **WHEN** a user assigns `new`, `locked`, `modify`, or `open` to a node
- **THEN** its primary fill changes immediately to green, grey, orange, or white respectively while its BPMN type remains recognizable

#### Scenario: Status is interpreted without color
- **WHEN** a user cannot distinguish the node fill colors or navigates with assistive technology
- **THEN** the selected-item metadata and accessible node description identify the exact status and its LLM meaning

#### Scenario: Node retains interaction cues
- **WHEN** a status-colored node is selected, focused, or hovered
- **THEN** its status fill remains understandable and its existing interaction outline or marker remains visible

### Requirement: Automatic persistence
The application SHALL automatically save changed CMMN or BPMN XML and Markdown after a short debounce, SHALL serialize writes to each path, and SHALL show whether the current content is pending, saved, or failed.

#### Scenario: Diagram model changes
- **WHEN** the user completes one or more supported CMMN or BPMN mutations
- **THEN** the latest model is serialized to its diagram XML format and saved to its original path without a manual save action

#### Scenario: Markdown source changes
- **WHEN** the user pauses after editing Markdown source
- **THEN** the latest source is saved to the selected target's derived Markdown path

#### Scenario: Save fails
- **WHEN** the local server rejects or cannot complete an automatic save
- **THEN** the application shows a failed state and does not report the affected revision as saved

### Requirement: Typed local file operations
The Rust server SHALL expose only the project-metadata read, diagram listing, confined CMMN and BPMN read/write, Markdown read/write, metadata rename, and composition-resolution operations required by the web application; SHALL return only the project directory basename from project metadata rather than its absolute path; and SHALL replace a saved file only after receiving complete content.

#### Scenario: Project metadata is read
- **WHEN** the browser requests project metadata
- **THEN** the server returns the basename of its canonical initialized project root without returning the absolute project or schematics path

#### Scenario: Complete content is saved
- **WHEN** the server receives a valid typed write request for a confined path
- **THEN** it writes a temporary sibling and replaces the target with the complete content

### Requirement: AI assistant palette entry points
The SSW editor SHALL display a magic assistant action in the context palette of every eligible supported diagram shape node and a magic assistant action in the persistent diagram palette. The node action SHALL open an assistant dialog identified as scoped to that node; the diagram action SHALL open the same dialog identified as scoped to the complete active diagram. Both actions SHALL provide accessible names, tooltips, keyboard operation, focus management, and visible hover/focus states consistent with existing palette controls.

#### Scenario: User invokes node assistance
- **WHEN** the user activates the magic action in an eligible node's context palette
- **THEN** the assistant dialog opens, identifies the node and active composition as its primary scope, and focuses the prompt input

#### Scenario: User invokes diagram assistance
- **WHEN** the user activates the magic action in the persistent diagram palette
- **THEN** the assistant dialog opens, identifies the complete active diagram as its scope, and focuses the prompt input

#### Scenario: Unsupported element palette is viewed
- **WHEN** the current context palette belongs to a connection, label, diagram root, or other unsupported element
- **THEN** it does not expose a node-scoped magic action

#### Scenario: Assistant dialog is dismissed
- **WHEN** the user cancels or closes the assistant dialog before approving a proposal
- **THEN** focus returns to the invoking palette control and no diagram or documentation content changes

### Requirement: CMMN-rooted tabbed editing
The diagram column SHALL open `main.cmmn` as the retained project-root tab and SHALL open linked BPMN designs as auxiliary tabs. Shared tab controls, breadcrumbs, inspector, Markdown panel, automatic persistence, and save status SHALL operate according to the active diagram type.

#### Scenario: BPMN design opens beside CMMN root
- **WHEN** a user opens `cybling.sdk.Birth` from the root CMMN anchor
- **THEN** the BPMN file receives one selectable auxiliary tab and the root CMMN tab remains retained

### Requirement: Diagram-type-aware controls
The editor SHALL obtain supported element creation, connection, composition, metadata, and modeler operations from the active diagram adapter. It SHALL hide or disable an operation that has no supported equivalent for the active CMMN element rather than applying a BPMN-specific command.

#### Scenario: Unsupported CMMN operation is inspected
- **WHEN** the selected CMMN element does not support a BPMN-only action
- **THEN** the editor does not present that action as available and does not mutate the diagram

### Requirement: CMMN magic actions
The editor SHALL display the existing diagram-scoped magic action for an active CMMN diagram and the existing node-scoped magic action for eligible CMMN elements. Each action SHALL use the same prompt, preview, approval, rejection, error, and revert experience as BPMN.

#### Scenario: User invokes CMMN node magic action
- **WHEN** a user invokes the magic action on an eligible CMMN Process Task
- **THEN** the assistant dialog identifies that element and its owning CMMN package as the request scope

### Requirement: Business context remains navigable
When a BPMN design is opened from a CMMN Process Task, the editor SHALL retain the originating CMMN tab and SHALL present the two documents as a need-to-design navigation relationship without requiring an intermediate CMMN package diagram.

#### Scenario: Linked BPMN tab opens
- **WHEN** a user opens `cybling.sdk.Birth` from a Process Task in `cybling/main.cmmn`
- **THEN** both tabs remain available and the user can return directly to the CMMN business context
