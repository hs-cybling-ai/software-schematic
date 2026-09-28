## 1. Status Rendering Lifecycle

- [x] 1.1 Add a tab-scoped status hydration routine that clears and rebuilds `nodeStatuses`, applies exactly one normalized status marker to every eligible registry element, and sets its accessible description through that tab's adapter; verify focused Vitest cases cover `new`, `modify`, `locked`, default/invalid `open`, unsupported elements, and stale-marker removal.
- [x] 1.2 Invoke status hydration after initial BPMN and CMMN tab import so persisted colors and accessible descriptions exist before selection; verify a regression test observes all expected markers without dispatching a selection event.
- [x] 1.3 Route conflict reload, process/package rename, assistant revert, and every other existing-tab XML reimport through the same post-import hydration contract; verify a reimport test replaces prior cache/marker state with the statuses from the new registry.

## 2. Integration Verification

- [x] 2.1 Update the browser workspace contract coverage to assert that every direct XML import path performs post-import status hydration, then run `npm test --prefix software-schematic-web` and verify the complete web suite passes.
- [x] 2.2 Run `npm run build --prefix software-schematic-web`, verify the committed `software-schematic-cli/assets/web/` bundle is regenerated, and run `cargo test --manifest-path software-schematic-cli/Cargo.toml` to confirm the embedded application remains valid.
