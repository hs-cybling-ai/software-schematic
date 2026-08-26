# Data Graph context storage

Each `name.dgraph` is paired with `name.context.sqlite`. Diagram topology remains readable JSON; Markdown sections and Float32 embeddings are captured together in SQLite.

The app uses the SQLite library supplied by the macOS SDK (`libsqlite3`) through the `CSQLite` Swift system-library target. Schema version 1 contains:

- `metadata`: `schema_version` and the stable `diagram_id`.
- `capture_revisions`: monotonically increasing revision, timestamp, and completion marker.
- `context_sections`: diagram/node/context/section identity, heading path JSON, ordinal, Markdown, SHA-256 content hash, packed native-endian Float32 embedding blob, provider/model/dimensions, timestamp, revision, and tombstone marker.

A capture runs under `BEGIN IMMEDIATE`. Its revision is incomplete until all validated rows are inserted. The transaction then sets `is_complete = 1` and commits. The `.dgraph` manifest is updated only afterward, so a database revision newer than the manifest is recoverable but not authoritative.

Limits default to 1 MiB Markdown per section, 2,048 sections per capture, 16,384 embedding dimensions, and a 512 MiB database. Active rows require non-empty Markdown, provider/model provenance, a 64-character lowercase SHA-256 hash, and a finite vector whose byte length is exactly `dimensions × 4`.
