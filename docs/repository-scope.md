# Repository scope and release gates

This repository maintains one product: the Software Schematic CLI and the
project-local wrapper/browser runtime it installs.

## Retained top-level paths

| Path | Purpose |
| --- | --- |
| `.codex/` | Local OpenSpec workflow skills used to maintain the specs. |
| `.github/` | Public continuous integration. |
| `docs/` | Focused usage, architecture, scope, and publication guidance. |
| `openspec/` | Current normative product specs and the active change only. |
| `scripts/` | Canonical build and release verification. |
| `software-schematic-cli/` | Rust product, tests, templates, and embedded runtime assets. |
| `software-schematic-web/` | Browser editor source, tests, and locked dependencies. |
| Root Markdown/license files | Public overview, contribution, security, license, and notices. |
| `.gitignore` | Excludes reproducible local build/dependency output and secrets. |

Any new top-level path must have a build, deployment, runtime, test,
specification, documentation, legal, or contribution purpose and must be added
to the allowlist in `scripts/verify-release.sh`.

## Required generated and large files

- `software-schematic-cli/assets/web/` is a reproducible output of the web build
  and is committed because the Rust binary embeds it for offline installation.
- `software-schematic-cli/assets/models/all-MiniLM-L6-v2/` contains the ONNX
  model, tokenizer, and source notice required for project-local MCP embeddings.
  The model is intentionally retained even though it dominates repository size.
- `Cargo.lock` and `package-lock.json` are exact dependency inputs for
  reproducible application builds.

Local `target/`, `node_modules/`, coverage, temporary, and environment files are
reproducible or private and must remain untracked.

## Removed scope

The native Swift/Xcode DiagramStudio, Data Graph and ArchiMate editor, legacy
web editor, native fixtures, sample workspace, and archived OpenSpec history are
available through Git history but are not part of the maintained source tree.

## Current capability contracts

Each retained runtime feature maps to a current main specification:

| Runtime feature | Main specification |
| --- | --- |
| AI-assisted diagram changes | `ai-diagram-assistance` |
| BPMN editing | `bpmn-diagram-editing` |
| CMMN editing and package traversal | `cmmn-diagram-editing`, `cmmn-package-composition` |
| Save-triggered graph refresh | `document-save-graph-refresh` |
| Project-local MCP server and graph | `schematic-mcp`, `schematic-property-graph` |
| Project initialization and update | `software-schematic-bootstrap` |
| Nested schematic composition and naming | `software-schematic-composition`, `scoped-process-naming` |
| Browser editor behavior | `software-schematic-editor` |

The active `prepare-open-source-release` change adds the future
`open-source-project-distribution` contract. No retained specification refers
to the retired native macOS shell, Data Graph, ArchiMate, generic diagram
workspace, or node-Markdown-context capabilities.

## Baseline recorded before cleanup

On 2026-09-03 the pre-cleanup baseline passed:

- 51 browser tests and a production Vite build.
- 34 Rust unit tests, 2 Rust integration tests, formatting, and release build.
- 17 OpenSpec validations, including this active change.
- Existing Rust tests covering complete initialization, MCP project binding,
  embedded model loading, and non-destructive idempotent update.

The inventory contained 114 archived/current OpenSpec files, 40 DiagramStudio
files, 26 legacy web-editor files, 24 legacy fixtures, 90 CLI files, and 16
current browser-source files. The active build referenced only the CLI's
embedded assets and current browser source.

## Publication checklist

- [ ] Confirm the public GitHub repository URL and replace any future placeholders.
- [ ] Enable private vulnerability reporting and confirm the security contact.
- [ ] Configure protected default-branch reviews and required CI checks.
- [ ] Decide supported release targets and produce checksums for every artifact.
- [ ] Decide whether release artifacts require platform signing/notarization.
- [ ] Complete Cybling Labs owner/legal review of Apache licensing,
      third-party notices, and trademark language.
- [ ] Keep the repository private until all unchecked publication gates are approved.

Repository preparation does not perform any of these external publication actions.
