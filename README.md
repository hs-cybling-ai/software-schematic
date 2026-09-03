# Software Schematic

Software Schematic is a lightweight, project-local documentation and modeling
tool for software development. It installs a self-contained browser editor,
CMMN business context, BPMN design diagrams, Markdown documentation, and a
query-only MCP graph beside the code that those documents describe.

Software Schematic is an open-source project owned by Cybling Labs, Inc.
Cybling Labs also makes the separate [cybling.ai](https://cybling.ai) platform;
using this project does not require cybling.ai.

## What it installs

Running `ss init` in an empty project creates:

```text
.ss/                 versioned runtime, browser assets, model, and notices
.codex/config.toml   project-scoped Software Schematic MCP registration
schematics/          CMMN/BPMN diagrams and Markdown documentation
AGENTS.md            managed model-driven development guidance
ssw / ssw.cmd        project-local macOS and Windows launchers
```

The installed application runs on an available loopback port. Browser code,
styles, fonts, diagram modelers, templates, and embeddings are local; no CDN or
package registry is needed at runtime. ONNX Runtime is required only for MCP
embeddings, not for the diagram editor.

## Supported targets

- macOS 13 or newer on Apple Silicon and Intel
- Windows 10 or newer on x64

The installed executable does not require Rust, Node.js, or npm. Building from
source requires a current stable Rust toolchain, Node.js 24 or newer, and npm.
On macOS, install ONNX Runtime with `brew install onnxruntime` to run MCP tests
and the local MCP server. Windows users provide `onnxruntime.dll` beside the
installed `.ss\bin\ss.exe` for MCP use.

## Build and verify from source

Run the canonical verification workflow from the repository root:

```sh
./scripts/verify-release.sh
```

For a release candidate, verify an isolated export containing only tracked and
new candidate source files:

```sh
./scripts/verify-release.sh --clean-export
```

The workflow installs exact web dependencies from
`software-schematic-web/package-lock.json`, runs  web and Rust tests, rebuilds
the embedded production web bundle, builds the release CLI, validates OpenSpec,
checks repository scope and required assets/notices, and smoke-tests `init` and
non-destructive `update` in temporary projects.

Individual development commands are:

```sh
npm ci --prefix software-schematic-web
npm test --prefix software-schematic-web
npm run build --prefix software-schematic-web
cargo fmt --manifest-path software-schematic-cli/Cargo.toml --check
cargo test --manifest-path software-schematic-cli/Cargo.toml
cargo build --release --manifest-path software-schematic-cli/Cargo.toml
openspec validate --all
```

The web build writes hashed production files to
`software-schematic-cli/assets/web/`. Those generated assets are committed
because the Rust release embeds them to produce an offline executable.

## Install into a project

After building, initialize a new project:

```sh
./software-schematic-cli/target/release/ss init /path/to/project
cd /path/to/project
./ssw
```

On Windows, use the matching `ss.exe` and run `ssw.cmd`. Initialization refuses
to overwrite existing `.ss`, `schematics`, or wrapper paths.

To refresh an existing compatible installation while preserving authored
diagrams, Markdown, unrelated Codex configuration, and unrelated `AGENTS.md`
content:

```sh
./ssw update
```

## Modeling workflow

`schematics/main.cmmn` is the project anchor for domain context, actors, needs,
business services, and links to BPMN designs. Named CMMN Process Tasks open
reusable BPMN compositions. Every supported node or edge can carry a stable
ID, label, architectural Name, Implementation Status, and ID-bound Markdown.

The browser editor saves complete CMMN/BPMN XML and Markdown automatically. A
successful save asks a running project MCP to rebuild the complete graph rooted
at `main.cmmn`. Unreachable diagrams are excluded, and stale connections to
deleted elements are skipped with warnings rather than blocking the refresh.

See [AI diagram assistance](docs/ai-diagram-assistant.md),
[scoped process naming](docs/scoped-process-naming.md), and
[project-local schematic MCP](docs/software-schematic-mcp.md) for detailed use.

## Codex and MCP

Initialization adds a project-scoped MCP entry. Trust or activate the
repository configuration in Codex and start a fresh task if registration has
just changed. Verify the project identity with `get_project_model` before using
graph context for development.

The MCP tools are query-only. Human-reviewed diagrams and documentation remain
the source of truth, and only elements marked `new` or `modify` authorize
implementation scope.

## Project layout

- `software-schematic-cli/` — Rust CLI, server, graph/MCP, tests, templates,
  embedded production bundle, and model assets.
- `software-schematic-web/` — browser editor source, tests, and pinned npm graph.
- `docs/` — focused operating and architecture documentation.
- `openspec/` — current normative specifications and active change artifacts.
- `scripts/` — release verification entry point.

The retained-file rationale and publication gates are documented in
[repository scope](docs/repository-scope.md).

## Contributing and security

See [CONTRIBUTING.md](CONTRIBUTING.md) before changing source, generated assets,
dependencies, or specifications. Report vulnerabilities using the private
process in [SECURITY.md](SECURITY.md), not a public issue.

## License

Original Software Schematic work is copyright 2026 Cybling Labs, Inc. and is
licensed under the [Apache License 2.0](LICENSE). Third-party components remain
under their own terms; see [NOTICE](NOTICE) and
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

The Apache License does not grant rights to Cybling Labs, Cybling,
Software Schematic, or cybling.ai trademarks beyond customary attribution.
