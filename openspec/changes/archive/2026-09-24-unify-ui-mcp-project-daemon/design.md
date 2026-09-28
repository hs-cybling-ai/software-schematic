## Context

Software Schematic serves one developer working with one AI coding client in a local project. The browser owns BPMN/CMMN modeler semantics; the AI is best at interviewing, proposing structure, and implementing code. The tool should connect those strengths without becoming a general collaboration service.

## Product Loop

```text
deep interview ⇄ small live diagram proposals ⇄ manual visual adjustment
                                      │
                                      ▼
                                  ▶ Play
                                      │
                                      ▼
                         graph scope → code → checks
```

The diagram is the durable contract. Conversation helps create it but is not a second requirements database.

## Goals

- Make the design interview thorough enough to expose actors, boundaries, states, failure paths, data, trust, and operational behavior before implementation.
- Make diagram iteration feel immediate: propose a small change, see it live, adjust it, and continue.
- Make Play the obvious transition from designing to building.
- Keep setup and recovery understandable to an individual developer.

## Non-Goals

- Multi-user or multi-browser collaboration.
- Concurrent-client fairness, rate limiting, load testing, or fleet-style health reporting.
- Remote daemon access, generic RPC, or a long-lived service platform.
- Preserving obsolete runtime modes or elaborate migration/rollback machinery.
- Letting an agent bypass the browser modeler with raw XML or arbitrary writes.

## Decisions

### One small project runtime

`./ssw` starts or reuses one loopback-only project process. It owns the HTTP UI, current interview/proposal, document writes, graph snapshot, and Play request. `./ssw mcp` is a small stdio adapter to that process so the AI and UI see the same model.

Discovery uses a project ID, generation, local port, and random local token. These prevent accidental attachment to the wrong project; they are not an enterprise authorization system.

### One current workflow

The runtime stores the current interview summary, current proposal, latest publication receipt, and current Play request. Records are small and replaceable. Full transcripts, credentials, and duplicated requirement corpora are not stored.

The browser reports only the active diagram, its durable revision, dirty state, and a heartbeat. There is no multi-browser arbitration. Opening a new UI session replaces the previous transient session.

### Interview-first, proposal-small

The `design` skill asks substantive follow-ups until the model is coherent. It covers happy paths and uncomfortable details, but it does not wait until the end to visualize. Each settled slice becomes a small typed proposal shown in the browser. The developer may approve it, reject it, or modify the diagram directly before continuing.

The AI cannot approve its own diagram proposal. The browser executes registered operations through `bpmn-js` or `cmmn-js`, saves complete content conditionally, and refreshes the graph.

### Play is the design/build boundary

The browser exposes a prominent Play button. Clicking it first flushes current edits, then records a build request containing the project identity, active diagram, source revision, graph revision, optional selected entity, and a monotonic request revision.

The AI waits for or reads that request, resolves eligible implementation scope from the graph, obtains a revision-bound implementation context, edits code, runs project checks, and reports evidence. A new diagram edit invalidates an old build request and requires Play again.

### Failure behavior stays simple

- If the runtime is absent, the wrapper starts it.
- If it restarted, retry the MCP command; the browser asks for a reload without discarding unsaved text.
- If a proposal is stale, regenerate it from current revisions.
- If application fails, restore the captured before-state.
- `./ssw doctor --repair` restores managed wrapper/MCP/skill files.

## Testing

Keep focused tests for protocol validation, atomic save/rollback, one-runtime startup, proposal delivery, Play request durability, build-scope handoff, clean initialization, and the browser interaction contract. Avoid multi-client matrices, artificial load suites, and enterprise security classifications.
