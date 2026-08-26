## Context

SSW is a self-contained browser editor backed by a local Rust server. The browser owns live `bpmn-js` modelers, selection, undoable commands, retained tabs, and Markdown UI; the Rust process owns confined project-file access and is the only appropriate place for provider credentials and outbound API requests. The editor already knows the selected element, active composition path, current BPMN XML, diagram Markdown, node Markdown, and transient node-status hints.

The assistant must coordinate changes across a live diagram, nested compositions, and Markdown without giving a remote model direct access to JavaScript, shell execution, arbitrary filesystem paths, or credentials. Model output is therefore an untrusted proposal that requires schema validation and user approval before the editor applies it.

## Goals / Non-Goals

**Goals:**

- Offer a node-scoped magic action in eligible element context palettes.
- Offer a diagram-scoped magic action in the persistent diagram palette.
- Assemble deterministic, size-bounded context for the chosen scope.
- Support requests that change BPMN structure and related Markdown together, including task-to-subprocess composition creation.
- Keep provider integration behind one interface, with an initial OpenAI Responses API adapter and contract fixtures for later Claude and Gemini adapters.
- Produce validated, human-readable change previews and require explicit approval.
- Apply approved operations through existing editor/server primitives, preserving automatic persistence and practical undo behavior.
- Enforce locked-node, identifier, path-confinement, operation-count, and request-size rules independently of the model.

**Non-Goals:**

- Allowing a provider to emit or execute arbitrary JavaScript, shell commands, patches, or filesystem operations.
- Letting the provider mutate the live editor during inference.
- Automatically approving or silently applying suggestions.
- Providing a general chat assistant, repository coding agent, or autonomous multi-turn planner in the first release.
- Shipping Anthropic or Gemini credentials/adapters in the first release; the provider interface must allow them without changing the UI or operation engine.
- Persisting conversation history beyond the current prompt/proposal lifecycle.

## Decisions

### Use two magic entry points with explicit scopes

The context-pad magic action appears on eligible BPMN shapes and binds the prompt to that element. Its request includes the complete active diagram for structural understanding but identifies the selected node as the primary target and gives its Markdown extra prominence. The diagram-palette magic action has no primary node and invites changes anywhere in the active diagram.

Both open the same accessible modal containing scope identification, a multiline prompt, submit/cancel controls, progress, proposal preview, approve/reject controls, and errors. Sharing one dialog and request pipeline keeps the additional UI small while preventing scope ambiguity.

### Build a compact semantic snapshot rather than relying on raw XML alone

The browser sends a versioned snapshot containing diagram path; normalized nodes and flows with IDs, BPMN types, labels, composition references, and status hints; active diagram Markdown; and, for node scope, selected-node Markdown. Raw BPMN XML can be included as bounded reference data when needed, but the semantic graph is the primary context because it is smaller and easier for providers to reason about.

The browser flushes pending Markdown and diagram edits before snapshotting. Every request includes a source revision/hash. Approval is rejected if the live diagram or relevant Markdown changed after the proposal was generated, requiring regeneration instead of applying a stale plan.

### Define a versioned provider-neutral operation plan

The provider returns a strict schema with a summary, assumptions/warnings, and ordered operations. The first version supports a deliberately bounded set: replace an eligible node type, update node label, create or open a composition, add supported flow nodes, connect nodes with sequence flows, set composition linkage, and replace diagram or node Markdown.

Operations reference stable IDs and normalized composition-relative paths. They never contain executable code or unrestricted XML. The same JSON schema is supplied to every provider adapter, making the operation engine and UI independent of provider response formats.

### Treat model output as untrusted and validate twice

The Rust boundary validates response shape, size, allowed operation types, normalized paths, and request correlation before returning a proposal. The browser then performs semantic validation against the current live model: referenced elements exist, created IDs are unique, connections and type replacements are permitted, operation limits are respected, and no operation modifies a `locked` node or uses it as a destructive replacement target.

`modify` and `new` statuses are included as positive hints but do not bypass validation. `open` provides no hint. Validation failures produce an explanation and no mutation.

### Preview before applying

The proposal view groups changes by diagram and documentation file and describes replacements, additions, connections, composition creation, and Markdown updates. Approval is a distinct action after generation. Rejecting or closing the preview has no effect.

The first release previews semantic changes rather than rendering a speculative second BPMN canvas. A full visual diff was considered but rejected for initial scope because it duplicates modelers and makes cross-file proposals significantly heavier.

### Apply browser-owned and server-owned operations in a coordinated transaction

