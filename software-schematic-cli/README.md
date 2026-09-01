# Software Schematic CLI

`ss` installs a versioned, project-local CMMN, BPMN, and Markdown modeling workspace. It is designed for model-driven AI development: business needs, solution designs, and documentation live beside the source code in formats that people and coding agents can inspect directly.

## Supported targets

The first release matrix is:

- macOS 13 or newer on Apple Silicon (`aarch64-apple-darwin`)
- macOS 13 or newer on Intel (`x86_64-apple-darwin`)
- Windows 10 or newer on x64 (`x86_64-pc-windows-msvc`)

Each target receives one native, self-contained `ss` executable. Windows ARM64 is deferred.

## Build and initialize

```sh
cargo build --release --manifest-path software-schematic-cli/Cargo.toml
./software-schematic-cli/target/release/ss init /path/to/project
```

Initialization refuses to overwrite `.ss/`, `schematics/`, `ssw`, or `ssw.cmd` when any target already exists.

Launch the initialized project with:

```sh
./ssw
```

On Windows, run `ssw.cmd`. The wrapper starts a server on an available `127.0.0.1` port and opens the application in the default browser.

## Generated layout

```text
project/
├── .ss/
│   ├── bin/ss          # ss.exe on Windows
│   ├── starter.cmmn    # bundled template for root and named business anchors
│   ├── version
│   └── web/            # complete offline application and vendor assets
├── schematics/
│   ├── main.cmmn
│   └── main.md
├── ssw
└── ssw.cmd
```

All browser code, CSS, BPMN/CMMN fonts and icons, `bpmn-js`, `cmmn-js`, Markdown rendering, sanitization, and Mermaid rendering are packaged in `.ss/web/`. The running application makes no CDN or package-registry requests and requires no Node.js installation.

## Composition conventions

- `schematics/main.cmmn` and `schematics/main.md` are the sole project anchor. They define the domain, actors, inputs, outputs, needs, and business services.
- BPMN defines logical architecture and solution building blocks. What might previously have been a root BPMN is a named process opened from a CMMN Process Task.
- Every diagram element has ID, Type, Label, Name, Implementation Status, and Documentation. Name is independent from ID and Label.
- A reusable process Name uses `package.Process`. A node or edge Name uses `package.Process#nodeOrEdgeName`.
- Process Name `sales.checkout.PlaceOrder` maps directly to `schematics/sales/checkout/PlaceOrder/`, which contains `main.bpmn` and the shared implementation documentation in `main.md`.
- Node and edge documentation is bound to the occurrence ID at `docs/<element-id>.md`. Changing Name does not move that file; changing ID renames it.
- Multiple call activities with the same process Name open the same subprocess implementation while retaining separate ID-bound call-site documentation in their owning diagrams.
- Double-clicking a call activity or subprocess, or selecting its subprocess link icon, opens its named composition. If it is unnamed, the editor collects a valid `package.Process` Name in a modal before creating anything.
- `calledElement` is a serialized mirror of the complete process Name and is not a separate editable field.
- A CMMN business anchor uses a package Name: `cybling.sdk` maps to `schematics/cybling/sdk/main.cmmn` and its sibling `main.md`.
- A named Process Task such as `cybling.sdk.Birth` creates or opens the BPMN design at `schematics/cybling/sdk/Birth/main.bpmn`.
- Anchors are intentionally sparse. BPMN may still create nested package folders, and `cybling/sdk/main.cmmn` is not required merely because a BPMN design exists below that folder.
- CMMN nodes and connections reuse the same ID, Type, Label, Name, Implementation Status, per-ID Markdown, automatic save, and bounded magic-action conventions as BPMN.

The web editor automatically saves BPMN XML, CMMN XML, and Markdown. The Rust server accepts only typed operations and confines paths to `schematics/`.

## Development checks

```sh
cargo test --manifest-path software-schematic-cli/Cargo.toml
npm test --prefix software-schematic-web
npm run build --prefix software-schematic-web
```

## Deferred scope

MCP exposure, Graph DB indexing, LLM graph queries, recovery for missing or externally modified files, collaboration, hosted serving, and code generation are intentionally deferred to later changes.
