## 1. Assistant Request Contracts

- [x] 1.1 Add bounded conversation turn, conversation request/result, and transcript-bearing proposal request types in the Rust assistant module; verify unit tests reject invalid roles/types, empty user input, oversized turns, excessive turn counts, and oversized aggregate context.
- [x] 1.2 Extend the provider abstraction and deterministic fake provider with a prose interview operation while keeping structured generation schema-bound; verify provider unit tests prove interview results contain only prose and proposal results retain request/revision correlation.
- [x] 1.3 Implement equivalent interview and transcript-aware suggestion prompting for Codex, Claude, and OpenAI adapters with tools and persistence disabled; verify adapter tests or captured request fixtures distinguish instructions, snapshot, turns, and output mode without exposing credentials.
- [x] 1.4 Add `/api/assistant/conversations` with the existing provider selection, concurrency, timeout, size-limit, cancellation, and redacted-error policies, and extend `/api/assistant/proposals` for current-invocation turns; verify Rust route tests cover success, invalid input, provider failure, concurrency rejection, and timeout behavior.

## 2. Scoped Context and Validation

- [x] 2.1 Extend browser context snapshots with a primary target identity/kind while preserving current node compatibility and bounded Markdown behavior; verify assistant unit tests cover diagram, node, and representable edge targets plus current persisted values after a flush.
- [x] 2.2 Enforce acute element mutation scope in the browser validator so parent-diagram operations can affect only the primary target while directly owned child-composition operations remain valid; verify tests reject unrelated peer and parent-flow mutations and accept selected-node, child-composition, and diagram-scoped multi-node plans.
- [x] 2.3 Mirror acute element mutation enforcement in Rust plan validation and provider instructions; verify Rust tests exercise the same reject/accept matrix and return an actionable diagram-level-assistance diagnostic for multi-node intent.
- [x] 2.4 Refresh and revision-bind the scoped snapshot before structured suggestion generation and retain the approval-time stale check; verify browser tests cover durable changes during interview, regenerated proposals, and changes occurring after preview.

## 3. Two-Phase Browser Workflow

- [x] 3.1 Replace implicit proposal controls with accessible phase-specific `Send`, `Suggest updates`, `Continue interview`, `Approve changes`, and `Close` controls and loading/error states; verify UI contract tests assert the correct controls and labels for interview, in-flight, and preview phases.
- [x] 3.2 Implement the explicit invocation state machine and remove transcript `localStorage` reads/writes so every magic activation aborts and clears the previous transcript, request, proposal, error, and phase; verify tests reopen the same and different scopes and observe an empty transcript seeded from current persisted diagram/Markdown values.
- [x] 3.3 Wire interview submission to the conversation endpoint, append only current-invocation turns, and render provider prose as inert sanitized content; verify browser tests cover multiple turns, focused follow-ups, explanation-only answers, cancellation, limits, and provider errors without creating a proposal.
- [x] 3.4 Wire `Suggest updates` to the structured endpoint with the refreshed snapshot and full bounded invocation transcript, then render the validated grouped preview; verify tests prove no structured request occurs before explicit activation and that invalid or stale plans never become approvable.
- [x] 3.5 Implement `Continue interview` by converting the validated candidate into a bounded deterministic proposal summary, invalidating approval state, and returning to interview mode; verify tests refine a candidate, regenerate a different proposal, and approve only the latest preview.
- [x] 3.6 Update close, approval, and application paths to discard the ephemeral invocation while preserving assistant-level revert state; verify tests cover close-without-mutation, successful apply, failed apply/rollback, focus restoration, and post-apply revert.
- [x] 3.7 Route inbox and runtime proposals into the same preview state and allow them to continue as fresh ephemeral interviews without importing historical transcripts; verify integration tests cover approve, close, continue, regenerate, and correlation cleanup for both proposal origins.
- [x] 3.8 Apply the same two-phase behavior and state isolation to BPMN and CMMN magic actions; verify UI/integration tests exercise node- and diagram-scoped invocations for both diagram kinds.

## 4. Packaging and Verification

- [x] 4.1 Run `npm test` in `software-schematic-web` and fix all browser unit and contract failures, verifying the complete Vitest suite passes.
- [x] 4.2 Run `cargo test` in `software-schematic-cli` and fix all assistant, route, provider, and existing regression failures, verifying the complete Rust suite passes.
- [x] 4.3 Run `npm run build` in `software-schematic-web` to refresh `software-schematic-cli/assets/web`; verify the packaged HTML/assets expose the two-phase controls and contain no legacy transcript-persistence behavior.
- [x] 4.4 Run `openspec validate refine-magic-conversation-workflow --strict` and the repository's relevant end-to-end checks, verifying the capability deltas, implementation, and bundled application agree before marking the change complete.

## 5. Chat-Style Usability Refinement

- [x] 5.1 Restructure the assistant dialog as a scrollable transcript with a responsive composer anchored to the bottom in interview and suggestion phases; verify layout tests cover long transcripts and constrained dialog heights without content being hidden behind the composer.
- [x] 5.2 Move the accessible send control into the composer corner and switch it to stop-generation while a conversational request is active; verify interaction tests cover click-to-send, Enter-to-send, Shift+Enter newline, IME composition, empty submission, cancellation, focus, tooltip, and target-size behavior.
- [x] 5.3 Clear the composer immediately after an accepted submission, keep a fresh blank input available, append the assistant response to the transcript, and place a labeled `Suggest changes` action beneath eligible assistant responses; verify multi-turn and error tests retain the submitted turn without leaking proposal state.
- [x] 5.4 Render generated and imported structured proposals inline as outlined `Suggested updates` sections with the complete grouped update list, assumptions, warnings, a secondary `Continue interview` action, and a prominent `Approve changes` action; verify continuing invalidates the card and only the current valid card can be approved.
- [x] 5.5 Rebuild packaged web assets and rerun browser, Rust, integration, accessibility, and strict OpenSpec validation checks so source, bundle, behavior, and updated artifacts agree.
