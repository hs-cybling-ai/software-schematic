## MODIFIED Requirements

### Requirement: Tabbed BPMN editing
The diagram column SHALL provide one retained, selectable editor tab per canonical BPMN path and SHALL use bundled `bpmn-js` assets to render and edit each diagram. The project's normalized root `schematics/main.bpmn` tab SHALL remain open as the navigation entry point and SHALL NOT expose an actionable close control or be removable through the tab-close operation. Every other diagram tab, including a nested composition named `main.bpmn`, SHALL display a keyboard-accessible `x` close control. Closing a permitted tab SHALL remove that retained editor session and release its UI/modeler resources without changing other open sessions; when an active auxiliary tab closes, the application SHALL select the nearest remaining tab, preferring the next tab and otherwise the previous tab. The application SHALL safely flush queued automatic persistence for the closing tab before releasing it.

#### Scenario: Multiple diagrams are opened
- **WHEN** the user opens BPMN files with different canonical paths
- **THEN** each file has one retained tab, switching tabs preserves each editor session, and every tab except root `schematics/main.bpmn` has a visible `x` close control

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
- **WHEN** the project root `schematics/main.bpmn` tab is open
- **THEN** it has no actionable `x` control and the tab-close operation refuses to remove its editor session

#### Scenario: Last auxiliary tab is closed
- **WHEN** the user closes the only auxiliary tab
- **THEN** the application releases that editor session and returns to the retained root `schematics/main.bpmn` tab

#### Scenario: Nested main diagram is displayed
- **WHEN** a composition opens a nested path such as `schematics/orders/main.bpmn`
- **THEN** that tab has an actionable `x` control because protection is based on the normalized root path rather than the filename

## ADDED Requirements

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
The SSW top bar SHALL provide a fullscreen control that uses the browser Fullscreen API to enter fullscreen for the application shell and exit the document's fullscreen state. The control SHALL derive its state from the document's actual fullscreen element, SHALL show a maximize icon labeled `Enter full screen` when windowed, and SHALL show a minimize icon labeled `Exit full screen` while fullscreen. Following each fullscreen transition, the application SHALL resize and fit the active BPMN canvas to its changed viewport. Unsupported or rejected fullscreen operations SHALL leave the current state intact and report an actionable failure.

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

## MODIFIED Requirements

### Requirement: Typed local file operations
The Rust server SHALL expose only the project-metadata read, diagram listing, BPMN read/write, Markdown read/write, metadata rename, and composition-resolution operations required by the web application; SHALL return only the project directory basename from project metadata rather than its absolute path; and SHALL replace a saved file only after receiving complete content.

#### Scenario: Project metadata is read
- **WHEN** the browser requests project metadata
- **THEN** the server returns the basename of its canonical initialized project root without returning the absolute project or schematics path

#### Scenario: Complete content is saved
- **WHEN** the server receives a valid typed write request for a confined path
- **THEN** it writes a temporary sibling and replaces the target with the complete content

## ADDED Requirements

### Requirement: LLM node authoring status
The SSW editor SHALL let a user assign exactly one status of `new`, `locked`, `modify`, or `open` to each eligible BPMN node in an open tab. The statuses SHALL mean, respectively, that an LLM is being asked to create the represented work/content, must not change the node, is allowed and expected to change the node, or receives no hint. A node with no assigned or recognized status SHALL be treated as `open`. Status SHALL remain an authoring hint and SHALL NOT prevent manual editing. Until GrafeoDB persistence is implemented, status SHALL be client-side state scoped to the lifetime of the open tab and SHALL NOT be written to BPMN XML or Markdown.

#### Scenario: User selects an eligible node
- **WHEN** the user selects a BPMN shape node in the active editor
- **THEN** the selected-item metadata displays its current status and a textual explanation of the corresponding LLM hint

#### Scenario: User changes node status
- **WHEN** the user changes a selected node from `open` to `locked`, `modify`, or `new`
- **THEN** the application updates that node's client-side status immediately without preventing manual edits or scheduling BPMN or Markdown persistence

#### Scenario: Unsupported element is selected
- **WHEN** the current selection is a connection, label, diagram root, or no element
- **THEN** the status control is unavailable and no node status changes

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
