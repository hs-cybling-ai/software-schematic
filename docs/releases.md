# Publishing Software Schematic releases

Software Schematic publishes self-contained CLI archives through
[GitHub Releases](https://github.com/hs-cybling-ai/software-schematic/releases).
The workflow builds three native targets:

| Runner | Rust target | Archive | Native ONNX verification |
| --- | --- | --- | --- |
| `macos-15` (Apple Silicon) | `aarch64-apple-darwin` | `.tar.gz` | ONNX Runtime installed with Homebrew; full test suite |
| `macos-15-intel` | `x86_64-apple-darwin` | `.tar.gz` | ONNX Runtime installed with Homebrew; full test suite |
| `windows-2022` | `x86_64-pc-windows-msvc` | `.zip` | Packaged-model load test skipped until the workflow provides `onnxruntime.dll`; remaining tests and initialization smoke test run |

Linux and Windows ARM are not release targets. Intel macOS runner availability
is currently expected through August 2027 and must be reassessed before then.

## Version policy

Stable releases use immutable `vMAJOR.MINOR.PATCH` tags. Prereleases may append
`-alpha.N`, `-beta.N`, or `-rc.N`. The numeric core must exactly match the
`version` in `software-schematic-cli/Cargo.toml`; the workflow fails before
packaging if it does not.

Do not move, reuse, or force-push a published version tag. Release assets are
also immutable by policy. Enable GitHub immutable releases and rulesets that
restrict release-tag creation when the repository owner is ready.

## Validate without publishing

The `Release CLI` workflow supports `workflow_dispatch`. A manual run accepts
an optional release ref and builds, packages, and assembles temporary workflow
artifacts, but its publication job is disabled. Locally, run:

```sh
node --test scripts/release.test.mjs
./scripts/verify-release.sh --clean-export
```

The release helper can also validate a matching tag without changing Git:

```sh
node scripts/release.mjs metadata --ref v0.1.7
```

## Publish a version

1. Choose the next semantic version and update `software-schematic-cli/Cargo.toml`.
2. Regenerate `software-schematic-cli/Cargo.lock` if Cargo changes it.
3. Run `./scripts/verify-release.sh --clean-export` from a clean checkout.
4. Merge the reviewed version change into protected `main` and wait for normal CI.
5. Create and push an annotated tag on the verified commit:

   ```sh
   git tag -a v0.2.0 -m "Software Schematic v0.2.0"
   git push origin v0.2.0
   ```

6. Observe every native build and the final assembly and publication jobs.
7. Inspect the release page, all three archives, `SHA256SUMS`,
   `release-manifest.json`, generated notes, and GitHub attestations.
8. Download one archive independently, verify its checksum and provenance, and
   run `ss --version` plus `ss init` before announcing the release.

The tag workflow receives read-only permissions during builds. Only the final
tag-only publication job receives the minimum release and attestation writes.
It refuses to replace an existing release and creates nothing when any required
target or final validation fails.

## Failure and correction

For a failed unpublished tag, diagnose the retained workflow logs. If policy
allows deleting an erroneous unpublished tag, delete it before recreating the
correct tag; otherwise increment the patch version. Never overwrite a
published release or asset. Correct published defects with a new patch release
and explain the superseded version in release notes.

Release archives are initially unsigned and are not Apple-notarized or Windows
Authenticode-signed. SHA-256 checksums and GitHub build provenance are required,
but they do not replace signing. Adding signing, notarization, Homebrew, WinGet,
or another distribution channel requires a separate reviewed change.
