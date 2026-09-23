## Context

See `proposal.md` for motivation. Today `load_schematic_graph_with_previous` creates an in-memory Grafeo database, configures the local model, performs a readiness embedding, chunks Markdown, embeds chunks, and only then publishes a snapshot. Prior work made save-triggered rebuilds asynchronous and reusable, but a cold MCP still has no reusable on-disk vectors.

Markdown ownership is already deterministic: a diagram uses adjacent `main.md`, while a node or edge uses `docs/<source-id>.md`. This makes the Markdown file a better persistence boundary than diagram XML. A generator can write documentation and its derivation together, and no BPMN/CMMN parser or cross-file merge is required. The editor must nevertheless keep the generated payload invisible and prevent background writes from losing newer authored text.

## Goals / Non-Goals

**Goals:**

- Make a cold MCP and graph text-searchable in a few seconds, independent of embedding-model startup and document count.
- Define one portable header contract usable by the SSW editor, CLI, and external Markdown generators.
- Preserve strict validation, deterministic chunking, source provenance, atomic graph publication, and last-known-good behavior.
- Separate authoritative Markdown-body identity from derived-vector readiness.

**Non-Goals:**

- Treating vectors as authored content or accepting them as implementation authority.
- Exposing a public MCP mutation or embedding-generation tool.
- Supporting arbitrary embedding formats or multiple vector profiles in one graph revision.
- Guaranteeing vector search at the instant a legacy or externally edited project first starts.

## Decisions

### 1. Store an opaque canonical SSW header in each Markdown file

The file begins with a reserved HTML-comment envelope followed by the authored Markdown body:

```text
<!-- software-schematic-embedding
{"schema":"ssw.embedding/v1", ... canonical JSON ...}
-->

# Authored documentation
```

The JSON uses deterministic key ordering and encoding and contains `owner`, `bodyHash`, `chunker`, `model`, `dimensions`, and ordered `chunks`; each chunk contains its deterministic identity, text hash, ordinal/heading metadata, and vector. The body hash covers the exact canonical authored-body bytes after the envelope separator, never the header. Size, chunk count, dimensions, numeric finiteness, JSON depth, model identity, and owner are bounded before allocation or index insertion.

The HTML-comment envelope is preferable to YAML front matter because authored Markdown may already use front matter and because the payload is explicitly derived rather than document metadata intended for general Markdown processors. Storing it in diagram extension elements was rejected: it couples a Markdown-only change to a BPMN/CMMN rewrite, introduces open-editor merge races, and forces external generators to understand diagram XML.

### 2. Present a body-only projection to users and body-oriented APIs

The server parses the reserved leading envelope and returns only the body to the existing editor read endpoint. Rendering, source editing, assistant context, hashing for authored revisions, and save requests operate on the body. A normal body save atomically preserves the file path but removes or marks the prior envelope stale before queuing derivation; the editor can therefore never accidentally round-trip generated vectors as visible source.

Low-level project tools may write a complete physical file containing the canonical envelope. The parser recognizes at most one reserved leading envelope. A malformed or misplaced envelope is not silently treated as trusted vectors: the body remains recoverable, vector readiness is degraded, and diagnostics identify the file.

### 3. Use content-addressed compare-and-swap publication

After a durable body save, a project-scoped latest-state worker receives the canonical document path, resolved owner, body hash, and generation number. It chunks and embeds outside the request path. Before publication it rereads the physical file, parses the current body, resolves the owner against the current diagram, and compares both identities. Only an exact match permits an atomic sibling-file replacement containing the new header plus the unchanged body.

The derived header write is tagged internally and does not recursively schedule another embedding job. It does notify the graph refresh coordinator so current vectors can be imported. Work is serialized per document and bounded globally; newer hashes supersede queued and completed older work.

### 4. Make graph construction a validator/importer, never a generator

