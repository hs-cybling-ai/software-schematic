import BpmnModeler from 'bpmn-js/lib/Modeler';
import 'bpmn-js/dist/assets/diagram-js.css';
import 'bpmn-js/dist/assets/bpmn-js.css';
import 'bpmn-js/dist/assets/bpmn-font/css/bpmn.css';
import { DiagramAdapter } from './DiagramAdapter.js';

export class BpmnAdapter extends DiagramAdapter {
  constructor(container, onEvent, Modeler = BpmnModeler) {
    super(container, onEvent);
    this.modeler = new Modeler({ container });
    this.modeler.on?.('commandStack.changed', () => this.emitChanged());
  }

  async load(xml) {
    this.loading = true;
    try {
      const result = await this.modeler.importXML(xml);
      this.modeler.get?.('canvas')?.zoom?.('fit-viewport');
      if (result?.warnings?.length) this.onEvent({ type: 'warnings', payload: { warnings: result.warnings.map(String) } });
      return result;
    } finally {
      this.loading = false;
    }
  }

  async export() {
    const result = await this.modeler.saveXML({ format: true });
    if (typeof result?.xml !== 'string') throw new Error('BPMN serializer returned no XML');
    return result.xml;
  }
}
