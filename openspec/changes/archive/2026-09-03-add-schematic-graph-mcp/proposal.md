## Why

Software Schematic currently stores the architectural model in navigable CMMN, BPMN, and Markdown files, but an LLM must receive hand-built snapshots and cannot reliably discover the complete project structure. A fast, local property-graph view and a small MCP surface will let model-driven development reason from the schematic as the source of truth without introducing an enterprise indexing platform or durable synchronization machinery.

## What Changes

- Add a Rust graph loader that begins at `schematics/main.cmmn`, follows supported composition references within the schematic workspace, and builds a fresh in-memory Grafeo property graph.
- Represent every loaded diagram, diagram node, and diagram edge as an addressable graph entity, preserving stable source identifiers, types, labels, names, paths, topology, containment, and composition relationships.
- Persist each eligible element's Implementation Status in diagram XML and carry the exact `new`, `modify`, `locked`, or `open` value into Grafeo. Give every graph entity a stable project-scoped SSW URN derived from the semantic owner Name and element source ID; keep file locations only in loader provenance because the graph is the compiled schematic.
- Load diagram-level and element-level Markdown into the corresponding graph entities, split it into retrievable chunks, and use Grafeo's native embedding, vector-index, text-index, and hybrid-search capabilities for retrieval.
- Add an `ss mcp` service to the Software Schematic CLI that loads the graph at startup and exposes project overview, entity lookup, neighborhood traversal, hybrid search, and proposal-language-to-development-scope resolution.
- Fail startup with clear, actionable diagnostics for an invalid root, unreadable referenced content, malformed diagrams, or unavailable configured embedding support; avoid background synchronization, graph persistence, distributed coordination, and elaborate compensation flows.
- Install project-local Codex MCP configuration that launches the pinned project `ssw mcp` wrapper for that repository, and make the MCP handshake identify the loaded project and root schematic.
- Update project bootstrap and migration behavior so `AGENTS.md` directs agents to keep detailed contracts in diagrams, create lightweight OpenSpec-style proposals, resolve natural proposal language to a green `new` or orange `modify` root through the project MCP, link tasks to diagram nodes, and implement only the returned green/orange scope.

## Capabilities

### New Capabilities

- `schematic-property-graph`: Loading a complete project schematic into an in-memory property graph, including entities, topology, compositions, Markdown, and embeddings.
- `schematic-mcp`: Serving the loaded schematic graph through a small MCP interface designed for LLM discovery, lookup, traversal, and retrieval.

### Modified Capabilities

- `software-schematic-bootstrap`: Bootstrap and migration shall install managed project-agent guidance for using the schematic MCP during software development.
- `software-schematic-editor`: Implementation Status shall become durable diagram metadata so the graph can enforce diagram-authored development scope.

## Impact

- Affects the Rust `software-schematic-cli` crate, especially project discovery, schematic parsing, configuration, startup diagnostics, and a new MCP command/service.
- Adds Rust dependencies for Grafeo's embedded property graph and native AI/embedding features, MCP transport/protocol support, XML parsing, and Markdown chunking.
- Adds no external database, graph daemon, durable index, browser UI, authentication layer, or enterprise lifecycle service.
- Extends SSW diagram metadata with persisted Implementation Status and introduces a project-local MCP process, managed `.codex/config.toml` entry, and managed `AGENTS.md` block; diagrams and Markdown remain the source of truth.
