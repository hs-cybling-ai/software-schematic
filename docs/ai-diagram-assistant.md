# AI diagram assistant

Software Schematic exposes the same proposal workflow from two magic actions: the persistent diagram palette sends the complete active diagram, while an eligible node's context palette marks that node as the primary target. The dialog always identifies its scope and provider before transmission.

## Configuration

The deterministic local provider is the default and is intended for testing:

```sh
SSW_ASSISTANT_PROVIDER=fake ./ssw
```

To use OpenAI, configure the Rust host process, never browser JavaScript or project files:

```sh
SSW_ASSISTANT_PROVIDER=openai OPENAI_API_KEY=... ./ssw
```

`SSW_ASSISTANT_MODEL` optionally selects the model. The only allowed OpenAI target is `https://api.openai.com/v1/responses`; a different or non-HTTPS `SSW_ASSISTANT_ENDPOINT` is rejected before project context is sent. Provider credentials are not returned by project metadata, included in previews, embedded in `.ss`, or written to diagrams and Markdown.

## Context and limits

Requests contain a versioned semantic graph of nodes and sequence flows, stable IDs, BPMN types, labels, composition links, transient `open`/`new`/`modify`/`locked` hints, active diagram Markdown, and node Markdown for node-scoped requests. Pending diagram and Markdown edits are flushed first. Context is limited to 256 KiB, prompts to 16,000 characters, request bodies to 512 KiB, responses to 512 KiB, proposals to 64 operations, two concurrent requests, and 30 seconds. Optional Markdown may be truncated and disclosed; required diagram structure is never silently removed.

## Review, application, and recovery

Provider output is an untrusted declarative plan. Supported operations replace a node type, rename a node, set a composition link, create/open a composition, add flow nodes, connect sequence flows, and replace diagram or node Markdown. Code, shell commands, raw patches, arbitrary XML, unknown operations, escaped paths, duplicate IDs, stale revisions, and changes to locked nodes are rejected.

No mutation occurs during generation or preview. The user must approve the complete grouped preview. The editor validates again immediately before application, captures affected BPMN and Markdown content, applies changes through `bpmn-js` and confined file endpoints, and restores captured state if an operation fails. Successful changes retain normal per-diagram undo and expose **Revert assistant change** for the last coordinated proposal.

Closing, rejecting, or cancelling the dialog aborts the browser request, ignores late responses, restores focus to the invoking control, and changes no project content.

## Provider extension boundary

The Rust `AssistantProvider` contract consumes the normalized request and returns the same versioned operation plan regardless of provider SDK. Claude and Gemini adapters can therefore be added beside the OpenAI and fake adapters without changing the dialog, context builder, validators, preview, or executor. A future local MCP server can expose the same read/propose/apply engine to Codex, Claude Code, or Gemini while retaining the approval and confinement rules.
