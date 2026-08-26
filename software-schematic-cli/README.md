# Software Schematic CLI

`ss` installs a versioned, project-local BPMN and Markdown modeling workspace. It is designed for model-driven AI development: diagrams and documentation live beside the source code in formats that people and coding agents can inspect directly.

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
│   ├── version
│   └── web/            # complete offline application and vendor assets
├── schematics/
│   ├── main.bpmn
│   └── main.md
├── ssw
└── ssw.cmd
```

All browser code, CSS, BPMN fonts, icons, `bpmn-js`, Markdown rendering, sanitization, and Mermaid rendering are packaged in `.ss/web/`. The running application makes no CDN or package-registry requests and requires no Node.js installation.

## Composition conventions

- `schematics/main.bpmn` and `schematics/main.md` describe the root composition.
- Every nested composition folder has a `main.bpmn` and `main.md`.
- Element documentation is stored beside its diagram as `docs/<element-id>.md`.
- A BPMN call activity is an external subprocess. Its standard `calledElement` value is a normalized path relative to `schematics/`, such as `order` or `sales/quote`.
- A BPMN subprocess is also composable: its label becomes a lowercase filesystem-safe folder name, so `Birth` resolves to `birth` and `Order Fulfillment` resolves to `order-fulfillment`.
- Double-click a call activity, or use **Open composition**, to create and focus its base diagram.
- Pools map to `<parent>/<pool-id>/`; lanes map to `<parent>/<pool-id>/<lane-id>/`.
- Multiple call activities may reference the same folder and therefore reuse the same process model.

The web editor automatically saves BPMN XML and Markdown. The Rust server accepts only typed operations and confines paths to `schematics/`.

## Development checks

```sh
cargo test --manifest-path software-schematic-cli/Cargo.toml
npm test --prefix software-schematic-web
npm run build --prefix software-schematic-web
```

## Deferred scope

MCP exposure, Graph DB indexing, LLM graph queries, recovery for missing or externally modified files, collaboration, hosted serving, and code generation are intentionally deferred to later changes.
