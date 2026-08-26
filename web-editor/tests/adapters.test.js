import { describe, expect, it, vi } from 'vitest';
import { BpmnAdapter } from '../src/adapters/BpmnAdapter.js';
import { DataGraphAdapter } from '../src/adapters/DataGraphAdapter.js';

function fakeModeler({ xml = '<saved/>', importError = null } = {}) {
  const listeners = new Map();
  return class {
    on(name, fn) { listeners.set(name, fn); }
    async importXML() { if (importError) throw importError; return { warnings: ['fixture warning'] }; }
    async saveXML() { return { xml }; }
    get(name) { return name === 'canvas' ? { zoom: vi.fn() } : { undo: vi.fn(), redo: vi.fn() }; }
    destroy() {}
    mutate() { listeners.get('commandStack.changed')?.(); }
  };
}

describe.each([['BPMN', BpmnAdapter]])('%s adapter', (_name, Adapter) => {
  it('loads, reports warnings and mutations, and exports', async () => {
    const events = [];
    const adapter = new Adapter(document.createElement('div'), event => events.push(event), fakeModeler());
    await adapter.load('<fixture/>');
    adapter.modeler.mutate();
    expect(events.map(event => event.type)).toEqual(['warnings', 'changed']);
    expect(await adapter.export()).toBe('<saved/>');
  });

  it('propagates import and export failures', async () => {
    const adapter = new Adapter(document.createElement('div'), () => {}, fakeModeler({ importError: new Error('bad xml') }));
    await expect(adapter.load('<bad>')).rejects.toThrow('bad xml');
    adapter.modeler.saveXML = async () => ({});
    await expect(adapter.export()).rejects.toThrow('XML');
  });
});

describe('Data Graph adapter', () => {
  it('loads, tracks only commands, exports, and propagates failures', async () => {
    const listeners = new Map();
    class Modeler {
      on(name, listener) { listeners.set(name, listener); }
      async importJSON(value) { if (value === 'bad') throw new Error('bad graph'); return { warnings: [] }; }
      async saveJSON() { return { json: '{"valid":true}' }; }
      get() { return { undo: vi.fn(), redo: vi.fn() }; }
      destroy() {}
    }
    const events = []; const adapter = new DataGraphAdapter(document.createElement('div'), event => events.push(event), Modeler);
    await adapter.load('{}'); expect(events).toEqual([]); listeners.get('commandStack.changed')(); expect(events[0].type).toBe('changed'); expect(await adapter.export()).toBe('{"valid":true}');
    await expect(adapter.load('bad')).rejects.toThrow('bad graph'); adapter.modeler.saveJSON = async () => ({}); await expect(adapter.export()).rejects.toThrow('JSON');
  });

  it('selects a node, edits and previews Markdown, captures, then restores it after reopen', async () => {
    const starter = JSON.stringify({
      format: 'diagram-studio/data-graph', version: 2, nodes: [], edges: [], properties: [],
      contextStore: { kind: 'sqlite', uri: 'workflow.context.sqlite', schemaVersion: 1, diagramId: 'workflow', revision: 0 }
    });
    const events = [];
    const firstContainer = document.createElement('div'); document.body.append(firstContainer);
    const first = new DataGraphAdapter(firstContainer, event => events.push(event));
    await first.load(starter);
    const node = first.modeler.addNode('objectNode');
    first.modeler.get('selection').select(node);
    expect(first.inspectorElement.hidden).toBe(true);
    first.modeler.get('eventBus').fire('element.click', { element: node });
    await new Promise(resolve => globalThis.setTimeout(resolve, 250));
    expect(first.inspectorElement.hidden).toBe(false);
    first.inspector.view.dispatch({ changes: { from: 0, to: 0, insert: '# Person\nUsed while generating person code.' } });
    first.inspectorElement.querySelector('[data-mode=preview]').click();
    expect(first.inspectorElement.querySelector('.markdown-preview').innerHTML).toContain('<h1>Person</h1>');
    first.inspectorElement.querySelector('[data-action=capture]').click();
    const request = events.find(event => event.type === 'captureRequested');
    expect(request.payload.drafts[0]).toMatchObject({ nodeId: node.id, markdown: '# Person\nUsed while generating person code.\n' });
    first.captureCompleted({ revision: 1 });
    expect(first.inspectorElement.querySelector('.capture-status').textContent).toBe('Captured revision 1');
    const topology = await first.export(); const captured = request.payload.drafts;
    first.destroy();

    const secondContainer = document.createElement('div'); document.body.append(secondContainer);
    const reopened = new DataGraphAdapter(secondContainer, () => {});
    await reopened.load(topology, captured);
    const restoredNode = reopened.modeler.get('elementRegistry').get(node.id);
    reopened.modeler.get('selection').select(restoredNode);
    reopened.modeler.get('eventBus').fire('element.click', { element: restoredNode });
    await new Promise(resolve => globalThis.setTimeout(resolve, 250));
    expect(reopened.inspector.view.state.doc.toString()).toBe('# Person\nUsed while generating person code.\n');
    reopened.destroy();
  });

  it('lets an attached function consume a double-click for label editing', async () => {
    const starter = JSON.stringify({
      format: 'diagram-studio/data-graph', version: 2, nodes: [], edges: [], properties: [],
      contextStore: { kind: 'sqlite', uri: 'labels.context.sqlite', schemaVersion: 1, diagramId: 'labels', revision: 0 }
    });
    const container = document.createElement('div'); document.body.append(container);
    const adapter = new DataGraphAdapter(container, () => {}); await adapter.load(starter);
    const object = adapter.modeler.addNode('objectNode');
    adapter.modeler.get('dataGraphContextPad').getContextPadEntries(object)['append-mutation'].action.click();
    const mutation = adapter.modeler.get('elementRegistry').filter(element => element.businessObject?.type === 'mutationNode')[0];
    const eventBus = adapter.modeler.get('eventBus');
    eventBus.fire('element.click', { element: mutation }); eventBus.fire('element.dblclick', { element: mutation });
    expect(adapter.modeler.get('directEditing').isActive(mutation)).toBe(true);
    expect(adapter.inspectorElement.hidden).toBe(true);
    adapter.modeler.get('directEditing')._active.provider.update(mutation, 'calculateTotal');
    adapter.modeler.get('directEditing').cancel();
    expect(mutation.businessObject.label).toBe('calculateTotal');
    adapter.destroy();
  });
});
