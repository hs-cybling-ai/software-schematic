## MODIFIED Requirements

### Requirement: Cross-platform single-executable CLI
The maintained source distribution SHALL build `ss` as one self-contained native executable for each supported Windows and macOS target. Each executable SHALL provide the `init`, `update`, `serve`, authentication, and MCP commands without requiring a language runtime or package manager after installation, and SHALL carry the project license, notice, required third-party notices, bundled web application, starter documents, and embedding assets needed by those commands.

#### Scenario: Run the CLI on a supported target
- **WHEN** a user invokes the matching `ss` executable on Windows or macOS
- **THEN** the CLI runs without requiring Node.js, Rust, or another separately installed language runtime

#### Scenario: Build the CLI from the public source tree
- **WHEN** a contributor follows the documented clean-checkout release workflow on a supported build host
- **THEN** the workflow produces the self-contained executable entirely from retained source, lockfiles, and bundled assets without any retired native-editor or Data Graph directory

#### Scenario: Inspect an installed distribution
- **WHEN** a user initializes or updates a project with the release executable
- **THEN** the installed runtime contains the applicable Software Schematic license, Cybling Labs notice, required third-party notices, browser assets, templates, and model assets
