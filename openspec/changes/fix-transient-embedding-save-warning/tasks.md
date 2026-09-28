## 1. Typed Graph and Vector Status

- [x] 1.1 Add stable diagnostic classification and vector-readiness data to graph construction so missing, stale, malformed, and incompatible embedding artifacts remain distinguishable from structural graph warnings; verify focused `schematic_graph` tests cover text-ready missing/stale headers, corrupt artifacts, and hybrid-ready current headers.
- [x] 1.2 Extend `GraphRefreshOutcome` with additive retrieval mode, embedding revision, and vector-readiness fields, and stop promoting expected pending-vector diagnostics into its actionable graph diagnostic; verify `schematic_mcp` tests preserve structural warnings and failures while a text-ready refresh reports pending vectors without the missing-header warning.
- [x] 1.3 Exercise the full Markdown transition from durable body save through compare-and-swap header publication and second graph refresh; verify a Rust integration test observes text/pending followed by hybrid/current without changing the authored body or emitting a false graph warning.

## 2. Editor Status and Packaged Assets

- [x] 2.1 Update save-status rendering to combine typed graph readiness with embedding-job lifecycle, continue polling while either is pending, and emit toasts only for actionable graph or derivation outcomes; verify Vitest cases cover queued, processing, current, stale-without-active-repair, derivation-failed, graph-warning, and graph-failed payloads.
- [x] 2.2 Rebuild the packaged web application after source tests pass and verify the generated asset references and UI contract tests use the updated status behavior.

## 3. Validation

- [x] 3.1 Run `npm test --prefix software-schematic-web` and focused `cargo test --manifest-path software-schematic-cli/Cargo.toml` coverage for graph refresh and embeddings; verify both suites pass without weakening existing corrupt-artifact or failure assertions.
- [x] 3.2 Run `openspec validate fix-transient-embedding-save-warning --strict` and `./scripts/verify-release.sh`; verify the change artifacts, packaged application, Rust tests, and release checks all pass.
