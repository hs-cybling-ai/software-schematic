# Contributing

Thank you for improving Software Schematic.

## Development workflow

1. Keep changes focused on the Rust CLI/project wrapper or its bundled browser
   editor and documentation.
2. Represent behavioral changes in the current OpenSpec capabilities. Use an
   active change when the work needs proposal, design, or migration context.
3. Add tests at the layer where behavior is owned.
4. Run `./scripts/verify-release.sh` before submitting a change.
5. Run `./scripts/verify-release.sh --clean-export` for release-affecting changes.

Changes to versioning, packaging, supported targets, or GitHub Release behavior
must also follow the test and review expectations in
[`docs/releases.md`](docs/releases.md). Never move or reuse a published version
tag.

## Generated and bundled assets

`npm run build --prefix software-schematic-web` replaces the hashed production
bundle in `software-schematic-cli/assets/web/`. Commit the complete replacement
with its source change. Do not hand-edit minified vendor files.

The MiniLM model and tokenizer are retained runtime inputs. Do not replace or
remove them without testing MCP initialization and updating their source,
transformation, checksum, and licensing notice.

## Dependency and license changes

Use exact lockfiles. When dependencies change, inspect changed license files and
metadata, update `THIRD_PARTY_NOTICES.md`, preserve required notices and bpmn.io
watermarks, and run the clean-export verification workflow.

Unless explicitly stated otherwise, intentionally submitted contributions are
provided under Apache License 2.0 as described in section 5 of `LICENSE`.
