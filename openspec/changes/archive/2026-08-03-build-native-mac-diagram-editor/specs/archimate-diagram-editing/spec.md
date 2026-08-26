## ADDED Requirements

### Requirement: ArchiMate modeler loading
The application SHALL load supported ArchiMate exchange XML files into an editable `archimate-js` modeler backed by bundled local assets.

#### Scenario: Valid ArchiMate file opens
- **WHEN** the user opens an ArchiMate XML file supported by the installed adapter
- **THEN** the system renders its model and view in an ArchiMate modeler with editing controls enabled

#### Scenario: Invalid ArchiMate file opens
- **WHEN** the ArchiMate importer rejects the selected file
- **THEN** the system reports the import failure, does not mark the tab dirty, and leaves the source file unchanged

### Requirement: ArchiMate mutation tracking
The application SHALL mark an ArchiMate tab dirty when a modeler command changes its model or view and SHALL retain the modeler session while the tab remains open.

#### Scenario: User edits ArchiMate content
- **WHEN** the user creates, changes, moves, or removes an ArchiMate element or relationship
- **THEN** the tab becomes dirty and the edit remains available when the user switches away from and back to the tab

#### Scenario: ArchiMate file is only viewed
- **WHEN** the user opens and navigates an ArchiMate diagram without changing model content
- **THEN** the tab remains clean

### Requirement: ArchiMate XML export
The application SHALL serialize the current ArchiMate model to the XML format supported by `archimate-js` before a save is written.

#### Scenario: Edited ArchiMate diagram is saved
- **WHEN** the native layer requests export from a valid edited ArchiMate session
- **THEN** the web editor returns serialized ArchiMate XML corresponding to the current model and view

#### Scenario: ArchiMate serialization fails
- **WHEN** `archimate-js` cannot serialize the current model
- **THEN** the web editor returns a correlated failure and no file write occurs

### Requirement: ArchiMate compatibility is verified
The application SHALL use an exact locked set of `archimate-js` and `diagram-js` ecosystem versions that passes automated import, mutation, and export smoke tests for a representative fixture.

#### Scenario: Web dependencies are updated
- **WHEN** an ArchiMate or shared diagram dependency version changes
- **THEN** the build is accepted only if the ArchiMate compatibility smoke test succeeds

### Requirement: ArchiMate editor diagnostics
The application SHALL present import warnings and runtime failures produced by the ArchiMate adapter without disabling unrelated tabs.

#### Scenario: Import completes with warnings
- **WHEN** `archimate-js` loads an ArchiMate file and returns non-fatal warnings
- **THEN** the affected tab makes those warnings available while remaining editable
