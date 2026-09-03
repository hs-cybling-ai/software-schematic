## Why

The repository still contains a discontinued native macOS/Data Graph product, sample project content, and extensive planning history even though its maintained purpose is now the portable Software Schematic CLI and project-local wrapper. Before publishing it for use outside Cybling Labs, the repository needs a clear Apache 2.0 license, a minimal reproducible source layout, and public-facing documentation that separates the Software Schematic project from the cybling.ai product.

## What Changes

- License the original project code under Apache License 2.0 with Cybling Labs, Inc. identified as copyright owner, while retaining applicable third-party license and notice material.
- Rewrite the repository entry-point documentation for public GitHub users, including supported platforms, prerequisites, source builds, tests, installation into another project, updates, MCP integration, contribution expectations, and the relationship to Cybling Labs and cybling.ai.
- **BREAKING**: remove the discontinued Swift `DiagramStudio`, Data Graph/ArchiMate web editor, native-only fixtures, sample schematics, obsolete root wrappers/build manifests, and documentation that do not participate in building, testing, packaging, installing, updating, or running the Software Schematic CLI.
- Remove archived OpenSpec change history and obsolete main specs, retaining only current behavioral specifications and this active release-preparation change as the minimal planning baseline for continued development.
- Replace legacy build scripts with a small, documented release/verification workflow for the Rust CLI and its bundled `software-schematic-web` application.
- Audit generated web assets, the embedded MiniLM model, package manifests, lockfiles, ignore rules, and repository metadata so every retained large or generated file has a build, deployment, runtime, test, or legal purpose.
- Add automated clean-checkout verification that builds the web bundle and CLI, runs both test suites, confirms packaged assets are present, and exercises CLI initialization/update without depending on removed directories.

## Capabilities

### New Capabilities

- `open-source-project-distribution`: Defines licensing, attribution, minimal source-tree contents, public documentation, and clean-checkout release verification for distributing Software Schematic outside Cybling Labs.

### Modified Capabilities

- `software-schematic-bootstrap`: Clarifies that the maintained source distribution builds and installs only the CLI, bundled browser editor, project-local wrappers, templates, model assets, and integration guidance.

## Impact

- Removes the root Swift package/Xcode project, `DiagramStudio/`, `web-editor/`, legacy fixtures and scripts, repository sample `schematics/`, obsolete documentation, archived OpenSpec history, and specs for retired native/Data Graph/ArchiMate behavior after dependency verification.
- Retains and may reorganize `software-schematic-cli/`, `software-schematic-web/`, required embedded assets, relevant docs/specs, lockfiles, tests, and development skills.
- Adds top-level Apache 2.0 licensing and minimal GitHub-facing project metadata/documentation.
- Changes contributor build and validation commands but does not change installed project data formats or the `ss`/`ssw` runtime contract.
