import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const html = readFileSync(resolve(root, 'index.html'), 'utf8');
const css = readFileSync(resolve(root, 'src/styles.css'), 'utf8');
const main = readFileSync(resolve(root, 'src/main.js'), 'utf8');
const adapters = readFileSync(resolve(root, 'src/diagram-adapters.ts'), 'utf8');

describe('browser workspace contract', () => {
  it('uses the defined status renderer when applying assistant status operations', () => {
    expect(main).toContain("operation.type === 'set_node_status'");
    expect(main).toContain('applyNodeStatus(tab, element)');
    expect(main).not.toContain('applyStatusMarker(');
  });

  it('contains retained diagram, metadata, documentation, and autosave surfaces', () => {
    for (const id of ['tabs', 'breadcrumbs', 'canvases', 'element-id', 'element-label', 'element-type', 'element-name', 'node-status', 'markdown-rendered', 'markdown-source', 'save-status']) {
      expect(html).toContain(`id="${id}"`);
    }
  });

  it('exposes one Name and no derived identity or documentation path fields', () => {
    expect(html).toContain('id="element-name"');
    expect(html).not.toContain('id="qualified-name"');
    expect(html).not.toContain('id="documentation-path"');
    expect(html).not.toContain('External process');
    expect(html).toContain('Implementation Status');
  });

  it('defines responsive, focus, reduced-motion, pending, and failure treatments', () => {
    expect(css).toContain('@media (max-width: 1120px)');
    expect(css).toContain('@media (prefers-reduced-motion: reduce)');
    expect(css).toContain(':focus-visible');
    expect(css).toContain('.save-status.pending');
    expect(css).toContain('.save-status.failed');
    expect(css).toContain('.save-status.stale');
    expect(css).toContain('.save-status.refresh-failed');
    expect(css).toContain('.djs-direct-editing-content');
    expect(css).toContain('color: #111827 !important');
  });

  it('declares no CDN or package-registry runtime assets', () => {
    expect(html).not.toMatch(/https?:\/\//);
    expect(html).not.toMatch(/(?:unpkg|jsdelivr|npmjs)/);
  });

  it('routes BPMN drilldown into retained composition tabs and overrides pool creation', () => {
    expect(main).toContain("closest?.('.bjs-drilldown')");
    expect(main).toContain("openElementComposition(element)");
    expect(main).toContain("'create.participant-expanded'");
    expect(main).toContain('modeling.createShape(participant');
  });

  it('requires a process Name in a modal before opening an unnamed subprocess', () => {
    expect(html).toContain('id="process-name-modal"');
    expect(html).toContain('id="process-name-input"');
    expect(html).toContain('placeholder="Process"');
    expect(main).toContain('const name = await requestProcessName(element)');
    expect(main).toContain('resolveCmmnElementName(name');
    expect(main).toContain('resolveBpmnElementName(name');
    expect(main).toContain('pendingProcessName.adapter?.updateName');
    expect(adapters).toContain("calledElement: resolveBpmnElementName(name, { diagramPath: path, reusable: true })");
    expect(main).not.toContain('globalThis.prompt');
  });

  it('labels tabs and diagram metadata from composition folder identity', () => {
    expect(main).toContain('tabButton.innerHTML = `<span class="tab-dot"></span><span>${identity.name}</span>`');
    expect(main).toContain('const elementLabel = activeTab.adapter.elementLabel(selectedElement)');
    expect(main).toContain("$('#element-label').value = selectedElement ? elementLabel : ''");
    expect(main).toContain("$('#element-label').readOnly = !selectedElement");
    expect(main).toContain("$('#element-label').addEventListener('input'");
    expect(main).toContain('resolveBpmnElementName(authoredName');
    expect(main).toContain('resolveCmmnElementName(authoredName');
    expect(main).toContain("$('#element-name').addEventListener('input'");
    expect(main).toContain("selectedElement ? tab.adapter.elementLabel(selectedElement) : ''");
    expect(adapters).toContain("business?.$type === 'cmmndi:CMMNEdge' && business.cmmnElementRef");
  });

  it('keeps the selected CMMN or legacy root open and gives auxiliary tabs separate close controls with cleanup', () => {
    expect(main).toContain('if (!isRootDiagram(path))');
    expect(main).toContain("closeButton.className = 'tab-close'");
    expect(main).toContain('if (!tab || isRootDiagram(tab.path)');
    expect(main).toContain('await flushDiagram(tab)');
    expect(main).toContain('tab.adapter.destroy()');
    expect(main).toContain('tab.nodeStatuses.clear()');
    expect(main).toContain("projectAnchorPath = selectProjectAnchor(await api('/api/diagrams'))");
    expect(main).toContain("Saved; graph updated");
    expect(main).toContain("Saved; MCP not running");
    expect(main).toContain("Saved; graph update failed");
    expect(main).toContain('await openDiagram(projectAnchorPath)');
  });

  it('adds CMMN as a lazy business-anchor adapter with local composition controls', () => {
    expect(html).toContain('id="new-cmmn"');
    expect(html).toContain('id="return-to-anchor"');
    expect(main).toContain("cmmn: [cmmnAssistantModule]");
    expect(main).toContain("kind: 'cmmn', package_name: packageName");
    expect(main).toContain("activeTab.adapter.kind === 'cmmn'");
    expect(adapters).toContain("await import('cmmn-js/lib/Modeler')");
    expect(adapters).toContain("import('cmmn-font/dist/css/cmmn.css')");
  });

  it('uses actual fullscreen state and refits the active canvas', () => {
    expect(main).toContain('app.requestFullscreen()');
    expect(main).toContain('document.exitFullscreen()');
    expect(main).toContain("document.addEventListener('fullscreenchange', syncFullscreenControl)");
    expect(main).toContain("replaceIcon(button, fullscreen ? 'minimize-2' : 'maximize-2')");
    expect(main).toContain("canvas?.zoom('fit-viewport')");
  });

  it('toggles Markdown between pencil editing and book reading actions', () => {
    expect(main).toContain("const label = editingMarkdown ? 'Read Markdown' : 'Edit Markdown'");
    expect(main).toContain("replaceIcon(button, editingMarkdown ? 'book-open' : 'pencil-line')");
    expect(main).toContain('await flushMarkdown()');
    expect(main).toContain("await renderMarkdown($('#markdown-source').value)");
  });

  it('persists accessible node status controls through the modeler command stack', () => {
    expect(html).toContain('id="node-status"');
    expect(html).toContain('id="node-status-meaning"');
    for (const status of ['open', 'new', 'locked', 'modify']) {
      expect(css).toContain(`.node-status-${status}`);
    }
    expect(main).toContain('activeTab.nodeStatuses.set(selectedElement.id, status)');
    expect(main).toContain('activeTab.adapter.updateStatus(selectedElement, status)');
    expect(adapters).toContain('implementationStatus: normalizeNodeStatus(status)');
    expect(main).toContain("gfx?.setAttribute('aria-label'");
    expect(main).not.toContain('!element.waypoints && !element.labelTarget');
  });

  it('keeps the assistant modal above BPMN palettes and context pads', () => {
    expect(css).toMatch(/\.modal-backdrop\s*\{[^}]*z-index:\s*10000/);
  });

  it('keeps CMMN assistant approval transactional and supports explicit revert', () => {
    expect(main).toContain('const before = { diagrams: new Map(), documents: new Map()');
    expect(main).toContain("for (const [path, xml] of before.diagrams) if (xml && tabs.has(path)) await tabs.get(path).adapter.importXML(xml)");
    expect(main).toContain("$('#assistant-revert').addEventListener('click'");
    expect(main).toContain("showToast('Assistant change reverted')");
  });

  it('cleans up a partially initialized modeler when diagram import fails', () => {
    expect(main).toContain('adapter?.destroy()');
    expect(main).toContain('container.remove()');
    expect(main).toContain('Could not open ${path}');
  });

  it('auto-places flow nodes with half an activity width between them', () => {
    expect(main).toContain('const DEFAULT_ACTIVITY_WIDTH = 100');
    expect(main).toContain('const DEFAULT_NODE_GAP = DEFAULT_ACTIVITY_WIDTH / 2');
    expect(main).toContain('source.x + source.width + DEFAULT_NODE_GAP + shape.width / 2');
    expect(main).toContain("eventBus.on('autoPlace', 1500");
  });
});
