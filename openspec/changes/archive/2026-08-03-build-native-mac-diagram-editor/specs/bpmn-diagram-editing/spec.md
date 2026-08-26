## ADDED Requirements

### Requirement: BPMN modeler loading
The application SHALL load supported BPMN XML files into an editable `bpmn-js` modeler backed by bundled local assets.

#### Scenario: Valid BPMN file opens
- **WHEN** the user opens a valid supported BPMN XML file
- **THEN** the system renders its diagram in a BPMN modeler with editing controls enabled

#### Scenario: Invalid BPMN file opens
- **WHEN** the BPMN importer rejects the selected file
- **THEN** the system reports the import failure, does not mark the tab dirty, and leaves the source file unchanged

### Requirement: BPMN mutation tracking
The application SHALL mark a BPMN tab dirty when a modeler command changes its diagram and SHALL retain the modeler session while the tab remains open.

#### Scenario: User edits BPMN content
- **WHEN** the user creates, changes, moves, or removes a BPMN element
- **THEN** the tab becomes dirty and the edit remains available when the user switches away from and back to the tab

#### Scenario: BPMN file is only viewed
- **WHEN** the user opens and navigates a BPMN diagram without changing model content
- **THEN** the tab remains clean

### Requirement: BPMN XML export
The application SHALL serialize the current BPMN model to valid BPMN XML through `bpmn-js` before a save is written.

#### Scenario: Edited BPMN diagram is saved
- **WHEN** the native layer requests export from a valid edited BPMN session
- **THEN** the web editor returns serialized BPMN XML corresponding to the current canvas state

#### Scenario: BPMN serialization fails
- **WHEN** `bpmn-js` cannot serialize the current model
- **THEN** the web editor returns a correlated failure and no file write occurs

### Requirement: BPMN editor diagnostics
The application SHALL present import warnings and runtime failures produced by the BPMN adapter without disabling unrelated tabs.

#### Scenario: Import completes with warnings
- **WHEN** `bpmn-js` loads a BPMN file and returns non-fatal warnings
- **THEN** the affected tab makes those warnings available while remaining editable
