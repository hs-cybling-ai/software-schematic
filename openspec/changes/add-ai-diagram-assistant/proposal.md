## Why

SSW users can author diagrams and Markdown manually, but cannot ask an AI assistant to reason over the same diagram context and propose coordinated structural and documentation changes. Adding scoped assistant entry points turns the diagram into an active design surface while keeping every mutation reviewable, validated, and provider-independent.

## What Changes

- Add a magic action to eligible node context palettes that opens a prompt dialog scoped to the selected node, its active diagram, diagram Markdown, node Markdown, composition location, and node-status hints.
- Add a magic button to the diagram palette that opens the same assistant dialog with the full active diagram and its documentation in scope, allowing suggestions that span existing nodes and flows.
- Let users request changes such as replacing a task with a subprocess, creating or opening the subprocess composition, adding a multi-step flow, and updating the related Markdown.
- Introduce a provider-neutral assistant boundary with an initial OpenAI implementation and extension points for Anthropic Claude and Google Gemini.
- Add `ssw auth login|status|logout` so projects can reuse an officially authenticated local Codex or Claude Code CLI without storing account credentials or API keys in the project.
- Require models to return a constrained, declarative change plan rather than raw JavaScript, shell commands, or unconstrained BPMN XML.
- Validate proposed operations against BPMN rules, project path confinement, unique identifiers, and node statuses; `locked` nodes cannot be changed.
- Show a human-readable preview of diagram and documentation changes and require explicit approval before applying them.
- Apply approved changes through supported `bpmn-js` commands and existing file operations so changes participate in automatic persistence and undo where supported.
- Keep provider credentials in the Rust host environment or an OS credential store and never expose them to browser code or diagram files.

## Capabilities

### New Capabilities

- `ai-diagram-assistance`: Scoped AI prompting, context assembly, provider abstraction, structured change plans, validation, preview/approval, and coordinated diagram/documentation application.

### Modified Capabilities

- `software-schematic-editor`: Add diagram-level and node-level magic actions to the established palette and context-palette interaction model.

## Impact

- Affects the SSW browser palette providers, context pads, modal UI, selection/context collection, retained modelers, command stack, Markdown editing, and tests.
- Adds confined Rust endpoints for assistant requests and provider configuration, plus outbound HTTPS from the local server when a remote provider is selected.
- Introduces a versioned diagram-operation schema shared by all providers and an operation executor spanning `bpmn-js`, composition creation, and Markdown writes.
- Requires secrets handling, request cancellation, timeout/error reporting, usage limits, audit-friendly previews, and provider-specific contract tests.
- Does not expose arbitrary filesystem, shell, or code-execution tools to a provider and does not permit unapproved mutations.
