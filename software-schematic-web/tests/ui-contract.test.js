import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const html = readFileSync(resolve(root, 'index.html'), 'utf8');
const css = readFileSync(resolve(root, 'src/styles.css'), 'utf8');
const main = readFileSync(resolve(root, 'src/main.js'), 'utf8');

describe('browser workspace contract', () => {
  it('contains retained diagram, metadata, documentation, and autosave surfaces', () => {
    for (const id of ['tabs', 'breadcrumbs', 'canvases', 'element-id', 'element-label', 'element-type', 'documentation-path', 'markdown-rendered', 'markdown-source', 'save-status']) {
      expect(html).toContain(`id="${id}"`);
    }
  });

  it('defines responsive, focus, reduced-motion, pending, and failure treatments', () => {
    expect(css).toContain('@media (max-width: 1120px)');
    expect(css).toContain('@media (prefers-reduced-motion: reduce)');
    expect(css).toContain(':focus-visible');
    expect(css).toContain('.save-status.pending');
    expect(css).toContain('.save-status.failed');
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

  it('labels tabs and diagram metadata from composition folder identity', () => {
    expect(main).toContain('tabButton.innerHTML = `<span class="tab-dot"></span><span>${identity.name}</span>`');
    expect(main).toContain("$('#element-label').value = business?.name || identity.name");
    expect(main).toContain("$('#element-label').readOnly = !business");
  });

  it('keeps root main open and gives auxiliary tabs separate close controls with cleanup', () => {
    expect(main).toContain('if (!isRootDiagram(path))');
    expect(main).toContain("closeButton.className = 'tab-close'");
    expect(main).toContain('if (!tab || isRootDiagram(tab.path)');
    expect(main).toContain('await flushDiagram(tab)');
    expect(main).toContain('tab.modeler.destroy()');
    expect(main).toContain('tab.nodeStatuses.clear()');
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

  it('provides transient accessible node status controls and colors', () => {
    expect(html).toContain('id="node-status"');
    expect(html).toContain('id="node-status-meaning"');
    for (const status of ['open', 'new', 'locked', 'modify']) {
      expect(css).toContain(`.node-status-${status}`);
    }
    expect(main).toContain('activeTab.nodeStatuses.set(selectedElement.id, status)');
    expect(main).toContain("gfx?.setAttribute('aria-label'");
  });

  it('auto-places flow nodes with half an activity width between them', () => {
    expect(main).toContain('const DEFAULT_ACTIVITY_WIDTH = 100');
    expect(main).toContain('const DEFAULT_NODE_GAP = DEFAULT_ACTIVITY_WIDTH / 2');
    expect(main).toContain('source.x + source.width + DEFAULT_NODE_GAP + shape.width / 2');
    expect(main).toContain("eventBus.on('autoPlace', 1500");
  });
});
