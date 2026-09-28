## Why

Software delivery currently crosses separate diagram, specification, and coding workflows, forcing developers to restate intent and manually synchronize tools before implementation can begin. A portable `design` → `graph` → `build` skill set will make the annotated schematic the durable contract and let Codex or Claude Code move from conversation to diagram to scoped code without the ceremony of a parallel specification system.

## What Changes

- Add portable, provider-neutral `design`, `graph`, and `build` skill packages that can run in Codex or Claude Code with the same behavioral contract and thin host-specific adapters.
- Make `design` an interview-driven workflow that incrementally proposes richly annotated BPMN/CMMN structure and Markdown, presents semantic changes for approval, and resumes from the current diagram state.
- Make `graph` synchronize only the durable diagram or Markdown documents whose content revisions changed, wait for a corresponding atomic graph publication, and fall back conservatively when dependency impact cannot be bounded.
- Make `build` resolve a user's natural-language target to eligible diagram nodes, freeze a revision-bound implementation scope, gather annotated graph context, author and verify code, and report work against stable node references.
- Add optimistic concurrency control shared by the web editor and agent workflows: every read returns a strong content revision, every write is compare-and-swap against that revision, stale writes produce a semantic conflict instead of overwriting newer content, and successful writes emit revisioned change notifications.
- Integrate the same design workflow into the diagram UI as an optional entry point while keeping the skill as the canonical orchestration contract; both surfaces use the same proposal, preview, approval, revision, and conflict APIs.
- Replace periodic full graph reloads in the skill flow with changed-document synchronization and explicit publication receipts, while retaining a safe full-rebuild fallback for structural ambiguity, rename/delete operations, and recovery.
- Generate installation and guidance artifacts that favor these skills over a separate OpenSpec/Speckit planning loop while preserving existing OpenSpec support during migration.

## Capabilities

### New Capabilities

- `diagram-delivery-skills`: Defines the portable `design`, `graph`, and `build` skill contracts, their handoffs, host adapters, node-scoped implementation workflow, and installation/guidance behavior.

### Modified Capabilities

- `ai-diagram-assistance`: Makes UI and chat-based design share one versioned interview/proposal workflow and conflict-safe write protocol.
- `schematic-mcp`: Adds revision-bound changed-document synchronization and publication receipts needed by the `graph` and `build` skills without exposing arbitrary graph mutation.
- `document-save-graph-refresh`: Requires content-addressed incremental refresh for changed documents and consistent handling of editor- and agent-authored updates.
- `software-schematic-editor`: Adds strong document revisions, conditional writes, and actionable conflict handling for concurrent UI and agent edits.

## Impact

- Adds reusable skill directories and Codex/Claude host adapters, likely generated or installed by the project CLI/bootstrap flow.
- Extends typed local document operations and MCP schemas with document revision tokens, expected-revision preconditions, change sets, publication receipts, and bounded refresh status.
- Reuses and refactors the existing assistant proposal validator/executor so the web UI and `design` skill do not implement divergent mutation semantics.
- Extends graph source manifests and refresh coordination to identify changed documents by content digest and invalidate only affected fragments, embeddings, relationships, and indexes.
- Changes managed agent guidance and tests for project identity, stale proposals, simultaneous editor/agent changes, incremental/full equivalence, natural-language scope ambiguity, and node-linked implementation evidence.
