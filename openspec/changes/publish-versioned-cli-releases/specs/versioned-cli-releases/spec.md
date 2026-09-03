## ADDED Requirements

### Requirement: Semantic release identity
The project SHALL publish CLI releases only from explicit Git tags whose stable form is `vMAJOR.MINOR.PATCH`. The release workflow SHALL verify that the tag version exactly matches the Rust package version before compiling or publishing, SHALL build the tagged commit, and SHALL NOT replace or move a previously published version.

#### Scenario: Stable version tag is valid
- **WHEN** a maintainer pushes a `vMAJOR.MINOR.PATCH` tag whose version matches `software-schematic-cli/Cargo.toml`
- **THEN** the workflow accepts the tagged commit as a release candidate

#### Scenario: Tag and package versions disagree
- **WHEN** the tag version is malformed or does not match the Rust package version
- **THEN** the workflow fails before creating a GitHub Release or uploading public assets

### Requirement: Supported native release matrix
The release workflow SHALL build optimized CLI executables for `aarch64-apple-darwin`, `x86_64-apple-darwin`, and `x86_64-pc-windows-msvc` on compatible native GitHub-hosted runners. Each target SHALL pass the applicable canonical verification and executable smoke checks before its artifact is accepted.

#### Scenario: Every supported target succeeds
- **WHEN** a valid release candidate runs successfully on every matrix target
- **THEN** the workflow produces one verified target archive for each supported target

#### Scenario: One target fails
- **WHEN** compilation, verification, packaging, or smoke testing fails for any required target
- **THEN** the workflow publishes no GitHub Release for that tag

### Requirement: Self-describing release packages
Each release archive SHALL have a deterministic name containing the product, version, and Rust target triple. It SHALL contain only the target executable and applicable `LICENSE`, `NOTICE`, and `THIRD_PARTY_NOTICES.md` files, while the executable SHALL retain all assets embedded for project initialization.

#### Scenario: User inspects an archive
- **WHEN** a user downloads and extracts a supported target archive
- **THEN** the executable and required legal notices are present under predictable names without source code or build output

### Requirement: Release integrity and provenance
Every published release SHALL include SHA-256 checksums for all downloadable archives and GitHub build-provenance attestations generated from the release workflow. The workflow SHALL verify archive names, contents, version identity, and checksums immediately before publication.

#### Scenario: User verifies a download
- **WHEN** a user obtains an archive and the published checksum manifest
- **THEN** the user can verify the archive's SHA-256 digest and identify its GitHub Actions build provenance

### Requirement: Least-privilege atomic publication
Build jobs SHALL use read-only repository permissions. A single publication job SHALL receive only the permissions required to attest artifacts and create a release, SHALL run only after all required build jobs succeed, and SHALL refuse to overwrite an existing release or release asset.

#### Scenario: Complete release candidate is published
- **WHEN** every required artifact and final validation succeeds for a new tag
- **THEN** the publication job creates one GitHub Release with all archives, checksums, provenance, and generated release notes

#### Scenario: Existing release is detected
- **WHEN** the tag already has a GitHub Release
- **THEN** the workflow fails without replacing its published assets

### Requirement: Stable and prerelease behavior
Stable semantic tags SHALL create normal GitHub Releases, and documented prerelease suffixes SHALL create GitHub prereleases. Untagged manual or branch workflow executions SHALL NOT publish a release.

#### Scenario: Release candidate tag is published
- **WHEN** a valid version such as `v1.2.0-rc.1` completes the workflow
- **THEN** its artifacts are published as a prerelease and are not presented as the latest stable release

#### Scenario: Manual packaging validation runs
- **WHEN** a maintainer runs an allowed manual packaging validation without a release tag
- **THEN** the workflow may produce temporary workflow artifacts but creates no tag or GitHub Release

### Requirement: Maintainer release procedure
The repository SHALL document version selection, Cargo version update, canonical verification, annotated tag creation, workflow observation, artifact and checksum inspection, provenance verification, and fix-forward recovery. The procedure SHALL identify signing and immutable-release controls that remain owner decisions.

#### Scenario: Maintainer prepares a release
- **WHEN** a maintainer follows the documented procedure from a clean checkout
- **THEN** the tag, package version, verification evidence, artifacts, and GitHub Release remain consistent and auditable
