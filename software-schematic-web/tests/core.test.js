import { describe, expect, it, vi } from 'vitest';
import { compositionBreadcrumbs, compositionIdentity, compositionPathFor, compositionSlug, documentationPath, isRootDiagram, NODE_STATUSES, normalizeCompositionPath, normalizeNodeStatus, projectDocumentTitle, RevisionQueue } from '../src/core.js';

describe('documentation paths', () => {
  it('maps diagrams and elements to deterministic Markdown', () => {
    expect(documentationPath('main.bpmn')).toBe('main.md');
    expect(documentationPath('order/main.bpmn', 'Task_1')).toBe('order/docs/Task_1.md');
  });
});

describe('composition identity', () => {
  it('uses folder names and paths instead of repeated base filenames', () => {
    expect(compositionIdentity('main.bpmn')).toEqual({ folder: '', name: 'Main', displayPath: 'schematics' });
    expect(compositionIdentity('sales/order/main.bpmn')).toEqual({
      folder: 'sales/order',
      name: 'Order',
      displayPath: 'schematics/sales/order',
    });
  });
  it('builds navigable breadcrumbs from Main through the current folder', () => {
    expect(compositionBreadcrumbs('birth/main.bpmn')).toEqual([
      { name: 'Main', diagramPath: 'main.bpmn' },
      { name: 'Birth', diagramPath: 'birth/main.bpmn' },
    ]);
    expect(compositionBreadcrumbs('people/early-childhood/main.bpmn').map((item) => item.name)).toEqual([
      'Main', 'People', 'Early Childhood',
    ]);
  });
});

describe('composition paths', () => {
  it('normalizes call activity paths and rejects traversal', () => {
    expect(normalizeCompositionPath('schematics/sales/quote/')).toBe('sales/quote');
    expect(() => normalizeCompositionPath('../private')).toThrow();
  });
  it('derives pool and lane folders', () => {
    const pool = { businessObject: { $type: 'bpmn:Participant', id: 'Pool_A' } };
    const lane = { businessObject: { $type: 'bpmn:Lane', id: 'Lane_B' }, parent: pool };
    expect(compositionPathFor(pool, 'sales/main.bpmn')).toBe('sales/Pool_A');
    expect(compositionPathFor(lane, 'sales/main.bpmn')).toBe('sales/Pool_A/Lane_B');
  });
  it('derives lowercase filesystem-safe subprocess folders from labels', () => {
    expect(compositionSlug(' Birth ')).toBe('birth');
    expect(compositionSlug('Order Fulfillment')).toBe('order-fulfillment');
    expect(compositionPathFor({ businessObject: { $type: 'bpmn:SubProcess', name: 'Birth' } }, 'main.bpmn')).toBe('birth');
    expect(() => compositionSlug('---')).toThrow();
  });
});

describe('RevisionQueue', () => {
  it('serializes writes and only reports the latest revision saved', async () => {
    const states = [];
    const writer = vi.fn(async () => {});
    const queue = new RevisionQueue(writer, (state) => states.push(state));
    await Promise.all([queue.enqueue('main.md', 'one'), queue.enqueue('main.md', 'two')]);
    expect(writer.mock.calls.map((call) => call.slice(1))).toEqual([['one', 1], ['two', 2]]);
    expect(states.at(-1)).toBe('saved');
  });
  it('allows lifecycle cleanup to await the latest path write', async () => {
    let release;
    const writer = vi.fn(() => new Promise((resolve) => { release = resolve; }));
    const queue = new RevisionQueue(writer);
    queue.enqueue('main.bpmn', '<xml />');
    const waiting = queue.waitFor('main.bpmn');
    await vi.waitFor(() => expect(writer).toHaveBeenCalledOnce());
    release();
    await waiting;
  });
});

describe('workspace UI state', () => {
  it('protects only the root main diagram', () => {
    expect(isRootDiagram('main.bpmn')).toBe(true);
    expect(isRootDiagram('orders/main.bpmn')).toBe(false);
  });

  it('normalizes node statuses and publishes the requested color/meaning registry', () => {
    expect(normalizeNodeStatus('locked')).toBe('locked');
    expect(normalizeNodeStatus('unsupported')).toBe('open');
    expect(NODE_STATUSES.new.color).toBe('#62c88a');
    expect(NODE_STATUSES.locked.color).toBe('#a7adb7');
    expect(NODE_STATUSES.modify.color).toBe('#f2a65a');
    expect(NODE_STATUSES.open.color).toBe('#ffffff');
    expect(NODE_STATUSES.locked.meaning).toContain('Do not change');
  });

  it('builds project-aware titles with a safe fallback and Unicode text', () => {
    expect(projectDocumentTitle('order-service')).toBe('Software Schematic - order-service');
    expect(projectDocumentTitle(' Café Platform ')).toBe('Software Schematic - Café Platform');
    expect(projectDocumentTitle('')).toBe('Software Schematic');
  });
});
