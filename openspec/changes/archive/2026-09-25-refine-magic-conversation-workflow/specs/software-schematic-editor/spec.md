## ADDED Requirements

### Requirement: Chat-style assistant interaction
The SSW editor SHALL present each magic-button invocation as a chat-style dialog with a scrollable transcript and a message composer anchored to the bottom of the available dialog viewport. The composer SHALL place a compact send control in its lower corner, keep that control keyboard accessible, and provide an accessible name and tooltip. Activating send or pressing Enter on non-empty input SHALL append the user message, clear the composer immediately, preserve a blank input for the next turn, and display the generated assistant response in the transcript. Shift+Enter SHALL insert a newline, Enter during IME composition SHALL NOT submit, and empty or whitespace-only input SHALL NOT submit. While a conversational response is being generated, the send control SHALL become an accessible stop-generation control.

Completed assistant responses that are eligible to produce changes SHALL expose a labeled `Suggest changes` action beneath the response. A valid structured result SHALL appear inline in the transcript as an outlined `Suggested updates` section containing its grouped changes, assumptions, warnings, a secondary `Continue interview` action, and a prominent `Approve changes` action. The dialog close control SHALL remain available in the header. Icon-only presentation SHALL be limited to conventional send, stop, and close actions and SHALL preserve accessible names, tooltips, focus indication, and sufficient target size.

#### Scenario: User sends from the composer
- **WHEN** the user activates the send control with a non-empty message
- **THEN** the message is appended to the transcript, the composer becomes blank and remains available at the bottom, and the assistant response is displayed as the next transcript entry

#### Scenario: User sends with the keyboard
- **WHEN** the composer contains a non-empty message and the user presses Enter outside an active IME composition
- **THEN** the editor performs the same submission as the send control

#### Scenario: User enters multiline text
- **WHEN** the user presses Shift+Enter in the composer
- **THEN** the editor inserts a newline without submitting the message

#### Scenario: User is composing text with an IME
- **WHEN** an Enter key event occurs before the active composition is complete
- **THEN** the editor does not submit the message

#### Scenario: User stops response generation
- **WHEN** a conversational response is in flight and the user activates the stop-generation control
- **THEN** the editor cancels the active request, retains the already submitted user message, and returns the composer to a send-ready state without creating a proposal

#### Scenario: Assistant response offers structured generation
- **WHEN** an eligible conversational response finishes
- **THEN** a labeled `Suggest changes` action appears beneath that assistant response while the bottom composer remains available

#### Scenario: Structured suggestions are generated
- **WHEN** the user activates `Suggest changes` and the returned plan passes validation
- **THEN** an outlined `Suggested updates` section is appended inline with the complete grouped update list and visible `Continue interview` and `Approve changes` actions

#### Scenario: Dialog viewport is constrained
- **WHEN** the transcript exceeds the available dialog height
- **THEN** the transcript scrolls independently while the composer remains available at the bottom without obscuring the latest content

## MODIFIED Requirements

### Requirement: AI assistant palette entry points
The SSW editor SHALL display a magic assistant action in the context palette of every eligible supported diagram shape node and a magic assistant action in the persistent diagram palette. The node action SHALL open a new assistant invocation identified as scoped to that node; the diagram action SHALL open a new assistant invocation identified as scoped to the complete active diagram. Every activation SHALL flush relevant edits, initialize the dialog from current persisted scoped values, and clear any transcript, draft proposal, error, request, or phase state left by an earlier invocation. Both actions SHALL provide accessible names, tooltips, keyboard operation, focus management, and visible hover/focus states consistent with existing palette controls.

#### Scenario: User invokes node assistance
- **WHEN** the user activates the magic action in an eligible node's context palette
- **THEN** the assistant dialog opens in interview mode, identifies the node and active composition as its primary scope, shows an empty invocation transcript, and focuses the prompt input

#### Scenario: User invokes diagram assistance
- **WHEN** the user activates the magic action in the persistent diagram palette
- **THEN** the assistant dialog opens in interview mode, identifies the complete active diagram as its scope, shows an empty invocation transcript, and focuses the prompt input

#### Scenario: Magic action is invoked after another session
- **WHEN** the user activates a magic action after closing or completing an earlier assistant invocation
- **THEN** the dialog starts over from the newly persisted target and diagram values without loading browser-stored conversation state from the earlier invocation

#### Scenario: Unsupported element palette is viewed
- **WHEN** the current context palette belongs to a connection, label, diagram root, or other unsupported element
- **THEN** it does not expose a node-scoped magic action

#### Scenario: Assistant dialog is dismissed
- **WHEN** the user cancels or closes the assistant dialog before approving a proposal
- **THEN** focus returns to the invoking palette control, the ephemeral invocation state is discarded, and no diagram or documentation content changes

### Requirement: CMMN magic actions
The editor SHALL display the existing diagram-scoped magic action for an active CMMN diagram and the existing node-scoped magic action for eligible CMMN elements. Each action SHALL use the same fresh invocation, interview, explicit suggestion transition, preview, continue-interview, approval, close, error, and revert experience as BPMN.

#### Scenario: User invokes CMMN node magic action
- **WHEN** a user invokes the magic action on an eligible CMMN Process Task
- **THEN** a new interview identifies that element and its owning CMMN package as the request scope and contains no state from an earlier invocation

#### Scenario: CMMN proposal needs more discussion
- **WHEN** a user continues the interview from a CMMN structured preview
- **THEN** the preview is invalidated and the current invocation resumes conversational mode without applying changes
