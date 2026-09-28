## MODIFIED Requirements

### Requirement: Tabbed diagram editing
The diagram column SHALL provide one retained, selectable editor tab per complete canonical `.cmmn` or `.bpmn` path and SHALL use the corresponding bundled modeler to render and edit each diagram. The project's normalized sole root anchor tab SHALL remain open as the navigation entry point and SHALL NOT expose an actionable close control or be removable through the tab-close operation. Every other diagram tab SHALL display a keyboard-accessible `x` close control. Closing a permitted tab SHALL remove that retained editor session and release its UI/modeler resources without changing other open sessions; when an active auxiliary tab closes, the application SHALL select the nearest remaining tab, preferring the next tab and otherwise the previous tab. The application SHALL safely flush queued automatic persistence for the closing tab before releasing it, but SHALL NOT wait for queued or processing graph-refresh work to finish.

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
- **THEN** the application flushes durable persistence, releases its editor without waiting for graph processing, and activates the next tab or the previous tab when no next tab exists

#### Scenario: Root main diagram is displayed
- **WHEN** the project root anchor tab is open
- **THEN** it has no actionable `x` control and the tab-close operation refuses to remove its editor session

### Requirement: Document and graph save status
The web application SHALL distinguish durable document persistence from asynchronous derived graph refresh. After a successful diagram or connected Markdown save, it SHALL report whether refresh was queued, is processing, published a new revision, found no revision change, failed while retaining its prior revision, or could not be scheduled because the MCP was not running. A refresh failure SHALL NOT be represented as a document-save failure and SHALL NOT roll back the authored file.

#### Scenario: Save queues graph refresh
- **WHEN** a document saves and the running MCP accepts its change notification
- **THEN** the web application reports the document as saved and the graph refresh as queued without waiting for publication

#### Scenario: Queued graph refresh succeeds
- **WHEN** the background worker publishes the replacement graph
- **THEN** the web application reports the updated active revision

#### Scenario: Save succeeds without MCP
- **WHEN** the document saves but no matching MCP process is running
- **THEN** the web application reports the document as saved and the graph as not refreshed

#### Scenario: Save succeeds but graph refresh fails
- **WHEN** the document saves but background refresh retains its prior graph after a build diagnostic
- **THEN** the web application reports the document as saved, the graph update as failed, and an actionable diagnostic

