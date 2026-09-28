import { describe, expect, it, vi } from 'vitest';
import { NODE_STATUSES } from '../src/core.js';
import { hydrateNodeStatuses, importTabXML } from '../src/status-rendering.js';

function statusHarness(kind, initialElements) {
  let elements = initialElements;
  const canvas = { addMarker: vi.fn(), removeMarker: vi.fn() };
  const graphics = new Map();
  const registry = {
    getAll: () => elements,
    getGraphics: (element) => {
      if (!graphics.has(element.id)) graphics.set(element.id, { setAttribute: vi.fn() });
      return graphics.get(element.id);
    },
  };
  const adapter = {
    kind,
    elementLabel: (element) => `${kind}-${element.id}`,
    elementStatus: (element) => element.persistedStatus,
    importXML: vi.fn(async (xml) => {
      elements = xml.elements;
      return { warnings: [] };
    }),
    isStatusEligible: (element) => !element.unsupported,
  };
  return {
    canvas,
    graphics,
    tab: {
      adapter,
      modeler: { get: (service) => service === 'canvas' ? canvas : registry },
      nodeStatuses: new Map([['Stale_1', 'locked']]),
    },
  };
}

describe('diagram status rendering lifecycle', () => {
  for (const kind of ['bpmn', 'cmmn']) {
    it(`hydrates every eligible ${kind.toUpperCase()} element before selection`, () => {
      const elements = [
        { id: 'New_1', persistedStatus: 'new' },
        { id: 'Modify_1', persistedStatus: 'modify' },
        { id: 'Locked_1', persistedStatus: 'locked' },
        { id: 'Default_1' },
        { id: 'Invalid_1', persistedStatus: 'pending' },
        { id: 'Unsupported_1', persistedStatus: 'new', unsupported: true },
      ];
      const { canvas, graphics, tab } = statusHarness(kind, elements);

      hydrateNodeStatuses(tab);

      expect(tab.nodeStatuses).toEqual(new Map([
        ['New_1', 'new'],
        ['Modify_1', 'modify'],
        ['Locked_1', 'locked'],
      ]));
      expect(canvas.removeMarker).toHaveBeenCalledTimes(5 * Object.keys(NODE_STATUSES).length);
      expect(canvas.addMarker.mock.calls.map(([element, marker]) => [element.id, marker])).toEqual([
        ['New_1', 'node-status-new'],
        ['Modify_1', 'node-status-modify'],
        ['Locked_1', 'node-status-locked'],
        ['Default_1', 'node-status-open'],
        ['Invalid_1', 'node-status-open'],
      ]);
      expect(canvas.addMarker.mock.calls.some(([element]) => element.id === 'Unsupported_1')).toBe(false);
      expect(graphics.get('New_1').setAttribute).toHaveBeenCalledWith(
        'aria-label',
        expect.stringContaining(`${kind}-New_1. New.`),
      );
      expect(graphics.get('Locked_1').setAttribute).toHaveBeenCalledWith(
        'aria-label',
        expect.stringContaining('Locked. Do not change'),
      );
    });
  }

  it('rebuilds cache, markers, and accessible labels after XML reimport', async () => {
    const { canvas, graphics, tab } = statusHarness('bpmn', [{ id: 'Old_1', persistedStatus: 'new' }]);
    hydrateNodeStatuses(tab);
    canvas.addMarker.mockClear();
    canvas.removeMarker.mockClear();

    const replacement = [
      { id: 'Replacement_1', persistedStatus: 'modify' },
      { id: 'Replacement_2' },
    ];
    await importTabXML(tab, { elements: replacement });

    expect(tab.adapter.importXML).toHaveBeenCalledOnce();
    expect(tab.nodeStatuses).toEqual(new Map([['Replacement_1', 'modify']]));
    expect(canvas.addMarker.mock.calls.map(([element, marker]) => [element.id, marker])).toEqual([
      ['Replacement_1', 'node-status-modify'],
      ['Replacement_2', 'node-status-open'],
    ]);
    expect(canvas.removeMarker).toHaveBeenCalledTimes(2 * Object.keys(NODE_STATUSES).length);
    expect(graphics.get('Replacement_1').setAttribute).toHaveBeenCalledWith(
      'aria-label',
      expect.stringContaining('bpmn-Replacement_1. Modify.'),
    );
  });
});
