## Context

See `proposal.md` for motivation. The browser currently keeps one assistant object, persists transcript turns in `localStorage`, and sends each submitted prompt directly to `/api/assistant/proposals`. Every provider adapter is therefore optimized for structured output, and the UI moves immediately from a single response to proposal approval.

The existing safety boundary remains valuable: the browser builds bounded semantic context, the Rust host owns provider credentials and timeouts, both Rust and browser code validate structured plans, and only explicit approval executes modeler operations. The new workflow must preserve that boundary while adding prose responses and tighter element mutation authority.

## Goals / Non-Goals

**Goals:**

- Represent the dialog as an explicit interview-to-suggestion state machine.
- Keep all conversational state ephemeral and owned by one open browser invocation.
- Give every provider the same ordered-turn input and distinct prose-versus-structured output contracts.
- Re-snapshot durable scoped content before structured generation and keep approval revision-bound.
- Enforce node-scoped mutation limits independently of provider prompting.
- Preserve review of proposals submitted through the existing assistant inbox/runtime paths.

**Non-Goals:**

- Persisting or resuming magic-button transcripts across dialog invocations, browser reloads, or devices.
- Giving the model tools, filesystem access, or direct modeler mutation authority during the interview.
- Expanding the current operation registry or automatically adding an edge context-pad action; the context contract remains able to represent an edge when a supported entry point supplies one.
- Automatically applying “obvious” updates or generating suggestions based only on model confidence.

## Decisions

### 1. Use an explicit client-side invocation state machine

The assistant state will carry a unique invocation ID and a phase of `interview`, `requestingInterview`, `requestingSuggestion`, or `preview`. `openAssistant` will abort any prior request and replace all invocation fields—turns, snapshot, proposal, submission correlation, error, and phase—rather than rehydrating transcript state. `closeAssistant` will abort and discard the active invocation. `lastBeforeState` remains outside this reset because it supports reverting an already applied change, not continuing an interview.

The dialog uses a chat layout rather than a phase-wide action footer. A scrollable transcript occupies the body and a composer remains anchored to the bottom throughout the invocation. The send control sits in the lower corner of the composer as a conventional arrow icon with an accessible name and tooltip. Activating it or pressing Enter accepts a non-empty message, appends it to the transcript, immediately clears the composer, and keeps a blank composer available for the next turn. Shift+Enter inserts a newline, and IME composition SHALL complete without accidentally sending. During an in-flight conversational response, the same corner exposes an accessible stop-generation icon instead of send.

After each completed assistant response that can inform changes, a compact response-action row exposes a sparkle icon plus the visible label `Suggest changes`. The label remains visible because this action crosses from discussion into structured generation and is less universal than send or close. Dialog close remains a labeled X icon in the header rather than a repeated response action.

A valid structured result is inserted into the transcript as an outlined `Suggested updates` card, not moved to a separate preview screen. The card contains the full grouped semantic change list, assumptions, warnings, a secondary `Continue interview` action, and a prominent `Approve changes` action. Continuing appends a bounded, normalized description of the invalidated candidate to the current invocation transcript, makes the old card non-approvable, and returns focus to the composer. The transcript remains readable and the composer remains anchored in both `interview` and `preview` phases.

Alternative considered: infer phase from whether `assistant.proposal` exists. Rejected because in-flight requests, imported proposals, invalidated previews, and error recovery otherwise produce ambiguous combinations of controls and stale state.

### 2. Keep transcript ownership in the browser and send complete bounded turns per request

No conversation session will be stored in Rust, provider state, `localStorage`, or project files. Each request carries the opening/current scoped snapshot plus an ordered list of typed turns from the active invocation. The host validates role, type, per-turn size, turn count, and total request size before calling a provider. Local CLI calls remain ephemeral and OpenAI requests contain the complete bounded input, so provider behavior does not depend on remote conversation IDs.

Turn types are `user`, `assistant`, and `proposalSummary`. A `proposalSummary` is generated from the already validated plan and includes its summary and deterministic operation descriptions; it is not accepted as arbitrary browser-authored structured output. This lets a user discuss a rejected candidate without keeping it approvable.

Alternative considered: create server-side sessions keyed by a conversation ID. Rejected because it adds lifecycle cleanup, synchronization, and accidental cross-invocation reuse risks without improving a dialog whose state is intentionally short-lived.

### 3. Separate conversational and structured provider contracts

Add a confined conversational endpoint, `/api/assistant/conversations`, beside the existing proposal endpoint. Both routes use the same concurrency, timeout, provider selection, disclosure, context validation, and redacted-error policy. The provider trait gains a conversational operation that returns `{ reply, provider, model, usage }`; the existing proposal operation accepts bounded current-invocation turns in addition to the latest snapshot and returns the canonical `AssistantPlan`.