Graph construction chunks the current body deterministically, builds all entities and the text index, and validates any envelope against the owner, body hash, chunk set, configured profile, dimensions, and limits. Matching vectors are imported into Grafeo. Missing, stale, unsupported, or corrupt envelopes are excluded with a structured readiness diagnostic and queued for repair when a derivation service is available.

The normal MCP path never configures the embedding model and never calls `embed_text`. It publishes the source-complete text-searchable graph first. A valid header notification builds a staging vector-capable snapshot/index and swaps it atomically; unrelated valid vectors are reused. Public search keeps its response shape but reports retrieval mode/readiness so callers can distinguish hybrid from text-only results.

This is preferable to making MCP startup asynchronously initialize the model: that still assigns generation work to every MCP process, duplicates computation, and makes readiness dependent on a query service remaining alive.

### 5. Separate source revision from embedding-set identity

The existing revision is split conceptually into `sourceRevision` over authoritative diagram and Markdown-body content and `embeddingRevision` over accepted header identities. Entity/chunk URNs remain derived from source content. Publishing a generated header for unchanged content changes only embedding readiness/revision, preventing background metadata from masquerading as an authored model change.

Refresh and editor status carry independent document, embedding, source-graph, and vector-index states. This also prevents a generated header write from creating an endless source-revision cycle.

### 6. Backfill without blocking startup

Opening the editor starts the same project-scoped derivation worker and scans reachable connected Markdown for missing/stale headers. A dedicated idempotent CLI backfill operation supports CI, migrations, and generators that want artifacts before MCP startup. MCP may request a private repair enqueue when a worker is reachable, but never performs the embedding itself and never exposes that operation as a public MCP tool.

For a project generated entirely by another tool, valid headers require no backfill. Invalid external vectors are never trusted merely because they are present.

## Risks / Trade-offs

- **[Embedding vectors make Markdown files and diffs large]** → Keep the envelope compact and deterministic, document it as generated metadata, bound its size, and consider a future binary/sidecar encoding only through a new schema version.
- **[Generic Markdown tools expose the comment in source mode]** → The SSW UI guarantees hiding it; document the envelope so other generators can preserve or replace it. HTML rendering naturally suppresses the comment.
- **[A background write races with authored text]** → Compare owner and body hash immediately before atomic replacement; stale results are discarded rather than merged.
- **[External tools strip the header]** → Treat this as a cache miss: publish text search promptly and regenerate asynchronously.
- **[Untrusted files contain huge or malicious vectors]** → Enforce byte, nesting, chunk, dimension, and finite-number limits before allocation/indexing; validate every identity field.
- **[Text-only results differ from hybrid ranking]** → Expose retrieval readiness/mode and atomically switch to hybrid search when current vectors arrive.
- **[Embedding runtime moves into the editor/server footprint]** → Load it lazily only in the worker; editor rendering and MCP startup remain independent of runtime availability.

## Migration Plan

1. Add envelope parsing/serialization and body-only file APIs while continuing to support headerless Markdown.
2. Teach graph construction to import validated vectors and publish text-only snapshots without synchronous generation; keep the old path behind a temporary development flag for equivalence testing.
3. Add the post-save worker, compare-and-swap publication, status model, and derived-write refresh notification.
4. Add project-open scanning and the idempotent CLI backfill path, then backfill fixtures and starter content where useful.
5. Remove the MCP critical-path model initialization and legacy synchronous generation after cold-start, retrieval-equivalence, corruption, and concurrency tests pass.

Rollback can ignore or strip recognized headers and re-enable the legacy generator. Because the authored body is independently parseable and headerless files remain valid, rollback does not require restoring document content.

## Resolved Implementation Parameters

- Headerless graph construction has a two-second regression ceiling in the focused cold-start test; representative release measurements may tighten that ceiling later.
- `ssw.embedding/v1` uses canonical JSON numeric arrays. They are larger than encoded floats but are endian-independent, generator-friendly, directly validated by existing JSON tooling, and bounded by the envelope resource limits.