The browser creates or opens required composition tabs first through the confined composition endpoint, then stages `bpmn-js` changes through supported modeling/replacement services and stages Markdown content in the appropriate editors/files. Each affected live modeler applies its operations as a grouped command where the framework permits, giving each diagram a coherent undo step.

Before application, the server records recoverable pre-change contents for all affected files or the client retains complete before-state exports. Persistence occurs only after every operation validates and stages successfully. If staging fails, all affected modelers and Markdown drafts return to their before-state and no proposal is reported as applied. This small transaction coordinator is preferred over sequential best-effort writes, which could leave a created subprocess without its parent linkage or documentation.

### Put credentials and provider calls in Rust

The Rust server exposes a confined assistant-proposal endpoint. An `AssistantProvider` trait accepts the normalized request and operation schema and returns a plan plus provider usage metadata. The initial OpenAI adapter uses the Responses API with strict structured/tool output. Credentials come from environment variables or a later OS credential-store integration and are never returned through metadata endpoints, logged, embedded in `.ss`, or sent to the browser.

The endpoint enforces timeouts, cancellation, maximum request/response sizes, concurrency limits, and redacted errors. Provider/model configuration is explicit and local. A deterministic fake provider drives tests.

### Delegate subscription authentication to official local agent CLIs

`ssw auth login` discovers Codex first and Claude Code second, or accepts an explicit provider selection. It launches the provider's official login command and never prompts for passwords, MFA codes, cookies, or tokens. The project stores only the provider name under `.ss`; credentials remain in the provider CLI's credential cache or OS keychain. Proposal generation invokes the selected CLI ephemerally with tools disabled, read-only or plan permissions, no conversation persistence, and the canonical operation-plan output schema. Status and logout likewise delegate to the official CLI.

### Keep MCP as a follow-on surface over the same engine

The operation schema, context builder, validators, and executor are designed so a later local MCP server can expose read/propose/apply tools to Codex, Claude Code, or Gemini without duplicating mutation logic. MCP is not required for the embedded magic-button workflow and is excluded from the initial implementation.

## Risks / Trade-offs

- [A valid schema can still encode a poor design] → Require a readable preview, surface assumptions, and never auto-approve.
- [Cross-diagram changes can partially apply] → Validate the complete plan first, capture before-state, stage changes, and roll back all affected models/files on failure.
- [Live content can change while inference is running] → Include revision hashes and reject stale proposals at approval time.
- [Large diagrams and Markdown can exceed context or cost limits] → Normalize context, cap sizes and operations, show truncation, and reject requests that cannot retain required structural context.
- [Node colors/statuses are currently transient] → Include the live status map in the request and enforce `locked` again at apply time; do not claim persistence across reopened tabs.
- [Provider behavior differs] → Keep one canonical operation schema, deterministic fixtures, adapter contract tests, and provider-independent validation.
- [Secrets or proprietary diagrams could leave the machine] → Make provider use explicit, show which context will be sent, keep keys server-side, and provide a cancel path before transmission.
- [Task replacement can invalidate references or documentation ownership] → Define replacement semantics that preserve stable IDs where supported and explicitly migrate documentation/composition linkage.
- [Undo across multiple tabs/files is not naturally atomic] → Store a coordinated before-state and offer one assistant-level revert action in addition to per-modeler undo.

## Migration Plan

1. Define the context snapshot and operation-plan schemas with fixtures and validators.
2. Add the provider trait, deterministic fake provider, confined assistant endpoint, configuration, cancellation, and redacted errors.
3. Add the shared assistant dialog and node/diagram magic entry points.
4. Implement scope-aware context collection and stale-revision detection.
5. Add semantic preview, approval, locked-node enforcement, and supported single-diagram operations.
6. Add composition creation, cross-diagram coordination, Markdown operations, rollback, and assistant-level revert.
7. Add the OpenAI provider adapter and end-to-end tests for the task-to-subprocess four-step example.

Rollback disables or removes the assistant controls and endpoint. Existing BPMN and Markdown remain standard project files; no format migration is required.

## Open Questions

- Should the first release support only OpenAI, or ship Claude and Gemini adapters simultaneously after the shared contract is proven?
- Should diagram-level context include Markdown for every node, or only diagram Markdown plus selected/referenced node documents retrieved on demand?
- What per-request token, operation-count, Markdown-size, and timeout defaults are acceptable?
- Should approval support deselecting individual operations, or remain all-or-nothing in the first release?
- Where should long-lived provider selection and credential references be configured: environment variables, `.ss` configuration, or the OS keychain?