Provider prompts will clearly delimit application instructions, semantic snapshot, and conversation turns. Conversational calls request plain prose and forbid operation plans. Proposal calls request only the existing JSON-schema-constrained operation plan and instruct the model to synthesize it from the transcript. The deterministic fake provider will implement both modes for browser and Rust tests.

Alternative considered: one endpoint with a `mode` discriminator and a union response. Rejected because separate routes and result types make it harder for prose to be mistaken for an approvable plan and simplify body validation and tests.

### 4. Snapshot once for discussion, refresh before suggestion, validate at approval

Opening a magic action flushes the active diagram and relevant Markdown and builds the initial semantic snapshot before the first provider call. All interview turns use that invocation snapshot so the discussion has a stable referent. Selecting `Suggest updates` flushes again and builds a fresh snapshot. If the scoped durable revision changed, the structured request uses the new revision; the transcript remains current-invocation context, and the provider is told that persisted scope is authoritative where it differs from earlier discussion.

Approval continues to rebuild the current snapshot and compare its revision with the proposal revision. Any later change makes the proposal stale and requires regeneration.

Alternative considered: rebuild context on every prose turn. Rejected because it adds repeated I/O and makes a discussion internally unstable without improving the final revision guard.

### 5. Enforce acute scope in validators, not prompts alone

The semantic snapshot will expose a primary target identity and kind for element scope while retaining compatibility aliases needed by the current node-oriented operation schema. Rust validation and browser validation will independently calculate the allowed mutation set.

For a node-scoped parent diagram, direct mutation operations may target only the primary node and its owned Markdown. Composition create/open/reference operations are allowed only when anchored by that node; operations inside that directly referenced or newly created child composition remain allowed. Operations targeting other existing parent nodes or parent flows are rejected. Diagram scope retains the current complete-diagram authority. A future supported edge invocation will analogously limit parent mutations to the primary edge and its Markdown.

Alternative considered: rely on provider instructions to remain focused. Rejected because provider output is untrusted and the existing validator currently permits any known node in the active diagram.

### 6. Treat imported proposals as preview-only invocations

Opening an inbox or runtime proposal creates a fresh invocation directly in `preview` with its validated proposal and current source correlation. The proposal is rendered as the same inline outlined card used for a generated suggestion. Approve and close behave as today. Choosing `Continue interview` converts the proposal into a bounded `proposalSummary`, clears its approval correlation, refreshes the scoped snapshot, and starts an ephemeral interview. It does not recover or merge any historical chat transcript.

Alternative considered: leave imported previews on their legacy approve/reject/close path. Rejected because the user-facing preview controls would differ depending on proposal origin and would undermine the two-phase model.

## Risks / Trade-offs

- [Long transcripts increase latency and token use] → Enforce per-turn, turn-count, and aggregate byte limits; show an actionable limit error instead of silently dropping current-invocation decisions.
- [Persisted content changes can contradict earlier turns] → Refresh before suggestion, mark the refreshed snapshot authoritative, and retain the existing stale-proposal approval check.
- [Tighter node validation rejects plans that previously happened to work] → Return a precise scope diagnostic directing multi-node intent to diagram-level magic.
- [A prose provider may still embed JSON-like text] → Render all replies as inert text/Markdown under the existing sanitization policy and never construct a proposal outside the structured endpoint and validator.
- [Enter-to-send can submit incomplete text or interfere with text composition] → Reserve Shift+Enter for newlines, ignore Enter while an IME composition is active, disable empty submission, and cover keyboard and composition events with interaction tests.
- [Icon-only controls can be ambiguous] → Limit them to established send, stop, and close metaphors; provide accessible names, tooltips, visible focus states, and keep workflow-changing actions visibly labeled.
- [Removing local transcript persistence changes an existing “persistent chat” expectation] → Make the invocation boundary visible in the dialog and cover reopen/reload behavior with tests.
- [Source and packaged web assets can drift] → Rebuild the packaged assets and retain source-versus-bundle contract tests in the release check.

## Migration Plan

1. Add the conversation request/result types, provider method, route, limits, and tests without changing proposal execution.
2. Extend proposal requests to accept current-invocation turns and add acute-scope validation in Rust and browser code.
3. Replace browser transcript persistence and implicit proposal submission with the explicit invocation state machine, scrolling transcript, anchored composer, response actions, and inline suggestion cards.
4. Update inbox/runtime preview handling to enter the same preview state and support continuing as a fresh invocation interview.
5. Rebuild `software-schematic-cli/assets/web`, then run browser, Rust, integration, and OpenSpec validation suites.

Rollback is a code rollback: no durable conversation data or project-file migration is introduced. Existing diagrams and Markdown remain compatible because operation and file formats do not change.
