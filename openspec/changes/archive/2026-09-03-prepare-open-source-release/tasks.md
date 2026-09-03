## 1. Establish the Cleanup Baseline

- [x] 1.1 Record the tracked-file inventory, top-level directory sizes, manifest/source path references, and a retained-path allowlist with a one-line purpose for every retained top-level path.
- [x] 1.2 Run and record the current web tests/build, Rust formatting/tests/release build, OpenSpec validation, and CLI `init`/`update` smoke tests before deleting files.
- [x] 1.3 Classify every tracked large or generated artifact, including the web bundle and MiniLM model, as required build input, reproducible build output, runtime asset, test asset, or removable content.

## 2. Add Licensing and Attribution

- [x] 2.1 Add the canonical Apache License 2.0 text as top-level `LICENSE` and a concise `NOTICE` naming Software Schematic and Cybling Labs, Inc.
- [x] 2.2 Change publishable Rust/package metadata and project documentation to consistently declare `Apache-2.0` without applying Cybling ownership claims to third-party artifacts.
- [x] 2.3 Inventory retained Rust, JavaScript, model, font, and bundled browser dependencies; preserve upstream notices and add a concise third-party attribution document with a repeatable review procedure.
- [x] 2.4 Include the project license, notice, and required third-party attribution files in initialized and updated project-local distributions, with tests for their presence and preservation behavior.
- [x] 2.5 Flag the completed license, attribution, and trademark language for owner/legal review before the repository is made public.

## 3. Create the Maintained Build and Release Path

- [x] 3.1 Replace the legacy native-editor build script with one root verification script that installs web dependencies from the lockfile, tests/builds `software-schematic-web`, formats/tests/builds `software-schematic-cli`, validates OpenSpec, and checks packaged assets.
- [x] 3.2 Add temporary-project smoke tests to the verification path for CLI `init`, wrapper launch/help, MCP project binding, and `update` while proving authored diagrams and Markdown remain byte-for-byte unchanged.
- [x] 3.3 Add a clean-tracked-export mode that prevents ignored build products, prior web bundles, or undeclared local files from satisfying release verification.
- [x] 3.4 Add a minimal GitHub Actions workflow for supported build hosts that invokes the same repository verification entry point used locally.
- [x] 3.5 Document production bundle regeneration, supported release targets, required native ONNX Runtime behavior, and the distinction between development dependencies and the self-contained installed runtime.

## 4. Remove Retired Product Code and Data

- [x] 4.1 Re-run reference searches, then remove `DiagramStudio/`, `DiagramStudio.xcodeproj/`, `Sources/`, `Package.swift`, and `project.yml` after confirming no retained build or test uses the native Swift application.
- [x] 4.2 Remove `web-editor/` and its legacy `scripts/build-web-assets.sh` after confirming `software-schematic-web` is the sole source for the embedded browser bundle.
- [x] 4.3 Remove native/Data Graph/ArchiMate fixtures and tests; retain or relocate only fixtures demonstrably exercised by the active CLI or browser test suites.
- [x] 4.4 Remove root sample `schematics/`, `ssw`, and `ssw.cmd` after tests prove that starter documents and wrappers are generated from retained CLI sources/assets.
- [x] 4.5 Remove obsolete documentation or fold still-accurate CLI guidance into the public README and focused retained docs before deletion.
- [x] 4.6 Update `.gitignore` and remaining path references so they describe only the retained Rust CLI, browser source, generated bundle, and supported release workflow.

## 5. Reduce OpenSpec to Current Contracts

- [x] 5.1 Map every retained runtime feature to its current main spec and identify any references to retired native macOS, Data Graph, ArchiMate, or generic workspace capabilities.
- [x] 5.2 Remove `openspec/changes/archive/` and obsolete main capability specs while retaining the active release-preparation change and current CLI/browser/CMMN/BPMN/assistant/graph/MCP contracts.
- [x] 5.3 Repair retained spec references and run `openspec validate --all`, confirming that no active requirement depends on removed planning history or retired behavior.

## 6. Prepare the Public Repository Surface

- [x] 6.1 Rewrite the top-level README around Software Schematic CLI with purpose, screenshots only if current, prerequisites, supported platforms, source build, tests, install, update, run, MCP setup, offline behavior, license, and Cybling Labs/cybling.ai context.
- [x] 6.2 Add concise `CONTRIBUTING.md` instructions that use the canonical verification workflow and explain generated-asset, spec, dependency-notice, and change-submission expectations.
- [x] 6.3 Add `SECURITY.md` with supported-version and private vulnerability-reporting guidance after confirming the public security contact.
- [x] 6.4 Check all retained Markdown links, commands, repository-relative paths, package names, descriptions, and ownership statements from a clean checkout.

## 7. Prove the Minimal Release Candidate

- [x] 7.1 Run the full verification workflow in a temporary clean tracked export on macOS and the applicable CI targets, with no removed directories copied into the export.
- [x] 7.2 Compare the final tracked tree to the retained-path allowlist and fail the review for unexplained files, missing required assets/notices, or tracked local build output.
- [x] 7.3 Build the final release executable and verify offline editor assets, local model installation, assistant schema, project wrappers, MCP startup, `init`, and non-destructive `update` behavior.
- [x] 7.4 Review the complete deletion diff, license/notice inventory, and clean-build evidence before committing; restore any required input from Git rather than weakening verification.
- [x] 7.5 Record the GitHub publication checklist—repository URL, visibility transition, default branch protection, release artifact checksums/signing decision, security contact, and owner/legal approval—without publishing externally in this change.
