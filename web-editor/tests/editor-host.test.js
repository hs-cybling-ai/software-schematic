import { describe, expect, it, vi } from 'vitest';
import { EditorHost } from '../src/EditorHost.js';

const command = (type, details = {}) => ({ version: 1, type, requestId: details.requestId ?? 'r1', format: details.format ?? 'bpmn', payload: details.payload ?? {} });

describe('EditorHost', () => {
  it('gates commands until load and correlates exports', async () => {
    const messages = [];
    const adapter = { load: vi.fn(), export: vi.fn(async () => '<saved/>'), destroy: vi.fn(), undo: vi.fn(), redo: vi.fn() };
    const host = new EditorHost(document.createElement('div'), message => messages.push(message), { bpmn: () => adapter });
    await host.receive(command('requestExport', { requestId: 'early' }));
    await host.receive(command('load', { payload: { xml: '<source/>' } }));
    await host.receive(command('requestExport', { requestId: 'save-7' }));
    expect(messages.find(message => message.requestId === 'early').type).toBe('failed');
    expect(messages.find(message => message.requestId === 'save-7')).toMatchObject({ type: 'exported', payload: { xml: '<saved/>' } });
  });

  it('destroys the previous adapter and never evaluates XML as script', async () => {
    const first = { load: vi.fn(), destroy: vi.fn() };
    const second = { load: vi.fn(), destroy: vi.fn() };
    const factory = vi.fn().mockReturnValueOnce(first).mockReturnValueOnce(second);
    const host = new EditorHost(document.createElement('div'), () => {}, { bpmn: factory });
    const xml = '</script><script>throw new Error("executed")</script>';
    await host.receive(command('load', { payload: { xml } }));
    await host.receive(command('load', { payload: { xml: '<safe/>' } }));
    expect(first.load).toHaveBeenCalledWith(xml);
    expect(first.destroy).toHaveBeenCalled();
  });

  it('routes Data Graph sessions independently and correlates JSON exports', async () => {
    const messages = []; const adapter = { load: vi.fn(), export: vi.fn(async () => '{"version":1}'), destroy: vi.fn(), undo: vi.fn(), redo: vi.fn() };
    const host = new EditorHost(document.createElement('div'), message => messages.push(message), { dataGraph: () => adapter });
    await host.receive(command('load', { format: 'dataGraph', payload: { xml: '{"version":1}' } })); await host.receive(command('requestExport', { format: 'dataGraph', requestId: 'graph-save' }));
    expect(adapter.load).toHaveBeenCalledWith('{"version":1}'); expect(messages.find(message => message.requestId === 'graph-save')).toMatchObject({ type: 'exported', format: 'dataGraph', payload: { xml: '{"version":1}' } });
  });
});
