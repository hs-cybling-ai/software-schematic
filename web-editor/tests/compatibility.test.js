import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import BpmnModeler from 'bpmn-js/lib/Modeler';
import { BpmnAdapter } from '../src/adapters/BpmnAdapter.js';
import { DataGraphAdapter } from '../src/adapters/DataGraphAdapter.js';

const fixture = path => readFile(resolve(import.meta.dirname, '../../fixtures', path), 'utf8');

describe('locked dependency compatibility', () => {
  it('imports and exports BPMN XML', async () => {
    const modeler = new BpmnModeler();
    await modeler.importXML(await fixture('bpmn/valid.bpmn'));
    const { xml } = await modeler.saveXML({ format: true });
    expect(xml).toContain('bpmn:definitions');
    modeler.destroy();
  });

  it.each(['bpmn/invalid.bpmn'])('rejects malformed fixture %s', async path => {
    const Modeler = BpmnModeler;
    const modeler = new Modeler();
    await expect(modeler.importXML(await fixture(path))).rejects.toBeDefined();
    modeler.destroy();
  });

  it.each([
    ['BPMN', BpmnAdapter, 'bpmn/starter.bpmn', 'bpmn:definitions']
  ])('imports and exports the %s starter document through its adapter', async (_name, Adapter, path, marker) => {
    const adapter = new Adapter(document.createElement('div'), () => {});
    adapter.modeler.get('canvas').zoom = () => {};
    await adapter.load(await fixture(path));
    expect(await adapter.export()).toContain(marker);
    adapter.destroy();
  });

  it('imports and deterministically exports the Data Graph starter and visual fixtures', async () => {
    for (const path of ['data-graph/starter.dgraph', 'data-graph/valid.dgraph']) {
      const adapter = new DataGraphAdapter(document.createElement('div'), () => {});
      await adapter.load(await fixture(path));
      const exported = await adapter.export();
      expect(JSON.parse(exported)).toMatchObject({ format: 'diagram-studio/data-graph', version: 2 });
      adapter.destroy();
    }
  });

  it('rejects the invalid Data Graph fixture', async () => {
    const adapter = new DataGraphAdapter(document.createElement('div'), () => {});
    await expect(adapter.load(await fixture('data-graph/invalid.dgraph'))).rejects.toThrow('Invalid Data Graph document');
    adapter.destroy();
  });
});
