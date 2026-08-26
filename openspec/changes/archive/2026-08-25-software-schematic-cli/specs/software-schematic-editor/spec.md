## ADDED Requirements

### Requirement: Two-column schematic workspace
The web application SHALL present a polished, responsive diagram column on the left and a metadata/documentation column on the right, with the diagram column occupying the larger share of the available workspace.

#### Scenario: Application opens
- **WHEN** the initialized application loads successfully
- **THEN** it displays `schematics/main.bpmn` in the left editor and the current diagram documentation in the right column

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

### Requirement: Tabbed BPMN editing
The diagram column SHALL provide one retained, selectable editor tab per canonical BPMN path and SHALL use bundled `bpmn-js` assets to render and edit each diagram.

#### Scenario: Multiple diagrams are opened
- **WHEN** the user opens BPMN files with different canonical paths
- **THEN** each file has one tab and switching tabs preserves each editor session

#### Scenario: An open diagram is requested again
- **WHEN** the application requests a BPMN path that already has a tab
- **THEN** it focuses the existing tab without creating a duplicate

### Requirement: Compact selected-item metadata
The top of the right column SHALL display the selected diagram item's ID, label, BPMN type, and derived Markdown documentation path in a compact form; ID and label SHALL be editable and type and documentation path SHALL be read-only.

#### Scenario: User selects a BPMN node or edge
- **WHEN** a BPMN node or edge is selected in the active editor
- **THEN** the metadata header displays values derived from that element and its composition folder

#### Scenario: User updates an element label
- **WHEN** the user changes the selected element's label in the metadata header
- **THEN** the active BPMN model updates its name and schedules the diagram for automatic save

#### Scenario: User updates an element ID
- **WHEN** the user submits a valid unused BPMN ID
- **THEN** the model adopts the ID and any existing documentation file is renamed from the old ID to the new ID without overwriting another file

### Requirement: Markdown display and source editing
The remainder of the right column SHALL render the selected item's Markdown by default and SHALL provide an edit icon that switches between rendered content and Markdown source.

#### Scenario: Documentation is viewed
- **WHEN** a diagram or BPMN element becomes the current documentation target
- **THEN** its Markdown is rendered, including Mermaid fenced blocks rendered as diagrams

#### Scenario: User enters documentation edit mode
- **WHEN** the user activates the edit icon
- **THEN** the rendered view is replaced by an editable Markdown source control containing the same content

#### Scenario: User leaves documentation edit mode
- **WHEN** the user activates the edit icon while source is visible
- **THEN** the current source is saved and the column returns to rendered Markdown

### Requirement: Automatic persistence
The application SHALL automatically save changed BPMN XML and Markdown after a short debounce, SHALL serialize writes to each path, and SHALL show whether the current content is pending, saved, or failed.

#### Scenario: BPMN model changes
- **WHEN** the user completes one or more BPMN mutations
- **THEN** the latest model is serialized to BPMN XML and saved to its original path without a manual save action

#### Scenario: Markdown source changes
- **WHEN** the user pauses after editing Markdown source
- **THEN** the latest source is saved to the selected target's derived Markdown path

#### Scenario: Save fails
- **WHEN** the local server rejects or cannot complete an automatic save
- **THEN** the application shows a failed state and does not report the affected revision as saved

### Requirement: Typed local file operations
The Rust server SHALL expose only the diagram listing, BPMN read/write, Markdown read/write, metadata rename, and composition-resolution operations required by the web application, and SHALL replace a saved file only after receiving complete content.

#### Scenario: Complete content is saved
- **WHEN** the server receives a valid typed write request for a confined path
- **THEN** it writes a temporary sibling and replaces the target with the complete content
