## ADDED Requirements

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
