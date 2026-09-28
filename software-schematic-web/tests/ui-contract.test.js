import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const html = readFileSync(resolve(root, 'index.html'), 'utf8');
const css = readFileSync(resolve(root, 'src/styles.css'), 'utf8');
const main = readFileSync(resolve(root, 'src/main.js'), 'utf8');
const assistantProtocol = readFileSync(resolve(root, 'src/assistant.js'), 'utf8');
const adapters = readFileSync(resolve(root, 'src/diagram-adapters.ts'), 'utf8');
const statusRendering = readFileSync(resolve(root, 'src/status-rendering.js'), 'utf8');
const saveStatus = readFileSync(resolve(root, 'src/save-status.js'), 'utf8');

describe('browser workspace contract', () => {
  it('uses the defined status renderer when applying assistant status operations', () => {
    expect(assistantProtocol).toContain("operation.type === 'set_node_status'");
    expect(main).toContain('renderStatus: applyNodeStatus');
    expect(`${main}\n${assistantProtocol}`).not.toContain('applyStatusMarker(');
  });

  it('hydrates status presentation after initial import and every existing-tab reimport', () => {
    expect(main).toContain('imported = await adapter.importXML(xml)');
    expect(main).toContain('hydrateNodeStatuses(tab)');
    expect(main.match(/adapter\.importXML\(/g)).toHaveLength(1);
    expect(main.match(/importTabXML\(/g)).toHaveLength(4);
    expect(statusRendering).toContain('const imported = await tab.adapter.importXML(xml)');
    expect(statusRendering).toContain('hydrateNodeStatuses(tab)');
  });

  it('contains retained diagram, metadata, documentation, and autosave surfaces', () => {
    for (const id of ['tabs', 'breadcrumbs', 'canvases', 'element-id', 'element-label', 'element-type', 'element-name', 'node-status', 'markdown-rendered', 'markdown-source', 'save-status']) {
      expect(html).toContain(`id="${id}"`);
    }
  });

  it('makes Play create a scoped semantic-versioned plan after every pending save', () => {
    expect(html).toContain('id="play-build"');
    expect(html).toContain('id="build-status"');
    expect(html).toContain('id="build-version-controls"');
    for (const bump of ['major', 'minor', 'fix']) expect(html).toContain(`data-build-bump="${bump}"`);
    expect(html).toContain('Build code from the current model');
    expect(css).toContain('.play-button');
    expect(main).toContain('for (const tab of tabs.values()) await flushDiagram(tab)');
    expect(main).toContain('await queue.waitForAll()');
    expect(main).toContain('await publishBrowserSession()');
    expect(main).toContain("api(runtimePath('/api/runtime/build-plans')");
    expect(main).toContain('semanticBump: selectedBuildBump');
    expect(main).toContain('currentBuildRequest.manifest.scopeLabel');
    expect(main).toContain('currentBuildRequest.manifest.version');
  });

  it('shows generated implementation contracts separately with persistent drift status', () => {
    expect(html).toContain('id="implementation-contract"');
    expect(html).toContain('id="contract-drift"');
    expect(main).toContain("'-contract.md'");
    expect(main).toContain('Differs from logical design');
    expect(css).toContain('.contract-drift');
  });

  it('discovers and verifies the editor operation registry before diagram assistance', () => {
    expect(main).toContain("api('/api/assistant/capabilities')");
    expect(main).toContain('assistantCapabilities.version !== ASSISTANT_SCHEMA_VERSION');
    expect(main).toContain('Diagram assistant operation registry does not match the editor');
  });

  it('opens chat-originated proposals in the shared preview and revalidates before approval', () => {
    expect(main).toContain("api('/api/assistant/inbox')");
    expect(main).toContain('await reviewSubmittedProposal(inbox[0])');
    expect(main).toContain('assistant.proposal = validateProposal(submission.proposal, snapshot)');
    expect(main).toContain('approveProposal(assistant.proposal, assistant.snapshot, current.sourceRevision)');
    expect(main).toContain("api('/api/assistant/inbox-dismissals'");
  });

  it('uses fresh two-phase assistant invocations with explicit suggestion generation', () => {
    for (const id of ['assistant-thread', 'assistant-transcript', 'assistant-prompt', 'assistant-submit', 'assistant-preview', 'assistant-close']) expect(html).toContain(`id="${id}"`);
    expect(main).toContain("api('/api/assistant/conversations'");
    expect(main).toContain("api('/api/assistant/proposals'");
    expect(main).toContain("setAssistantPhase('interview')");
    expect(main).toContain("setAssistantPhase('preview')");
    expect(main).toContain("role: 'proposalSummary'");
    expect(main).toContain("suggest.id = 'assistant-suggest'");
    expect(main).toContain("continueButton.id = 'assistant-continue'");
    expect(main).toContain("approveButton.id = 'assistant-approve'");
    expect(main).not.toContain('localStorage');
    expect(main).not.toContain('assistantInterviewKey');
    expect(html).not.toContain('id="assistant-cancel"');
    expect(html).not.toContain('id="assistant-reject"');
  });

  it('keeps a responsive chat composer anchored below an independently scrolling transcript', () => {
    expect(html).toContain('class="assistant-composer-wrap"');
    expect(html).toContain('class="assistant-send-button"');
    expect(html).toContain('Enter to send · Shift+Enter for a new line');
    expect(css).toMatch(/\.assistant-dialog\s*\{[^}]*grid-template-rows:\s*auto auto minmax\(0, 1fr\) auto/);
    expect(css).toMatch(/\.assistant-thread\s*\{[^}]*overflow-y:\s*auto/);
    expect(css).toMatch(/\.assistant-composer-wrap\s*\{[^}]*border-top:/);
    expect(css).toContain('@media (max-width: 720px), (max-height: 640px)');
  });

  it('uses an accessible corner control for send and stop with guarded keyboard submission', () => {
    expect(html).toContain('title="Send message" aria-label="Send message"');
    expect(css).toMatch(/\.assistant-send-button\s*\{[^}]*position:\s*absolute[^}]*width:\s*42px[^}]*height:\s*42px/);
    expect(main).toContain("submit.dataset.mode = stopping ? 'stop' : 'send'");
    expect(main).toContain("submit.title = stopping ? 'Stop generating' : 'Send message'");
    expect(main).toContain("replaceIcon(submit, stopping ? 'square' : 'arrow-up')");
    expect(main).toContain('assistantComposerIntent(event, event.currentTarget.value, assistant.phase)');
    expect(main).toContain("if (intent === 'send')");
    expect(main).toContain("intent === 'empty'");
    expect(main).toContain('controller?.abort()');
  });

  it('places suggestion actions under responses and proposals inline in outlined cards', () => {
    expect(main).toContain("suggest.innerHTML = '<i data-lucide=\"sparkles\"></i><span>Suggest changes</span>'");
    expect(main).toContain("title.textContent = 'Suggested updates'");
    expect(main).toContain("actions.className = 'assistant-preview-actions'");
    expect(main).toContain("continueButton.textContent = 'Continue interview'");
    expect(main).toContain("approveButton.textContent = 'Approve changes'");
    expect(css).toMatch(/\.assistant-preview\s*\{[^}]*border:\s*1px solid/);
    expect(css).toContain('.assistant-response-actions');
    expect(css).toContain('.assistant-preview-actions');
  });

  it('preserves local work and requires an explicit revision-conflict resolution', () => {
    expect(html).toContain('id="revision-conflict"');
    expect(html).toContain('id="conflict-reload"');
    expect(html).toContain('id="conflict-reapply"');
    expect(html).toContain('id="revision-conflict-preview"');
    expect(main).toContain("detail?.code === 'revisionConflict'");
    expect(main).toContain('conflict.localContent');
    expect(main).toContain('await readFile(conflict.path)');
    expect(main).toContain('await queue.enqueue(conflict.path, conflict.localContent)');
    expect(main).toContain('mergeMarkdown(error.baseContent, error.localContent, current.content)');
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
    expect(saveStatus).toContain("Saved; graph updated");
    expect(saveStatus).toContain("Saved; MCP not running");
    expect(saveStatus).toContain("Saved; graph update failed");
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
    expect(statusRendering).toContain("gfx?.setAttribute('aria-label'");
    expect(main).not.toContain('!element.waypoints && !element.labelTarget');
  });

  it('keeps the assistant modal above BPMN palettes and context pads', () => {
    expect(css).toMatch(/\.modal-backdrop\s*\{[^}]*z-index:\s*10000/);
  });

  it('keeps CMMN assistant approval transactional and supports explicit revert', () => {
    expect(assistantProtocol).toContain('createdCompositions: []');
    expect(assistantProtocol).toContain('openedTabs: []');
    expect(assistantProtocol).toContain('processRenames: []');
    expect(main).toContain('async function rollbackAssistantChange');
    expect(main).toContain("api('/api/schematic-revision')");
    expect(main).toContain("api('/api/file-deletes'");
    expect(main).toContain("api('/api/composition-reverts'");
    expect(assistantProtocol).toContain('The schematic changed after this assistant proposal');
    expect(main).toContain("$('#assistant-revert').addEventListener('click'");
    expect(main).toContain("showToast('Assistant change reverted')");
    for (const operation of [
      'replace_node_type', 'update_node_label', 'update_node_name', 'set_node_status', 'set_process_reference',
      'create_process', 'open_process', 'rename_process', 'add_flow_node', 'connect_sequence_flow',
      'add_participant', 'connect_message_flow', 'move_element', 'remove_element', 'disconnect_flow',
      'add_plan_item', 'connect_cmmn', 'replace_diagram_markdown', 'replace_node_markdown', 'replace_edge_markdown',
    ]) expect(`${main}\n${assistantProtocol}`).toContain(`'${operation}'`);
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
