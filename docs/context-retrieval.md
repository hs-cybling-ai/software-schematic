# Context retrieval contract

A future Codex tool opens the `.dgraph` file, resolves its sibling `.context.sqlite` path within the authorized workspace, and reads only the manifest's authoritative complete revision.

Queries can specify text, node/context/section IDs, an embedding with provider/model provenance, maximum graph hops, and a result limit. Results contain node/context/section identity, label, heading path, Markdown, capture revision, graph distance, embedding provenance, and lexical/semantic/graph/total score components. `semanticCoverage` is false when no stored vectors are compatible with the query vector.

Graph expansion is bounded and cycle-safe over stored topology. Ranking weights are semantic `0.55`, lexical `0.35`, and graph proximity `0.10`, with an exact-ID boost of `1.0`. Ties sort by node ID and section ID. Missing diagrams, paths outside the workspace, unsupported schema versions, incomplete revisions, incompatible vector dimensions, and configured-limit violations are actionable errors.
