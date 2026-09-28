import { describe, expect, it } from 'vitest';
import { saveStatusPresentation } from '../src/save-status.js';

const published = {
  status: 'updated',
  activeRevision: 'sha256:graph',
  operationId: 'sync-7',
  retrievalMode: 'text',
  vectorReadiness: 'pending',
  vectorDiagnostics: [{
    code: 'embeddingHeaderMissing',
    level: 'warning',
    message: 'embedding header is missing; vector retrieval is pending',
    source: 'cybling/network/sponsord/SponsorBirth/main.md',
  }],
};

describe('save status presentation', () => {
  it('keeps polling while graph refresh is queued', () => {
    const result = saveStatusPresentation('saved', {
      graphRefresh: { status: 'queued', affectedPaths: ['main.md'] },
      embedding: { status: 'current' },
    });

    expect(result).toMatchObject({
      displayState: 'pending', label: 'Saved; graph queued', poll: true, toast: null,
    });
  });

  it('shows embedding processing as normal background progress', () => {
    const result = saveStatusPresentation('saved', {
      graphRefresh: published,
      embedding: { status: 'processing', path: 'main.md' },
    });

    expect(result).toMatchObject({
      displayState: 'pending', label: 'Saved; embedding…', title: 'Embedding main.md', poll: true, toast: null,
    });
  });

  it('shows current hybrid retrieval without a toast', () => {
    const result = saveStatusPresentation('saved', {
      graphRefresh: {
        ...published,
        retrievalMode: 'hybrid',
        vectorReadiness: 'current',
        vectorDiagnostics: [],
      },
      embedding: { status: 'current' },
    });

    expect(result).toMatchObject({
      displayState: 'saved', label: 'Saved; graph updated', poll: false, toast: null,
    });
  });

  it('surfaces stale vectors without active repair when their diagnostic is actionable', () => {
    const result = saveStatusPresentation('saved', {
      graphRefresh: {
        ...published,
        vectorReadiness: 'degraded',
        vectorDiagnostics: [{
          code: 'embeddingHeaderMalformed',
          level: 'warning',
          message: 'invalid software-schematic embedding header',
          source: 'main.md',
        }],
      },
      embedding: { status: 'stale' },
    });

    expect(result).toMatchObject({
      displayState: 'stale',
      label: 'Saved; vectors stale',
      title: 'main.md: invalid software-schematic embedding header',
      poll: false,
      toast: 'Saved; vector search degraded: main.md: invalid software-schematic embedding header',
    });
  });

  it('surfaces embedding derivation failure without calling the document save failed', () => {
    const result = saveStatusPresentation('saved', {
      graphRefresh: published,
      embedding: { status: 'failed', diagnostic: 'model unavailable' },
    });

    expect(result).toMatchObject({
      displayState: 'refresh-failed',
      label: 'Saved; vector search degraded',
      title: 'model unavailable',
      toast: 'Saved; embedding failed: model unavailable',
    });
  });

  it('preserves actionable graph warnings', () => {
    const result = saveStatusPresentation('saved', {
      graphRefresh: {
        ...published,
        vectorReadiness: 'current',
        vectorDiagnostics: [],
        diagnostic: 'main.cmmn: skipped edge Stale',
      },
      embedding: { status: 'current' },
    });

    expect(result).toMatchObject({
      displayState: 'saved',
      label: 'Saved; graph updated with warnings',
      toast: 'Graph rebuilt from main.cmmn; main.cmmn: skipped edge Stale',
    });
  });

  it('surfaces graph refresh failure as derived-state failure', () => {
    const result = saveStatusPresentation('saved', {
      graphRefresh: { status: 'failed', diagnostic: 'malformed XML' },
      embedding: { status: 'current' },
    });

    expect(result).toMatchObject({
      displayState: 'refresh-failed',
      label: 'Saved; graph update failed',
      title: 'malformed XML',
      toast: 'Saved; graph update failed: malformed XML',
    });
  });

  it('does not toast the expected missing header while vectors are pending', () => {
    const result = saveStatusPresentation('saved', {
      graphRefresh: published,
      embedding: { status: 'queued', path: 'cybling/network/sponsord/SponsorBirth/main.md' },
    });

    expect(result).toMatchObject({
      displayState: 'pending', label: 'Saved; embeddings queued', poll: true, toast: null,
    });
  });
});
