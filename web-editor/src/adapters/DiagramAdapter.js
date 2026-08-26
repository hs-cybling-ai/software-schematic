export class DiagramAdapter {
  constructor(container, onEvent) {
    if (new.target === DiagramAdapter) throw new Error('DiagramAdapter is abstract');
    this.container = container;
    this.onEvent = onEvent;
    this.modeler = null;
    this.loading = false;
  }

  async load(_xml) { throw new Error('load must be implemented'); }
  async export() { throw new Error('export must be implemented'); }

  undo() { this.modeler?.get?.('commandStack')?.undo?.(); }
  redo() { this.modeler?.get?.('commandStack')?.redo?.(); }

  destroy() {
    this.modeler?.destroy?.();
    this.modeler = null;
  }

  emitChanged() {
    if (!this.loading) this.onEvent({ type: 'changed', payload: {} });
  }
}
