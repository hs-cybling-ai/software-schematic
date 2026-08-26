import { BpmnAdapter } from './adapters/BpmnAdapter.js';
import { DataGraphAdapter } from './adapters/DataGraphAdapter.js';
import { envelope, errorPayload, validateCommand } from './protocol.js';

const defaultFactories = {
  bpmn: (container, emit) => new BpmnAdapter(container, emit),
  dataGraph: (container, emit) => new DataGraphAdapter(container, emit)
};

export class EditorHost {
  constructor(container, postMessage, factories = defaultFactories) {
    this.container = container;
    this.postMessage = postMessage;
    this.factories = factories;
    this.adapter = null;
    this.format = null;
    this.ready = true;
    this.exportInFlight = false;
    this.emit('ready');
  }

  emit(type, details = {}) {
    this.postMessage(envelope(type, { format: this.format, ...details }));
  }

  async receive(rawMessage) {
    let message;
    try {
      message = validateCommand(typeof rawMessage === 'string' ? JSON.parse(rawMessage) : rawMessage);
      if (message.type !== 'load' && message.type !== 'appearance' && !this.adapter) throw new Error('No diagram is loaded');

      switch (message.type) {
        case 'load': return await this.load(message);
        case 'requestExport': return await this.export(message);
        case 'undo': return this.adapter.undo();
        case 'redo': return this.adapter.redo();
        case 'appearance': return document.documentElement.dataset.appearance = message.payload?.appearance === 'light' ? 'light' : 'dark';
        case 'captureCompleted': return this.adapter.captureCompleted?.(message.payload);
        case 'captureFailed': return this.adapter.captureFailed?.(message.payload);
      }
    } catch (error) {
      this.emit('failed', { requestId: message?.requestId ?? null, payload: errorPayload(error) });
    }
  }

  async load(message) {
    const factory = this.factories[message.format];
    if (!factory) throw new Error(`No adapter for ${message.format}`);
    this.adapter?.destroy();
    this.container.replaceChildren();
    this.format = message.format;
    this.adapter = factory(this.container, event => this.emit(event.type, { payload: event.payload }));
    if (message.payload.contexts) await this.adapter.load(message.payload.xml, message.payload.contexts);
    else await this.adapter.load(message.payload.xml);
    this.emit('ready', { requestId: message.requestId });
  }

  async export(message) {
    if (this.exportInFlight) throw new Error('An export is already in progress');
    this.exportInFlight = true;
    try {
      const xml = await this.adapter.export();
      this.emit('exported', { requestId: message.requestId, payload: { xml } });
    } finally {
      this.exportInFlight = false;
    }
  }

  destroy() { this.adapter?.destroy(); }
}
