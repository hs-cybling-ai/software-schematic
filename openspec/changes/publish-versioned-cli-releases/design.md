## Context

The repository currently verifies source on macOS and Windows but does not publish installable binaries. Users must install Rust and Node.js even though the product is designed to run as a self-contained executable. The CLI embeds a roughly 90 MB model and a generated browser bundle, so release packaging must preserve large runtime inputs, notices, and target-specific executable behavior.

GitHub Releases provide stable, version-addressed downloads and generated release notes. GitHub Actions can build on native hosted runners and generate artifact attestations, while the existing `scripts/verify-release.sh --clean-export` remains the canonical source and behavior gate.

## Goals / Non-Goals

**Goals:**

- Publish trustworthy CLI archives for every supported macOS and Windows target from semantic version tags.
- Make the release version unambiguous and consistent with Cargo metadata.
- Reuse existing verification and build behavior instead of creating a divergent release-only test path.
- Give users stable latest-release and pinned-version download paths, checksums, provenance, installation instructions, and a source-build fallback.
- Keep release credentials and permissions minimal and prevent partial releases.

**Non-Goals:**

- Publishing a Rust library or CLI package to crates.io, Homebrew, WinGet, Chocolatey, npm, or a container registry.
- Adding Linux support or promising targets that the project cannot execute and test natively.
- Automating version selection, changelog commits, tag creation, code signing, Apple notarization, or Windows Authenticode in the first release workflow.
- Replacing pull-request CI or committing generated release archives to Git.

## Decisions

### Release only immutable semantic version tags

The workflow will trigger on pushed tags matching `v[0-9]+.[0-9]+.[0-9]+`, with an optional documented prerelease suffix such as `-rc.1`. A validation step will parse the tag and require its core version to exactly match `package.version` in `software-schematic-cli/Cargo.toml`; malformed or mismatched tags fail before packaging. The workflow will build the exact tagged commit and never move a published version tag.

This is preferred over releasing every `main` build because a public download should identify an intentional, reproducible version. Manual dispatch may exercise packaging in dry-run mode but will not publish an untagged release.

### Build supported targets on native GitHub runners

A matrix will build `aarch64-apple-darwin`, `x86_64-apple-darwin`, and `x86_64-pc-windows-msvc` using compatible native hosted runners. Each matrix entry will run the relevant verification, compile with Cargo's release profile for its explicit target triple, smoke-test executable behavior where runnable, and stage one archive.

Native builds are preferred to unsupported cross-compilation for Windows MSVC and reduce linker/runtime surprises. Linux and Windows ARM are excluded until their runtime and ONNX requirements are specified and tested.

### Publish self-describing archives rather than bare executables

macOS artifacts will use `.tar.gz`; Windows will use `.zip`. Names will include the project, semantic version, and Rust target triple, for example `software-schematic-v0.2.0-aarch64-apple-darwin.tar.gz`. Each archive will contain the target executable plus `LICENSE`, `NOTICE`, and `THIRD_PARTY_NOTICES.md`. The binary continues to install its embedded browser, model, templates, and notices through `ss init`.

A release manifest will enumerate filenames, versions, target triples, and SHA-256 digests. A combined checksum file will allow conventional offline verification.

### Separate unprivileged builds from privileged publication

Matrix jobs will use read-only repository permissions and upload short-lived workflow artifacts. A single dependent publication job will download all expected artifacts, verify their exact names, versions, checksums, archive contents, and executable smoke tests, then generate build-provenance attestations and create the GitHub Release.

Only the publication job receives `contents: write`, `id-token: write`, and the attestation permissions required by GitHub. It will use GitHub-maintained actions pinned to immutable commit SHAs where practical and the preinstalled GitHub CLI/API rather than a release-specific third-party action.

### Fail atomically and make reruns safe

No GitHub Release is created until all matrix jobs succeed and the publication job validates the complete artifact set. If a release with the tag already exists, the workflow fails rather than replacing assets. Maintainers correct the source/version, delete an unpublished erroneous tag if appropriate, and create a new version; published assets are never silently overwritten.

Prerelease suffixes produce a GitHub prerelease. Stable semantic versions become normal releases, and GitHub determines the latest stable release according to semantic versioning. Generated release notes cover changes between tags.

### Put downloads in the primary onboarding path

The README will link to the repository's `/releases/latest` page and describe selecting a target archive, verifying SHA-256, extracting it, and running `ss init`. It will also link to the full releases list for pinned versions and retain source-build instructions. A maintainer section will document version bump, verification, annotated tag, push, workflow observation, artifact inspection, and rollback/correction procedures.

## Risks / Trade-offs

- **[Large embedded model increases build and artifact transfer time]** → Cache only dependency inputs, use short artifact retention, and verify the final archive size and contents.
- **[Hosted runner availability or architecture labels change]** → Express target triples explicitly, document the supported matrix, and update runner labels without changing artifact contracts.
- **[ONNX Runtime differs by operating system]** → Run native smoke tests where dependencies are available, preserve current platform guidance, and do not claim unsupported bundled native libraries.
- **[A compromised workflow could publish binaries]** → Trigger only from repository tags, protect tag creation administratively, pin actions, minimize permissions, generate attestations, and publish only after complete validation.
- **[Tag and Cargo versions drift]** → Fail before any release is created and document the version-bump order.
- **[Unsigned binaries produce operating-system warnings]** → State the limitation clearly; checksums and attestations establish integrity but do not replace platform signing.

## Migration Plan

1. Add packaging/version validation and tests without changing existing CI behavior.
2. Add the release workflow and validate its YAML plus a non-publishing packaging run.
3. Update public and maintainer documentation with repository-specific download URLs.
4. Bump Cargo to the intended first public version, commit it, and create an annotated semantic tag after `main` verification passes.
5. Inspect every draft artifact, checksum, attestation, and generated note before treating the first release as supported.
6. If publication fails, leave the tag diagnosable, publish no partial release, fix forward with a new patch version, and preserve previously published assets.

## Open Questions

- Whether Cybling Labs will enable GitHub immutable releases and tag protection before the first stable release.
- Whether a later change will add Apple signing/notarization, Windows Authenticode, Homebrew, or WinGet distribution.
