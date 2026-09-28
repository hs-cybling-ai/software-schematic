# Implementation evidence

Verified 2026-09-23.

## Commands

- `cargo test --manifest-path software-schematic-cli/Cargo.toml` — 65 library tests, 2 MCP wrapper tests, and doc tests passed.
- `npm test -- --run` in `software-schematic-web` — 5 files and 64 tests passed.
- `npm run build` in `software-schematic-web` — production assets rebuilt successfully. Vite reported only its advisory large-chunk warning.
- `openspec validate add-diagram-driven-delivery-skills --strict` — change valid.

## Conformance and end-to-end coverage

- Synchronization rejects foreign projects, path traversal, stale durable revisions, and unsupported change kinds before enqueueing work.
- Editor and skill notifications share one coalescing coordinator; publication receipts are emitted only after a complete replacement snapshot is published.
- Incremental and complete graph builds compare entities, relationships, chunks, source hashes, dependency maps, search ownership, and deterministic revisions.
- Capability discovery, handoff validation, Codex/Claude adapter equivalence, generated clean-project packages, and optional OpenSpec compatibility output have executable fixtures.
- The browser uses the shared typed proposal path, persists bounded interview turns, accepts chat inbox proposals, rejects stale approvals, rolls back failed coordinated changes, and preserves explicit approval/revert behavior.
- The delivery handoff fixture carries stable entity references from design through graph publication into build without copied requirement or transcript text.

## Performance baselines

- Save/refresh enqueue acknowledgement remains below 100 ms while a deliberately delayed 250 ms graph builder runs in the background.
- Digest comparison for a 500-document manifest completes below 2 seconds on the test host and treats touched-but-identical content as unchanged.
