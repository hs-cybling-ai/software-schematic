# Diagram Studio

Diagram Studio is a sandboxed native macOS 14+ workspace focused on Data Graph diagrams, with BPMN retained as the interaction benchmark and a supported format. The application shell is SwiftUI/AppKit; each retained editor tab hosts a fully local `WKWebView` bundle.

## Developer setup

Requirements: macOS 14+, full Xcode selected with `xcode-select`, Node.js 24 LTS or newer, npm, and XcodeGen.

```sh
cd web-editor
npm ci
npm run verify
cd ..
xcodegen generate
xcodebuild -project DiagramStudio.xcodeproj -scheme DiagramStudio test
```

The Xcode target runs `scripts/build-web-assets.sh` before compilation. Production editor assets are copied into the app bundle; the runtime does not use a CDN or network service.

## Supported files

- BPMN 2.0 XML: `.bpmn`, `.bpmn20.xml`
- Diagram Studio Data Graph: `.dgraph`, paired with `.context.sqlite` when node context is captured

Data Graph diagrams are saved as a normalized ontology property graph containing `nodes`, `edges`, `properties`, and a `contextStore` manifest. Full-size `objectNode` circles represent object types. First-class, double-outline `edgeNode` circles define relationships with scalar (`0..1`), stack, queue, set, and map semantics. A `domain` edge connects `objectNode → edgeNode`; a `range` edge connects `edgeNode → objectNode`. Generated IDs use `<edge-node-id>_domain` and `<edge-node-id>_range_<target-node-id>`.

Selecting a node opens a right-side Markdown inspector with CodeMirror source editing and a sanitized preview. Capture embeds every changed Markdown section with Apple's local sentence-embedding model and stores the section text, SHA-256 hash, node metadata, provider/model provenance, and packed Float32 vector together in `name.context.sqlite`. Capture commits one complete SQLite revision before atomically advancing the manifest in `name.dgraph`; failures retain the prior authoritative revision. Move, back up, and restore the `.dgraph` and `.context.sqlite` files together.

SQLite context defaults are 1 MiB per Markdown section, 2,048 sections per capture, 16,384 vector dimensions, and 512 MiB per database. Unreferenced revisions after an interrupted manifest write are recoverable and can be compacted. A future Codex tool can use direct IDs, lexical matching, compatible vector similarity, and bounded graph-neighborhood expansion through the documented retrieval contract in `docs/context-retrieval.md`.

Objects offer two connection choices. **Relationship** ends at a scalar `edgeNode` when no target object is reached, or places that edge node midway between the source and target objects. **Object link** creates a direct dashed object-to-object edge with a semantic modifier. The initial modifiers are `sameAs` and `subclassOf`; both let the source resolve the target object's existing relationships without copying them into the stored graph. Resolution is deterministic and cycle-safe.

Typed property definitions are stored in `properties` with an `ownerId` referencing a node or topology edge. Their datatype marks are compact and explicit (`STR`, `INT`, `BYTE`, `URI`, and the catalog below) rather than color-coded. URI properties define identifiers or resource references such as an object `id` or `companyUrn`; URNs are represented by the broader URI datatype.

Functions are first-class `mutationNode` nodes. A `modifier` edge connects a mutation node to either an `edgeNode`, meaning the function applies to that relationship in the context of its domain object, or directly to an `objectNode`, meaning it operates on that object type. Query functions that return collection items use `←ƒ`, functions that update the collection use `ƒ→`, and functions that modify internal domain-object state use `ƒ`. Mutation parameters are properties owned by the mutation node—for example, `add(companyUrn)` or `searchCompanies(name)`. Runtime mutation classes, collections, lookup behavior, and dynamic binding are future implementation concerns rather than part of the diagram schema.

Property ownership is rendered as a short-dash line without an arrow or edge label. Object links use a longer dashed pattern and a directional arrow; `subclassOf` adds a solid circle at the source, while `sameAs` has no source marker. Mutation modifier edges are solid and end in a black circle. Domain and range edges retain standard directional arrows.

Select a property, mutation, edge node, or object link and use its wrench action to choose a type or link modifier from the labeled menu. Objects, properties, mutations, and edge nodes can be dragged freely and persist independently. Select an edge node to connect or create target objects or to attach a property or mutation.

Supported property datatypes are String (`STR`), Integer (`INT`), Byte (`BYTE`), Boolean (`BOOL`), Decimal (`DEC`), Float (`FLT`), Long (`LONG`), Date (`DATE`), Time (`TIME`), Date and time (`DT`), UUID (`UUID`), URI (`URI`), JSON (`JSON`), and Binary (`BIN`). Intermediate icons use three horizontal lines for stack, three vertical lines for queue, `{}` for set, and `{:}` for map.

The Data Graph document definition is still under development. `.dgraph.json` is no longer canonical; the workspace service provides explicit migration to `.dgraph`. Older filter/color-based development documents remain unsupported.

Files are limited to 20 MB. Symlinks resolving outside the selected folder are excluded. Saves use atomic replacement and prompt before overwriting an externally changed source.

## Commands

- Open Folder: ⇧⌘O
- Save: ⌘S
- Save All: ⌥⌘S
- Close Tab: ⌘W

## Updating diagram dependencies

Query stable releases, update exact versions in `web-editor/package.json`, run `npm install`, then run `npm audit` and `npm run verify`. Commit the resulting lockfile only if both real compatibility fixtures import/export successfully. See [docs/compatibility.md](docs/compatibility.md) for the current split dependency graph and known ArchiMate workarounds.

## Manual acceptance

1. Launch in dark mode and open a folder containing all three fixture formats.
2. Confirm nested supported files appear and unsupported files do not.
3. Open several diagrams, switch tabs, make edits, and verify dirty indicators persist.
4. Save each format, close it, reopen it, and verify the edit remains.
5. Modify an open source externally and verify Overwrite/Reload/Cancel appears.
6. Exercise Save/Don’t Save/Cancel while closing a dirty tab, replacing the folder, and quitting.
7. Disconnect networking and confirm all editors and fonts/icons still load.
8. Select each node kind, edit and preview Markdown, Capture, close and reopen the diagram, and confirm the paired SQLite content is restored.
