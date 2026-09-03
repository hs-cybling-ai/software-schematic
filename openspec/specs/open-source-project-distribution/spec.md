# Open Source Project Distribution

## Purpose

Define the licensing, maintained repository scope, public documentation, and clean release-verification requirements for distributing Software Schematic as an open-source project.

## Requirements


### Requirement: Apache 2.0 project licensing
The repository SHALL contain the canonical Apache License 2.0 text, SHALL identify Cybling Labs, Inc. as copyright owner of the original Software Schematic work, and SHALL declare `Apache-2.0` in publishable package metadata. The project license SHALL NOT claim ownership of third-party dependencies, generated vendor content, fonts, or model assets distributed under their own terms.

#### Scenario: License metadata is inspected
- **WHEN** a user or automated scanner inspects a clean source checkout
- **THEN** it finds the Apache 2.0 license text, Cybling Labs attribution, and consistent `Apache-2.0` package metadata without conflicting project-license claims

### Requirement: Distribution notices
Source and binary distributions SHALL include the Software Schematic notice and every license or attribution notice required for retained third-party code, browser assets, fonts, and model assets. The repository SHALL document how the notice inventory is reviewed when dependencies or embedded artifacts change.

#### Scenario: Release contents are audited
- **WHEN** a release candidate is checked for legal notices
- **THEN** the project license, Cybling Labs notice, model notice, and required third-party attributions are present in the source tree and installed distribution

### Requirement: Minimal maintained source tree
The tracked repository SHALL retain only files needed to develop, specify, build, test, package, install, update, execute, document, license, or contribute to the Software Schematic CLI and project-local wrapper. It SHALL exclude the retired native DiagramStudio application, Data Graph and ArchiMate editor sources and fixtures, sample workspace state, generated local build directories, and archived planning history.

#### Scenario: Repository scope is verified
- **WHEN** the maintained-tree verification runs on a clean checkout
- **THEN** every allowed top-level path has a documented project purpose and no retired product tree, sample workspace, build output, or archived OpenSpec change is tracked

### Requirement: Current development specifications
The repository SHALL retain the current normative OpenSpec capabilities required to continue development of the CLI, browser editor, CMMN/BPMN composition, documentation/status behavior, assistant, compiled graph, refresh, and MCP integration. It SHALL remove obsolete capabilities and historical change artifacts after confirming no retained spec depends on them.

#### Scenario: Specifications are validated
- **WHEN** `openspec validate --all` runs after cleanup
- **THEN** all retained specifications pass and describe only maintained Software Schematic behavior

### Requirement: Public project documentation
The top-level documentation SHALL identify Software Schematic as an Apache-licensed project owned by Cybling Labs, Inc.; explain that Cybling Labs also makes the separate cybling.ai platform; and provide concise prerequisites, supported platforms, source-build, test, install, update, run, MCP, contribution, security, and licensing guidance suitable for an external organization.

#### Scenario: External developer starts from the README
- **WHEN** a developer without Cybling Labs internal context opens the repository
- **THEN** the documentation gives enough accurate information to build, test, install, and run Software Schematic and distinguishes it from cybling.ai

### Requirement: Clean-checkout release verification
The repository SHALL provide an automated workflow that starts from tracked source and lockfiles, installs pinned web dependencies, runs web and Rust tests, builds the production web bundle and release CLI, verifies required embedded assets, and smoke-tests initialization and update in temporary projects. The workflow SHALL fail when a required source, runtime asset, notice, or maintained-tree constraint is missing.

#### Scenario: Release candidate is verified
- **WHEN** the release verification workflow runs on a clean checkout
- **THEN** it proves the CLI and wrapper can be built, packaged, initialized, updated, and executed without any removed legacy directory or untracked local artifact

### Requirement: Authored project preservation
Repository cleanup and the resulting CLI update workflow SHALL preserve authored CMMN, BPMN, Markdown, and unrelated project files. Removal of repository sample workspace state SHALL NOT change the runtime rules for initialized user projects.

#### Scenario: Existing initialized project is updated
- **WHEN** a CLI built from the cleaned repository updates a compatible project
- **THEN** the runtime and managed integration files are refreshed while authored schematics, documentation, and unrelated files remain unchanged
