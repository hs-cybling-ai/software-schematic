# Diagram dependency compatibility

| Package | Locked version | Role |
| --- | ---: | --- |
| `bpmn-js` | 18.22.1 | BPMN 2.0 modeler |
| `diagram-js` | 15.23.2 | Current top-level diagram toolkit |
| `@codemirror/state` | 6.5.2 | Markdown editor state |
| `@codemirror/view` | 6.38.1 | Markdown editor UI |
| `@codemirror/lang-markdown` | 6.3.4 | Markdown language support |
| `markdown-it` | 14.1.0 | Preview rendering with raw HTML disabled |
| `dompurify` | 3.2.6 | Preview sanitization |
| `@noble/hashes` | 1.8.0 | Browser-side SHA-256 section hashes |

The Data Graph editor targets the same current `diagram-js` generation as BPMN. Its `.dgraph` contract is validated before a live session is replaced and exported with deterministic key and ID ordering. Version 2 references a sibling `.context.sqlite` database; vector payloads remain outside topology JSON. Property, mutation, and intermediate subtype values are registry-validated. Former `.dgraph.json` files are migration inputs rather than the canonical extension.

The BPMN and Data Graph adapters share the current diagram toolkit while keeping their notation modules separate. `npm ci` reproduces the graph. Any dependency update must pass `npm run test:compat`.

Native context persistence links the SQLite library from the macOS SDK. Context files use relational schema version 1 and packed Float32 embeddings; see `docs/context-storage.md`. The context database is intentionally a standard SQLite file so independent tools, including a future Codex integration, can inspect it without the Diagram Studio process.
