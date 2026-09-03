## Context

The repository began as a Swift/Xcode diagram editor with a separate `web-editor`, Data Graph storage, broad notation fixtures, sample diagrams, and many archived OpenSpec changes. The maintained product is now the Rust `ss` CLI, its `software-schematic-web` browser application, project-local `ssw` wrappers, bundled CMMN/BPMN/Markdown templates, local embedding assets, and the specifications and documentation for those behaviors.

The Rust build embeds `software-schematic-cli/assets/web`, `assets/models`, and starter documents at compile time. The web source builds directly into the embedded web directory. Nothing in the active Rust or web manifests references the Swift package, Xcode project, `DiagramStudio`, `web-editor`, root sample `schematics`, or legacy fixtures. Cleanup must nevertheless be driven by a retained-file allowlist and clean-checkout tests rather than by directory names alone.

Cybling Labs, Inc. owns the original work and wants Apache License 2.0 distribution so Software Schematic can be used independently by other organizations. Third-party dependencies and the bundled embedding model remain governed by their own licenses and notices.

## Goals / Non-Goals

**Goals:**

- Produce a small, understandable public repository whose only product is Software Schematic CLI and its project-local wrapper/editor runtime.
- Add an unmodified Apache License 2.0 license, Cybling Labs attribution, and retained third-party notices appropriate to source and binary distribution.
- Remove code, fixtures, generated products, sample project data, historical changes, and specs that do not support the maintained product.
- Preserve everything required to build, test, package, initialize, update, and run the CLI on supported platforms.
- Make a clean checkout reproducibly verifiable before deletion is committed.

**Non-Goals:**

- Rewrite Git history, erase authorship records, rename the product, or transfer ownership of cybling.ai.
- Change installed schematic formats, runtime commands, MCP semantics, or the current browser editing experience.
- Publish a GitHub repository or release artifacts as part of this change; the repository will be prepared for that subsequent external action.
- Add a large governance or community-documentation set beyond what is useful for a small open-source project.

## Decisions

### Use a retained-file allowlist

The implementation will first record the files and directories that form the supported source distribution: top-level license/readme/ignore metadata, `software-schematic-cli`, `software-schematic-web`, relevant docs, current OpenSpec specs, the active change, and any small release verification scripts. Everything proposed for removal will be checked against manifest references, source includes, tests, documentation links, and release commands before deletion.

This is safer than deleting directories solely because they look old. The allowlist also becomes a reviewable assertion about the repository's intended scope.

### Keep generated runtime assets that make the executable self-contained

The compiled web bundle, starter documents, tokenizer, ONNX model, and model notice will remain because the Rust executable embeds and installs them. Source inputs, exact lockfiles, and deterministic build commands will remain alongside generated assets. A generated artifact may be removed only if the release workflow regenerates it before the Rust compile and clean-checkout verification proves packaging remains complete.

The approximately 90 MB model is intentionally retained despite repository size because removing it would make MCP installation incomplete or introduce a network download at initialization time.

### Remove retired product trees and sample workspace state

After reference checks, remove `DiagramStudio/`, `DiagramStudio.xcodeproj/`, `Sources/`, `Package.swift`, `project.yml`, `web-editor/`, native/Data Graph fixtures, the legacy web-build script, root `ssw` launchers, and root `schematics/`. The root wrappers and schematics are development/sample workspace state, not inputs to the distributable CLI; installed wrappers and starter schematics are generated from Rust constants and retained assets.

Relevant conceptual documentation will be rewritten or folded into public CLI documentation before obsolete documents are removed. Tests that rely on fixtures will be retained only when they exercise the active CLI/web product; otherwise the obsolete tests and fixtures leave with their retired implementation.

### Collapse planning history to current product contracts

Remove `openspec/changes/archive/` and main specs describing the retired native macOS shell, Data Graph, ArchiMate, or generic workspace behavior that is not part of the CLI. Retain current specs for bootstrap, CMMN/BPMN editing and composition, assistant behavior, status/documentation, graph compilation/MCP, graph refresh, and naming. Update cross-references so `openspec validate --all` succeeds.

The active `prepare-open-source-release` change remains until implementation and archival. Git history remains the authoritative historical record; deleting archived planning copies does not rewrite it.

### Apply Apache 2.0 at the repository and distribution boundaries

Add the canonical Apache License 2.0 text as `LICENSE` and a concise `NOTICE` naming Cybling Labs, Inc. and Software Schematic. Set package manifest license metadata to `Apache-2.0`. Public documentation will state that cybling.ai is a separate Cybling Labs product and that use of Software Schematic does not grant rights to Cybling trademarks.

Do not blanket-relicense third-party code, fonts, generated vendor bundles, or the model. Preserve existing notices and add a compact third-party attribution document generated or reviewed from direct dependencies and bundled artifacts. Installed distributions will include the project license/notice and required third-party notices.

### Verify from a clean tracked export

Validation will use a temporary clean export of the candidate tracked tree so ignored local build products cannot mask missing inputs. It will install web dependencies from the lockfile, run web tests and build, run Rust formatting/tests/build, inspect the release executable's embedded assets, and exercise `ss init` and `ss update` against temporary projects. A repository-scope check will fail if retired top-level product paths or archived planning history return.

### Keep GitHub preparation minimal

Provide a product-focused README, Apache licensing files, contribution guidance, security-reporting instructions, and one CI workflow matching the clean-checkout verification. Avoid badges, issue templates, funding files, or organization-specific automation unless they provide immediate value.

## Risks / Trade-offs

- [A removed legacy file is an undocumented build input] → Search manifests/source references first and require clean-export build, test, init, update, and offline asset checks before accepting deletion.
- [Generated assets make the repository larger] → Retain only assets embedded into the self-contained runtime and document how each is regenerated or licensed.
- [The bundled model or vendor code has attribution obligations] → Inventory direct dependencies and bundled artifacts, preserve upstream notices, and review third-party attribution before public release.
- [Apache licensing is applied to code Cybling Labs does not own] → Scope the project license to original work and explicitly preserve third-party terms instead of adding ownership headers to generated/vendor files.
- [Removing OpenSpec history loses useful rationale] → Keep current normative specs and rely on Git history for prior proposals and designs.
- [Root sample wrappers are mistaken for release launchers] → Verify the actual wrappers are generated by Rust during `init`/`update`, then test both commands after removing root samples.
- [Large deletions make review difficult] → Separate inventory/legal work, deletion, documentation, and verification into ordered task groups with a before/after manifest.

## Migration Plan

1. Capture the current tracked-file inventory, directory sizes, manifest references, and baseline test/build results.
2. Add licensing, attribution, public documentation, release scripts, and CI while the old tree still exists.
3. Remove only paths classified as retired and update remaining references/specs.
4. Run verification from a clean temporary export, including installation and update smoke tests.
5. Review the final tracked allowlist and license inventory before committing the cleanup.
6. If verification fails, restore the affected path from Git and amend the allowlist rather than weakening a test.

Rollback is a normal Git revert before publication. No installed user project or data migration is required.

## Open Questions

- Confirm the first public GitHub repository URL before adding clone commands or repository-specific links; use placeholders or relative links until then.
- Confirm the preferred security contact address for `SECURITY.md` before publication.
- A legal owner should perform final review of trademark wording and third-party notices before the repository is made public.
