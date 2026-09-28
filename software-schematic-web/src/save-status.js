const PENDING_VECTOR_CODES = new Set(['embeddingHeaderMissing', 'embeddingHeaderStale']);

function vectorDiagnostic(refresh) {
  return refresh?.vectorDiagnostics?.find((diagnostic) => !PENDING_VECTOR_CODES.has(diagnostic.code));
}

function diagnosticMessage(diagnostic) {
  if (!diagnostic) return null;
  return diagnostic.source ? `${diagnostic.source}: ${diagnostic.message}` : diagnostic.message;
}

export function saveStatusPresentation(state, detail = null) {
  const presentation = {
    displayState: state,
    label: state === 'pending' ? 'Saving…' : state === 'failed' ? 'Save failed' : 'Saved',
    title: null,
    poll: false,
    toast: null,
    toastKey: null,
  };

  if (state === 'failed') {
    presentation.title = detail?.message || 'Save failed';
    if (detail?.code !== 'revisionConflict' && detail?.message) presentation.toast = detail.message;
    presentation.toastKey = presentation.toast;
    return presentation;
  }
  if (state !== 'saved' || !detail?.graphRefresh) return presentation;

  const refresh = detail.graphRefresh;
  const embedding = detail.embedding;
  const actionableVector = diagnosticMessage(vectorDiagnostic(refresh));

  if (refresh.status === 'queued' || refresh.status === 'processing') {
    presentation.displayState = 'pending';
    presentation.label = refresh.status === 'queued' ? 'Saved; graph queued' : 'Saved; graph updating…';
    presentation.title = refresh.affectedPaths?.length
      ? `Graph update: ${refresh.affectedPaths.join(', ')}`
      : presentation.label;
    presentation.poll = true;
  } else if (refresh.status === 'updated' || refresh.status === 'unchanged') {
    presentation.label = refresh.diagnostic
      ? 'Saved; graph updated with warnings'
      : refresh.status === 'updated' ? 'Saved; graph updated' : 'Saved; graph current';
    presentation.title = refresh.diagnostic
      || (refresh.activeRevision ? `Graph revision ${refresh.activeRevision}` : presentation.label);
    if (refresh.diagnostic) presentation.toast = `Graph rebuilt from main.cmmn; ${refresh.diagnostic}`;
  } else if (refresh.status === 'notRunning') {
    presentation.displayState = 'stale';
    presentation.label = 'Saved; MCP not running';
    presentation.title = refresh.diagnostic || presentation.label;
  } else {
    presentation.displayState = 'refresh-failed';
    presentation.label = 'Saved; graph update failed';
    presentation.title = refresh.diagnostic || presentation.label;
    if (refresh.diagnostic) presentation.toast = `Saved; graph update failed: ${refresh.diagnostic}`;
  }

  if (embedding?.status === 'queued' || embedding?.status === 'processing') {
    presentation.displayState = 'pending';
    presentation.label = embedding.status === 'queued' ? 'Saved; embeddings queued' : 'Saved; embedding…';
    presentation.title = embedding.path ? `Embedding ${embedding.path}` : presentation.label;
    presentation.poll = true;
  } else if (embedding?.status === 'failed') {
    presentation.displayState = 'refresh-failed';
    presentation.label = 'Saved; vector search degraded';
    presentation.title = embedding.diagnostic || actionableVector || presentation.label;
    presentation.toast = `Saved; embedding failed: ${presentation.title}`;
  } else if (embedding?.status === 'stale') {
    presentation.displayState = 'stale';
    presentation.label = 'Saved; vectors stale';
    presentation.title = embedding.diagnostic || actionableVector || presentation.label;
    if (embedding.diagnostic || actionableVector) {
      presentation.toast = `Saved; vector search degraded: ${presentation.title}`;
    }
  } else if (refresh.vectorReadiness === 'degraded') {
    presentation.displayState = 'stale';
    presentation.label = 'Saved; vector search degraded';
    presentation.title = actionableVector || presentation.label;
    if (actionableVector) presentation.toast = `Saved; vector search degraded: ${actionableVector}`;
  } else if (refresh.vectorReadiness === 'pending') {
    presentation.displayState = 'pending';
    presentation.label = 'Saved; vectors pending';
    presentation.title = presentation.label;
    presentation.poll = true;
  }

  presentation.toastKey = presentation.toast
    ? `${refresh.operationId || refresh.completedAtEpochMs || refresh.activeRevision || ''}:${presentation.toast}`
    : null;
  return presentation;
}
