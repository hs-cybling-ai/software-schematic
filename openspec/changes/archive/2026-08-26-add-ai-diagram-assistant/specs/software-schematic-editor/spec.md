## ADDED Requirements

### Requirement: AI assistant palette entry points
The SSW editor SHALL display a magic assistant action in the context palette of every eligible BPMN shape node and a magic assistant action in the persistent diagram palette. The node action SHALL open an assistant dialog identified as scoped to that node; the diagram action SHALL open the same dialog identified as scoped to the complete active diagram. Both actions SHALL provide accessible names, tooltips, keyboard operation, focus management, and visible hover/focus states consistent with existing palette controls.

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
