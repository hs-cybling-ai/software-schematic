## 1. Establish Release Metadata and Packaging

- [x] 1.1 Define the supported release matrix with native runner labels, Rust target triples, archive formats, executable names, and current ONNX Runtime test expectations for Apple Silicon macOS, Intel macOS, and Windows x64.
- [x] 1.2 Add a release helper that parses stable and prerelease tags, reads the Cargo package version, rejects mismatches, and exposes normalized version metadata without creating or moving tags.
- [x] 1.3 Add deterministic packaging commands that stage only the target executable plus `LICENSE`, `NOTICE`, and `THIRD_PARTY_NOTICES.md` under versioned target-specific archive names.
- [x] 1.4 Generate a machine-readable release manifest and conventional SHA-256 checksum file covering every required archive.
- [x] 1.5 Add automated tests for valid, malformed, mismatched, and prerelease versions; expected filenames and archive contents; checksum verification; and refusal to overwrite existing output.

## 2. Build Supported Native Artifacts

- [x] 2.1 Add a separate GitHub Actions release workflow triggered by semantic-version tags and a non-publishing manual validation mode.
- [x] 2.2 Configure native matrix jobs for `aarch64-apple-darwin`, `x86_64-apple-darwin`, and `x86_64-pc-windows-msvc`, using stable Rust, locked dependencies, explicit targets, and dependency caching that excludes final artifacts.
- [x] 2.3 Reuse the canonical clean-export verification before release compilation and run target-native executable/version/init smoke checks wherever the hosted runner can execute the artifact.
- [x] 2.4 Upload each verified archive and its target metadata as short-lived workflow artifacts using GitHub-maintained actions pinned to reviewed immutable commit SHAs.
- [x] 2.5 Ensure build and pull-request contexts retain read-only permissions and cannot publish a release or access unnecessary credentials.

## 3. Publish Atomically and Securely

- [x] 3.1 Add one publication job that waits for all matrix jobs, downloads the complete expected artifact set, rejects missing or unexpected files, revalidates archive contents, and verifies all checksums.
- [x] 3.2 Configure least-privilege `contents`, identity-token, and attestation permissions only on the publication job and generate GitHub build-provenance attestations for every archive.
- [x] 3.3 Create the GitHub Release with generated notes and all archives, manifest, and checksums only after final validation, marking recognized prerelease suffixes as prereleases.
- [x] 3.4 Fail without replacing assets when the tag already has a release, and guarantee that failed or incomplete matrix runs create no partial public release.
- [x] 3.5 Add workflow concurrency keyed by release tag and validate all action references, expressions, permissions, shell behavior, and artifact-retention settings.

## 4. Document Downloads and Maintainer Operations

- [x] 4.1 Add prominent README links to the latest release and complete release list for `hs-cybling-ai/software-schematic`, with a target-selection table for supported macOS and Windows downloads.
- [x] 4.2 Document checksum and GitHub attestation verification, extraction, executable invocation, `ss init`, platform security warnings, ONNX Runtime requirements, and the source-build fallback.
- [x] 4.3 Add a maintainer release guide covering semantic version selection, Cargo version update, clean verification, annotated tag creation, push, workflow monitoring, artifact inspection, and fix-forward recovery.
- [x] 4.4 Document that releases are unsigned initially, record immutable-release and tag-protection recommendations, and preserve signing/notarization as explicit future decisions.
- [x] 4.5 Verify every new public link, command, filename, version example, target claim, and permission statement from a clean tracked export.

## 5. Prove the Release Workflow

- [x] 5.1 Extend repository verification to test release metadata and packaging without requiring release-write permissions or creating GitHub resources.
- [x] 5.2 Run existing web tests/build, Rust formatting/tests/release build, OpenSpec validation, and the new packaging tests on macOS.
- [ ] 5.3 Run the release workflow in manual non-publishing mode for every available hosted target and confirm the expected temporary archives, legal notices, checksums, manifest, and executable smoke evidence.
- [x] 5.4 Exercise failure cases for tag/version mismatch, a failed matrix target, incomplete artifact sets, and an existing release, confirming no public asset is created or overwritten.
- [ ] 5.5 Create a disposable prerelease tag for an end-to-end owner-approved test, inspect downloads and provenance, then retain or remove that prerelease according to the documented release procedure.
