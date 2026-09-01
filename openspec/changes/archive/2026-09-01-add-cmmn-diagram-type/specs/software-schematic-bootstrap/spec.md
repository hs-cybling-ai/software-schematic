## ADDED Requirements

### Requirement: Bundled CMMN assets
The `ss init` output SHALL include the pinned local CMMN modeler runtime, styles and required vendor assets, the registered SSW CMMN moddle descriptor, and a starter CMMN document in the self-contained `.ss/` installation. Browser-executable assets SHALL be produced by the existing web build from the HTML, CSS, and TypeScript source and SHALL require no runtime package manager or CDN.

#### Scenario: Initialized project opens CMMN offline
- **WHEN** a newly initialized project opens a `.cmmn` file without network access
- **THEN** the extracted `.ss/` assets load and edit the diagram without requesting external code, styles, fonts, or services

### Requirement: CMMN-rooted initialization
Adding CMMN support SHALL initialize `schematics/main.cmmn` as the sole project anchor and SHALL NOT create a competing `schematics/main.bpmn`. The root CMMN SHALL be the project entry for domain, actors, inputs, outputs, needs, business services, and Process Task links to BPMN logical architecture and solution building blocks.

#### Scenario: New project starts normally
- **WHEN** `ss init` completes after this change
- **THEN** the project starts from `schematics/main.cmmn`, no root BPMN exists, and BPMN designs are created or opened from CMMN Process Tasks

#### Scenario: Legacy BPMN-only project starts
- **WHEN** an existing project has `schematics/main.bpmn` and no `schematics/main.cmmn`
- **THEN** the editor opens the legacy BPMN root as a compatibility fallback without creating a second anchor

#### Scenario: Competing roots are detected
- **WHEN** both `schematics/main.cmmn` and `schematics/main.bpmn` exist
- **THEN** startup reports competing anchors instead of choosing one silently
