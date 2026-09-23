# Markdown embedding headers

Software Schematic stores derived vectors in a reserved leading HTML comment so
documentation generators can emit Markdown and embeddings together without
editing BPMN or CMMN. The browser hides this header in rendered and source-edit
views; the Markdown body remains authoritative.

```text
<!-- software-schematic-embedding
{"schema":"ssw.embedding/v1","owner":"main.cmmn#diagram","bodyHash":"sha256:…","chunker":"ssw.markdown-chunks/v1","model":"all-MiniLM-L6-v2","dimensions":384,"chunks":[…]}
-->

# Authored documentation
```

The compact JSON uses camel-case field names. Each ordered chunk contains
`id` (`<contentHash>:<ordinal>`), `contentHash`, `ordinal`, `headingPath`, and a
finite numeric `vector`. `bodyHash` is SHA-256 over the exact UTF-8 body bytes
after the header separator. An element document uses an owner such as
`orders/main.bpmn#Task_1`; diagram documentation uses
`orders/main.bpmn#diagram`.

Consumers must validate schema, owner, body hash, chunker, model, dimensions,
chunk identities, and finite values. Unknown, stale, malformed, or oversized
headers are cache misses: graph startup continues with text retrieval and the
editor queues regeneration. Run `./ssw embeddings` to backfill a project before
starting its MCP. The command is idempotent for current headers.

The v1 format uses JSON numeric arrays for portability and transparent
validation. They are larger than encoded float payloads, but avoid endianness
and decoder ambiguity; the header has an 8 MiB limit, 4,096 chunks, and 4,096
dimensions per vector.
