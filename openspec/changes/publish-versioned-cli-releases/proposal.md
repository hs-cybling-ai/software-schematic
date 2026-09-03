## Why

Software Schematic can be built from source, but external users do not yet have a trusted, versioned way to download a ready-to-run CLI. Automated GitHub Releases will make supported binaries easy to install while preserving the verification, traceability, and least-privilege practices expected of an open-source project.

## What Changes

- Add a GitHub Actions release workflow that builds the Rust CLI on native supported runners when an explicit semantic-version tag is pushed.
- Require the Git tag, Cargo package version, and embedded application version to agree before publishing.
- Run the canonical verification workflow before packaging platform-native executables.
- Package clearly named macOS and Windows archives with the applicable license and notice files.
- Generate SHA-256 checksums and GitHub artifact attestations for downloadable archives.
- Publish a GitHub Release with generated notes only after every required platform artifact succeeds, while supporting prerelease tags without treating them as stable.
- Add README links and commands for downloading, verifying, and installing the latest or a pinned release.
- Document the maintainer release procedure, supported artifact matrix, version policy, permissions, and failure/retry behavior.

## Capabilities

### New Capabilities

- `versioned-cli-releases`: Defines semantic-version release triggers, native Rust builds, artifact naming and contents, integrity/provenance metadata, atomic publication, and release failure behavior.

### Modified Capabilities

- `open-source-project-distribution`: Requires public documentation to link directly to GitHub Releases and explain download, verification, installation, supported platforms, and source-build alternatives.

## Impact

- Adds a release workflow under `.github/workflows/` and may add a small packaging/version-check script under `scripts/`.
- Extends CI permissions only in the release job to create releases and provenance attestations; pull-request verification remains read-only.
- Establishes `vMAJOR.MINOR.PATCH` tags as the stable release interface and the Rust package version as the version source checked at publication time.
- Produces GitHub Release archives for the currently supported macOS and Windows targets without publishing to crates.io or another package registry.
- Updates the README, contribution/release documentation, OpenSpec contracts, and verification tests.
