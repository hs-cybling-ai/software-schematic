## Why

The magic-button assistant currently collapses discovery and mutation planning into one response, forcing a user to review changes before the intended design is settled. It also carries prior transcript state into later invocations, which broadens LLM context beyond the element or diagram the user deliberately chose and undermines the fast, tightly scoped workflow.

## What Changes

- Split each magic-button invocation into two explicit phases: a multi-turn design interview that returns conversational answers without operations, followed only on user request by structured suggested updates.
- Present the interview as a familiar chat surface: a scrolling transcript, a composer anchored to the bottom of the dialog, and a compact send control in the composer corner. Sending by the control or Enter SHALL append the message, clear the composer, keep a fresh input available, and display the assistant response in the transcript; Shift+Enter SHALL insert a newline.
- Let the user move from a structured preview back into the interview, then regenerate suggestions from the refined conversation before approving.
- Place response-specific actions beneath the latest eligible assistant response. `Suggest changes` SHALL be a labeled action because it changes workflow phase, while compact conventional controls such as send, stop-generation, and dialog close MAY be icons with accessible names and tooltips.
- Render structured suggestions inline in the transcript as an outlined `Suggested updates` card containing the grouped update list, assumptions, warnings, `Continue interview`, and a prominent `Approve changes` action.
- Start every magic-button invocation as a new ephemeral session seeded from the latest persisted values for the invoked node or supported edge scope and its connected active diagram, or from the complete active diagram for diagram scope.
- Prevent transcripts, draft proposals, and other invocation state from leaking into a later magic-button request; retain multi-turn context only for the lifetime of the currently open invocation.
- Keep element-scoped proposals acute: changes remain focused on the invoked element and its legitimate subprocess/composition context, while unrelated multi-node changes require diagram-level invocation.
- Preserve existing revision binding, validation, preview, approval, rollback, provider disclosure, and request-size safeguards when the workflow enters structured-update mode.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `ai-diagram-assistance`: Separate conversational interviewing from structured proposal generation, define invocation-lifetime conversation state, and tighten proposal scope around the explicit magic-button target.
- `software-schematic-editor`: Refine the magic assistant dialog states, controls, fresh-session behavior, and element-versus-diagram scope presentation.

## Impact

- Browser assistant state and dialog UI in `software-schematic-web/src/main.js`, `index.html`, and `styles.css`.
- Assistant request/response contracts and provider prompting in `software-schematic-cli/src/assistant.rs` and the local HTTP routes in `software-schematic-cli/src/lib.rs`.
- Browser and Rust tests covering chat composer behavior, keyboard interaction, generation cancellation, conversation turns, inline response actions, phase transitions, scope isolation, stale revisions, preview regeneration, approval, and dismissal.
- Packaged browser assets in `software-schematic-cli/assets/web` must be rebuilt after source changes.
