import { DiagramAdapter } from './DiagramAdapter.js';
import { DataGraphModeler } from '../data-graph/DataGraphModeler.js';
import { MarkdownInspector } from '../data-graph/MarkdownInspector.js';

export class DataGraphAdapter extends DiagramAdapter {
  constructor(container, onEvent, Modeler = DataGraphModeler) {
    super(container, onEvent);
    this.layout = document.createElement('div'); this.layout.className = 'data-graph-layout';
    this.canvas = document.createElement('div'); this.canvas.className = 'data-graph-canvas-host';
    this.inspectorElement = document.createElement('aside'); this.inspectorElement.setAttribute('aria-label', 'Node Markdown context');
    this.layout.append(this.canvas, this.inspectorElement); container.append(this.layout);
    this.modeler = new Modeler({ container: this.canvas });
    this.pendingInspectorTimer = null;
    this.inspector = new MarkdownInspector(this.inspectorElement, { onCommit: (id, value) => { if (this.modeler.nodeMarkdown(id) !== value) this.modeler.setNodeMarkdown(id, value); }, onCapture: () => this.onEvent({ type: 'captureRequested', payload: { diagram: this.modeler.saveJSONSync(), ...this.modeler.contextSnapshot() } }), onClose: () => this.modeler.get('canvas')?.focus?.() });
    this.modeler.on?.('commandStack.changed', () => this.emitChanged());
    const selectedContextNode = elements => elements?.find(element => ['objectNode', 'edgeNode', 'mutationNode', 'property'].includes(element.businessObject?.type));
    this.modeler.on?.('element.click', event => {
      const selected = selectedContextNode([event.element]); if (!selected) return;
      globalThis.clearTimeout(this.pendingInspectorTimer);
      // Leave enough time for element.dblclick to claim the gesture for label
      // editing before a single click opens and resizes the context inspector.
      this.pendingInspectorTimer = globalThis.setTimeout(() => {
        this.pendingInspectorTimer = null;
        this.inspector.open(selected, this.modeler.nodeMarkdown(selected.id));
      }, 240);
    });
    this.modeler.on?.('element.dblclick', () => { globalThis.clearTimeout(this.pendingInspectorTimer); this.pendingInspectorTimer = null; });
    this.modeler.on?.('selection.changed', event => {
      // Opening the inspector changes the canvas width. Wait for element.click so a
      // pointer-down selection cannot invalidate an in-progress diagram-js drag.
      if (this.inspectorElement.hidden) return;
      const selected = selectedContextNode(event.newSelection);
      if (selected) this.inspector.open(selected, this.modeler.nodeMarkdown(selected.id));
    });
  }

  async load(json, contexts = []) {
    this.loading = true;
    try {
      const result = await this.modeler.importJSON(json);
      this.modeler.importContextDrafts?.(contexts);
      if (result?.warnings?.length) this.onEvent({ type: 'warnings', payload: { warnings: result.warnings.map(String) } });
      return result;
    } finally { this.loading = false; }
  }

  async export() {
    this.inspector.flush();
    const result = await this.modeler.saveJSON();
    if (typeof result?.json !== 'string') throw new Error('Data Graph serializer returned no JSON');
    return result.json;
  }

  destroy() { globalThis.clearTimeout(this.pendingInspectorTimer); this.inspector?.destroy(); super.destroy(); this.layout?.remove(); }
  captureCompleted(payload) { this.modeler.contextStore.revision = payload.revision; this.inspector.setCaptureStatus(`Captured revision ${payload.revision}`); }
  captureFailed(payload) { this.inspector.setCaptureStatus(payload?.message ?? 'Capture failed'); }
}
