import Modeler from 'bpmn-js/lib/Modeler';
import 'bpmn-js/dist/assets/diagram-js.css';
import 'bpmn-js/dist/assets/bpmn-font/css/bpmn.css';
import MarkdownIt from 'markdown-it';
import DOMPurify from 'dompurify';
import mermaid from 'mermaid';
import { createIcons, BookOpen, ExternalLink, Maximize2, Minimize2, PencilLine, X } from 'lucide';
import { compositionBreadcrumbs, compositionIdentity, compositionPathFor, compositionSlug, documentationPath, isRootDiagram, NODE_STATUSES, normalizeCompositionPath, normalizeNodeStatus, projectDocumentTitle, RevisionQueue } from './core.js';
import { buildContextSnapshot, isAssistantEligible, proposalGroups, stableRevision, validateProposal } from './assistant.js';
import './styles.css';

const md = new MarkdownIt({ html: false, linkify: true, typographer: true });
const defaultFence = md.renderer.rules.fence;
md.renderer.rules.fence = (tokens, index, options, env, self) => {
  const token = tokens[index];
  if (token.info.trim() === 'mermaid') return `<pre class="mermaid">${md.utils.escapeHtml(token.content)}</pre>`;
  return defaultFence(tokens, index, options, env, self);
};
mermaid.initialize({ startOnLoad: false, theme: 'dark', securityLevel: 'strict', themeVariables: { primaryColor: '#24324a', primaryTextColor: '#eef4ff', lineColor: '#8da2c5', fontFamily: 'Inter, ui-sans-serif, system-ui' } });
const icons = { BookOpen, ExternalLink, Maximize2, Minimize2, PencilLine, X };
createIcons({ icons });

const $ = (selector) => document.querySelector(selector);
const tabs = new Map();
let activeTab = null;
let selectedElement = null;
let editingMarkdown = false;
let markdownTimer = null;
let assistantProvider = 'fake';
const queue = new RevisionQueue(writeFile, setSaveState);

const DEFAULT_ACTIVITY_WIDTH = 100;
const DEFAULT_NODE_GAP = DEFAULT_ACTIVITY_WIDTH / 2;

function CompositionAutoPlaceProvider(eventBus) {
  eventBus.on('autoPlace', 1500, ({ source, shape }) => {
    const sourceBusiness = source.businessObject;
    const shapeBusiness = shape.businessObject;
    if (!sourceBusiness?.$instanceOf?.('bpmn:FlowNode') || !shapeBusiness?.$instanceOf?.('bpmn:FlowNode')) return;

    return {
      x: source.x + source.width + DEFAULT_NODE_GAP + shape.width / 2,
      y: source.y + source.height / 2,
    };
  });
}
CompositionAutoPlaceProvider.$inject = ['eventBus'];

function CompositionPaletteProvider(palette, canvas, elementFactory, modeling) {
  this.getPaletteEntries = () => ({
    'ai.diagram-assistant': {
      group: 'tools', className: 'ai-assistant-entry', title: 'Suggest changes to the complete diagram',
      action: { click: (event) => window.dispatchEvent(new CustomEvent('ssw:assistant', { detail: { scope: 'diagram', invoker: event.currentTarget || event.target?.closest?.('.entry') || event.target } })) },
    },
    'create.participant-expanded': {
      group: 'collaboration',
      className: 'bpmn-icon-participant',
      title: 'Create pool/participant',
      action: {
        click: createPool,
        dragstart: createPool,
      },
    },
  });

  function createPool(event) {
    event.preventDefault?.();
    event.stopPropagation?.();
    const root = canvas.getRootElement();
    const viewbox = canvas.viewbox();
    const participant = elementFactory.createParticipantShape();
    modeling.createShape(participant, {
      x: viewbox.x + viewbox.width / 2,
      y: viewbox.y + viewbox.height / 2,
    }, root);
    canvas.zoom('fit-viewport');
  }

  // Run after the stock provider so this entry replaces its drop-constrained action.
  palette.registerProvider(500, this);
}
CompositionPaletteProvider.$inject = ['palette', 'canvas', 'elementFactory', 'modeling'];

function AssistantContextPadProvider(contextPad) {
  this.getContextPadEntries = (element) => isAssistantEligible(element) ? {
    'ai.node-assistant': {
      group: 'edit', className: 'ai-assistant-entry', title: `Suggest changes for ${businessLabel(element)}`,
      action: { click: (event) => window.dispatchEvent(new CustomEvent('ssw:assistant', { detail: { scope: 'node', elementId: element.id, invoker: event.currentTarget || event.target?.closest?.('.entry') || event.target } })) },
    },
  } : {};
  contextPad.registerProvider(500, this);
}
AssistantContextPadProvider.$inject = ['contextPad'];

const compositionPaletteModule = {
  __init__: ['compositionPaletteProvider', 'compositionAutoPlaceProvider', 'assistantContextPadProvider'],
  compositionPaletteProvider: ['type', CompositionPaletteProvider],
  compositionAutoPlaceProvider: ['type', CompositionAutoPlaceProvider],
  assistantContextPadProvider: ['type', AssistantContextPadProvider],
};

async function api(path, options = {}) {
  const response = await fetch(path, options);
  if (!response.ok) {
    let message = `${response.status} ${response.statusText}`;
    try { message = (await response.json()).error || message; } catch {}
    const error = new Error(message);
    error.status = response.status;
    throw error;
  }
  if (response.status === 204) return null;
  const type = response.headers.get('content-type') || '';
  return type.includes('application/json') ? response.json() : response.text();
}

async function readFile(path, optional = false) {
  try { return await api(`/api/file?path=${encodeURIComponent(path)}`); }
  catch (error) { if (optional && error.status === 404) return ''; throw error; }
}

function writeFile(path, content, revision) {
  return api('/api/file', { method: 'PUT', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ path, content, revision }) });
}

function setSaveState(state, error) {
  const control = $('#save-status');
  control.className = `save-status ${state}`;
  control.lastElementChild.textContent = state === 'pending' ? 'Saving…' : state === 'failed' ? 'Save failed' : 'Saved';
  if (error) showToast(error.message);
}

function showToast(message) {
  const toast = $('#toast');
  toast.textContent = message;
  toast.classList.remove('hidden');
  clearTimeout(showToast.timer);
  showToast.timer = setTimeout(() => toast.classList.add('hidden'), 4800);
}

function replaceIcon(button, name) {
  button.innerHTML = `<i data-lucide="${name}"></i>`;
  createIcons({ icons });
}

function isStatusEligible(element) {
  const business = element?.businessObject;
  return Boolean(element && !element.waypoints && !element.labelTarget && business?.$type && !['bpmn:Process', 'bpmn:Collaboration', 'bpmn:Definitions'].includes(business.$type));
}

function statusFor(tab, element) {
  return normalizeNodeStatus(tab?.nodeStatuses.get(element?.id));
}

function applyNodeStatus(tab, element) {
  if (!tab || !isStatusEligible(element)) return;
  const canvas = tab.modeler.get('canvas');
  Object.keys(NODE_STATUSES).forEach((status) => canvas.removeMarker(element, `node-status-${status}`));
  const status = statusFor(tab, element);
  const definition = NODE_STATUSES[status];
  canvas.addMarker(element, `node-status-${status}`);
  const gfx = tab.modeler.get('elementRegistry').getGraphics(element);
  gfx?.setAttribute('aria-label', `${businessLabel(element)}. ${definition.label}. ${definition.meaning}`);
}

function businessLabel(element) {
  const business = element?.businessObject;
  return business?.name || business?.id || business?.$type?.replace('bpmn:', '') || 'Diagram node';
}

function enhanceAssistantEntries(container) {
  container.querySelectorAll('.ai-assistant-entry').forEach((entry) => {
    entry.setAttribute('role', 'button');
    entry.setAttribute('tabindex', '0');
    entry.setAttribute('aria-label', entry.title || 'AI diagram assistant');
    if (entry.dataset.keyboardReady) return;
    entry.dataset.keyboardReady = 'true';
    entry.addEventListener('keydown', (event) => {
      if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); entry.click(); }
    });
  });
}

async function openDiagram(path) {
  if (tabs.has(path)) return activateTab(tabs.get(path));
  const xml = await readFile(path);
  const container = document.createElement('div');
  container.className = 'diagram-canvas hidden';
  container.dataset.path = path;
  $('#canvases').append(container);
  const modeler = new Modeler({ container, additionalModules: [compositionPaletteModule] });
  await modeler.importXML(xml);
  enhanceAssistantEntries(container);
  const tabElement = document.createElement('div');
  tabElement.className = 'tab';
  const tabButton = document.createElement('button');
  tabButton.className = 'tab-select';
  tabButton.type = 'button';
  tabButton.role = 'tab';
  const identity = compositionIdentity(path);
  tabButton.title = identity.displayPath;
  tabButton.innerHTML = `<span class="tab-dot"></span><span>${identity.name}</span>`;
  tabButton.addEventListener('click', () => activateTab(tab));
  tabElement.append(tabButton);
  const tab = { path, modeler, container, tabElement, tabButton, nodeStatuses: new Map(), diagramTimer: null };
  if (!isRootDiagram(path)) {
    const closeButton = document.createElement('button');
    closeButton.className = 'tab-close';
    closeButton.type = 'button';
    closeButton.title = `Close ${identity.name}`;
    closeButton.setAttribute('aria-label', `Close ${identity.name}`);
    closeButton.innerHTML = '<i data-lucide="x"></i>';
    closeButton.addEventListener('click', (event) => { event.stopPropagation(); closeTab(tab); });
    tabElement.append(closeButton);
    tab.closeButton = closeButton;
  }
  $('#tabs').append(tabElement);
  createIcons({ icons });
  tabs.set(path, tab);

  const eventBus = modeler.get('eventBus');
  eventBus.on('contextPad.open', () => globalThis.requestAnimationFrame(() => enhanceAssistantEntries(container)));
  container.addEventListener('click', (event) => {
    const button = event.target.closest?.('.bjs-drilldown');
    if (!button) return;
    const elementId = button.closest('.djs-overlays')?.dataset.containerId;
    const element = elementId ? modeler.get('elementRegistry').get(elementId) : null;
    if (!element || !['bpmn:CallActivity', 'bpmn:SubProcess'].includes(element.businessObject?.$type)) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    openElementComposition(element);
  }, true);
  eventBus.on('selection.changed', ({ newSelection }) => { if (activeTab === tab) selectElement(newSelection[0] || null); });
  eventBus.on('commandStack.changed', () => scheduleDiagramSave(tab));
  eventBus.on('element.dblclick', 5000, (event) => {
    const { element } = event;
    if (['bpmn:CallActivity', 'bpmn:SubProcess'].includes(element.businessObject?.$type)) {
      event.preventDefault();
      event.stopPropagation();
      openElementComposition(element);
    }
  });
  return activateTab(tab);
}

async function activateTab(tab) {
  if (activeTab === tab) return;
  for (const item of tabs.values()) {
    item.container.classList.toggle('hidden', item !== tab);
    item.tabElement.classList.toggle('active', item === tab);
    item.tabButton.setAttribute('aria-selected', item === tab ? 'true' : 'false');
  }
  activeTab = tab;
  selectedElement = null;
  renderBreadcrumbs(tab.path);
  tab.modeler.get('canvas').resized();
  tab.modeler.get('canvas').zoom('fit-viewport');
  await updateInspector();
}

async function closeTab(tab) {
  if (!tab || isRootDiagram(tab.path) || !tabs.has(tab.path)) return false;
  const ordered = [...tabs.values()];
  const index = ordered.indexOf(tab);
  const wasActive = activeTab === tab;
  try {
    await flushDiagram(tab);
    if (wasActive) await flushMarkdown();
  } catch (error) {
    showToast(`Could not close ${compositionIdentity(tab.path).name}: ${error.message}`);
    return false;
  }
  tabs.delete(tab.path);
  clearTimeout(tab.diagramTimer);
  tab.nodeStatuses.clear();
  tab.modeler.destroy();
  tab.container.remove();
  tab.tabElement.remove();
  if (wasActive) {
    activeTab = null;
    selectedElement = null;
    const next = ordered[index + 1] || ordered[index - 1] || tabs.get('main.bpmn');
    if (next && next !== tab) await activateTab(next);
  }
  return true;
}

function renderBreadcrumbs(diagramPath) {
  const container = $('#breadcrumbs');
  container.replaceChildren();
  const items = compositionBreadcrumbs(diagramPath);
  items.forEach((item, index) => {
    if (index > 0) {
      const separator = document.createElement('span');
      separator.className = 'breadcrumb-separator';
      separator.textContent = '/';
      separator.setAttribute('aria-hidden', 'true');
      container.append(separator);
    }
    const crumb = document.createElement('button');
    crumb.type = 'button';
    crumb.className = 'breadcrumb';
    crumb.textContent = item.name;
    if (index === items.length - 1) {
      crumb.classList.add('current');
      crumb.setAttribute('aria-current', 'page');
    } else {
      crumb.addEventListener('click', () => openDiagram(item.diagramPath));
    }
    container.append(crumb);
  });
}

function selectElement(element) {
  selectedElement = element;
  updateInspector().catch((error) => showToast(error.message));
}

async function updateInspector() {
  if (!activeTab) return;
  const business = selectedElement?.businessObject;
  const identity = compositionIdentity(activeTab.path);
  $('#element-id').value = business?.id || '';
  $('#element-label').value = business?.name || identity.name;
  $('#element-type').value = business?.$type?.replace('bpmn:', '') || 'Diagram';
  const docPath = documentationPath(activeTab.path, business?.id);
  $('#documentation-path').value = docPath;
  $('#documentation-title').textContent = business?.name || business?.id || identity.name;
  const isCall = business?.$type === 'bpmn:CallActivity';
  const isSubProcess = business?.$type === 'bpmn:SubProcess';
  const composable = isCall || isSubProcess || business?.$type === 'bpmn:Participant' || business?.$type === 'bpmn:Lane';
  $('#called-element-field').classList.toggle('hidden', !(isCall || isSubProcess));
  $('#open-composition').classList.toggle('hidden', !composable);
  $('#called-element').value = isCall ? business.calledElement || '' : isSubProcess && business.name ? compositionSlug(business.name) : '';
  $('#called-element').readOnly = isSubProcess;
  $('#element-id').disabled = !business;
  $('#element-label').disabled = false;
  $('#element-label').readOnly = !business;
  const statusEligible = isStatusEligible(selectedElement);
  $('#node-status-field').classList.toggle('hidden', !statusEligible);
  if (statusEligible) {
    const status = statusFor(activeTab, selectedElement);
    $('#node-status').value = status;
    $('#node-status-meaning').textContent = NODE_STATUSES[status].meaning;
    applyNodeStatus(activeTab, selectedElement);
  }
  const markdown = await readFile(docPath, true);
  $('#markdown-source').value = markdown;
  await renderMarkdown(markdown);
}

async function renderMarkdown(source) {
  $('#markdown-rendered').innerHTML = DOMPurify.sanitize(md.render(source));
  try { await mermaid.run({ nodes: $('#markdown-rendered').querySelectorAll('.mermaid') }); }
  catch (error) { showToast(`Mermaid: ${error.message}`); }
}

function scheduleDiagramSave(tab = activeTab) {
  if (!tab) return;
  clearTimeout(tab.diagramTimer);
  setSaveState('pending');
  tab.diagramTimer = setTimeout(() => flushDiagram(tab).catch((error) => setSaveState('failed', error)), 450);
}

async function flushDiagram(tab) {
  if (!tab) return;
  if (tab.diagramTimer) {
    clearTimeout(tab.diagramTimer);
    tab.diagramTimer = null;
    const { xml } = await tab.modeler.saveXML({ format: true });
    await queue.enqueue(tab.path, xml);
  }
  await queue.waitFor(tab.path);
}

function scheduleMarkdownSave() {
  clearTimeout(markdownTimer);
  setSaveState('pending');
  markdownTimer = setTimeout(() => flushMarkdown().catch((error) => setSaveState('failed', error)), 500);
}

async function flushMarkdown() {
  clearTimeout(markdownTimer);
  markdownTimer = null;
  const path = $('#documentation-path').value;
  if (!path) return;
  await queue.enqueue(path, $('#markdown-source').value);
}

async function updateBusinessProperty(property, value) {
  if (!selectedElement) return;
  activeTab.modeler.get('modeling').updateProperties(selectedElement, { [property]: value || undefined });
}

const assistant = { scope: null, element: null, invoker: null, controller: null, snapshot: null, proposal: null, lastBeforeState: null };

function assistantError(message = '') {
  const control = $('#assistant-error');
  control.textContent = message;
  control.classList.toggle('hidden', !message);
}

function closeAssistant() {
  assistant.controller?.abort();
  assistant.controller = null;
  assistant.proposal = null;
  $('#assistant-modal').classList.add('hidden');
  const invoker = assistant.invoker;
  assistant.invoker = null;
  invoker?.focus?.();
}

function openAssistant({ scope, elementId, invoker }) {
  const element = elementId ? activeTab?.modeler.get('elementRegistry').get(elementId) : null;
  if (scope === 'node' && !isAssistantEligible(element)) return;
  assistant.scope = scope;
  assistant.element = element;
  assistant.invoker = invoker || document.activeElement;
  assistant.snapshot = null;
  assistant.proposal = null;
  assistantError();
  $('#assistant-preview').classList.add('hidden');
  $('#assistant-preview').replaceChildren();
  $('#assistant-submit').classList.remove('hidden');
  $('#assistant-approve').classList.add('hidden');
  $('#assistant-reject').classList.add('hidden');
  $('#assistant-prompt').value = '';
  $('#assistant-scope').textContent = scope === 'node' ? `Node: ${businessLabel(element)} in ${compositionIdentity(activeTab.path).displayPath}` : `Complete diagram: ${compositionIdentity(activeTab.path).displayPath}`;
  $('#assistant-disclosure').textContent = `Provider: ${assistantProvider}. It receives active diagram structure and relevant Markdown. No credential is sent to the browser.`;
  $('#assistant-modal').classList.remove('hidden');
  $('#assistant-prompt').focus();
}

window.addEventListener('ssw:assistant', (event) => openAssistant(event.detail));
$('#assistant-close').addEventListener('click', closeAssistant);
$('#assistant-cancel').addEventListener('click', closeAssistant);
$('#assistant-modal').addEventListener('click', (event) => { if (event.target === $('#assistant-modal')) closeAssistant(); });
document.addEventListener('keydown', (event) => { if (event.key === 'Escape' && !$('#assistant-modal').classList.contains('hidden')) closeAssistant(); });

async function currentAssistantSnapshot() {
  await flushDiagram(activeTab);
  await flushMarkdown();
  const diagramDoc = documentationPath(activeTab.path);
  const nodeDoc = assistant.element ? documentationPath(activeTab.path, assistant.element.id) : '';
  const snapshot = await buildContextSnapshot({
    scope: assistant.scope, tab: activeTab, primaryNode: assistant.element,
    diagramMarkdown: await readFile(diagramDoc, true),
    nodeMarkdown: nodeDoc ? await readFile(nodeDoc, true) : '',
    flush: async () => {},
  });
  return snapshot;
}

function renderProposal(proposal) {
  const preview = $('#assistant-preview');
  preview.replaceChildren();
  const title = document.createElement('h3'); title.textContent = proposal.summary || 'Suggested changes'; preview.append(title);
  for (const [path, descriptions] of proposalGroups(proposal)) {
    const heading = document.createElement('h4'); heading.textContent = path; preview.append(heading);
    const list = document.createElement('ul');
    descriptions.forEach((description) => { const item = document.createElement('li'); item.textContent = description; list.append(item); });
    preview.append(list);
  }
  for (const [label, values] of [['Assumptions', proposal.assumptions], ['Warnings', proposal.warnings]]) {
    if (!values?.length) continue;
    const heading = document.createElement('h4'); heading.textContent = label; preview.append(heading);
    const list = document.createElement('ul'); values.forEach((value) => { const item = document.createElement('li'); item.textContent = value; list.append(item); }); preview.append(list);
  }
  preview.classList.remove('hidden');
}

$('#assistant-submit').addEventListener('click', async () => {
  const prompt = $('#assistant-prompt').value.trim();
  if (!prompt) return assistantError('Describe the diagram or documentation change you want.');
  const button = $('#assistant-submit'); button.disabled = true; button.textContent = 'Generating…'; assistantError();
  const controller = new AbortController(); assistant.controller?.abort(); assistant.controller = controller;
  try {
    const snapshot = await currentAssistantSnapshot();
    snapshot.requestId = globalThis.crypto?.randomUUID?.() || `request-${Date.now()}`;
    // Bind correlation into the revision without making the random request ID part of content staleness.
    assistant.snapshot = snapshot;
    const result = await api('/api/assistant/proposals', { method: 'POST', headers: { 'content-type': 'application/json' }, signal: controller.signal, body: JSON.stringify({ requestId: snapshot.requestId, prompt, snapshot }) });
    if (assistant.controller !== controller || controller.signal.aborted) return;
    assistant.proposal = validateProposal(result.proposal, snapshot);
    renderProposal(assistant.proposal);
    button.classList.add('hidden'); $('#assistant-approve').classList.remove('hidden'); $('#assistant-reject').classList.remove('hidden');
  } catch (error) { if (error.name !== 'AbortError') assistantError(error.message); }
  finally { if (assistant.controller === controller) assistant.controller = null; button.disabled = false; button.textContent = 'Generate proposal'; }
});

$('#assistant-reject').addEventListener('click', closeAssistant);

async function applyAssistantProposal(proposal) {
  const before = { diagrams: new Map(), documents: new Map(), revision: assistant.snapshot.sourceRevision };
  const diagramPaths = new Set([activeTab.path, ...proposal.operations.map((op) => op.diagramPath).filter(Boolean)]);
  for (const path of diagramPaths) {
    const tab = tabs.get(path);
    if (tab) before.diagrams.set(path, (await tab.modeler.saveXML({ format: true })).xml);
  }
  const markdownOps = proposal.operations.filter((op) => op.type === 'replace_diagram_markdown' || op.type === 'replace_node_markdown');
  for (const operation of markdownOps) {
    const path = operation.path || (operation.type === 'replace_node_markdown' ? documentationPath(operation.diagramPath, operation.nodeId) : documentationPath(operation.diagramPath));
    before.documents.set(path, await readFile(path, true));
  }
  try {
    for (const operation of proposal.operations.filter((op) => op.type === 'create_composition' || op.type === 'open_composition')) {
      const result = await api('/api/compositions', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ path: operation.path }) });
      await openDiagram(result.diagram);
      if (!before.diagrams.has(result.diagram)) before.diagrams.set(result.diagram, null);
    }
    for (const operation of proposal.operations) {
      if (['create_composition', 'open_composition'].includes(operation.type)) continue;
      const tab = tabs.get(operation.diagramPath || assistant.snapshot.diagramPath) || activeTab;
      const registry = tab.modeler.get('elementRegistry'); const modeling = tab.modeler.get('modeling');
      if (operation.type === 'replace_node_type') tab.modeler.get('bpmnReplace').replaceElement(registry.get(operation.nodeId), { type: operation.bpmnType });
      else if (operation.type === 'update_node_label') modeling.updateProperties(registry.get(operation.nodeId), { name: operation.label });
      else if (operation.type === 'set_composition_link') modeling.updateProperties(registry.get(operation.nodeId), { calledElement: operation.path });
      else if (operation.type === 'add_flow_node') {
        const elementFactory = tab.modeler.get('elementFactory'); const root = tab.modeler.get('canvas').getRootElement();
        const shape = elementFactory.createShape({ type: operation.bpmnType, id: operation.nodeId });
        const created = modeling.createShape(shape, { x: operation.x || 180, y: operation.y || 160 }, root);
        if (operation.label) modeling.updateProperties(created, { name: operation.label });
      } else if (operation.type === 'connect_sequence_flow') modeling.connect(registry.get(operation.sourceId), registry.get(operation.targetId), { type: 'bpmn:SequenceFlow', id: operation.flowId });
      else if (operation.type === 'replace_diagram_markdown' || operation.type === 'replace_node_markdown') {
        const path = operation.path || (operation.type === 'replace_node_markdown' ? documentationPath(operation.diagramPath, operation.nodeId) : documentationPath(operation.diagramPath));
        await queue.enqueue(path, operation.markdown);
      }
    }
    for (const tab of tabs.values()) if (diagramPaths.has(tab.path) || before.diagrams.has(tab.path)) await flushDiagram(tab);
    assistant.lastBeforeState = before;
    $('#assistant-revert').classList.remove('hidden');
  } catch (error) {
    for (const [path, xml] of before.diagrams) if (xml && tabs.has(path)) await tabs.get(path).modeler.importXML(xml);
    for (const [path, content] of before.documents) await queue.enqueue(path, content);
    throw error;
  }
}

$('#assistant-approve').addEventListener('click', async () => {
  const button = $('#assistant-approve'); button.disabled = true; assistantError();
  try {
    const current = await currentAssistantSnapshot();
    validateProposal(assistant.proposal, assistant.snapshot, { currentRevision: current.sourceRevision });
    await applyAssistantProposal(assistant.proposal);
    await updateInspector();
    closeAssistant(); showToast('Assistant changes applied. Use diagram undo or Revert assistant change if needed.');
  } catch (error) { assistantError(error.message); }
  finally { button.disabled = false; }
});

$('#assistant-revert').addEventListener('click', async () => {
  const before = assistant.lastBeforeState;
  if (!before) return;
  try {
    for (const [path, xml] of before.diagrams) if (xml && tabs.has(path)) { await tabs.get(path).modeler.importXML(xml); await queue.enqueue(path, xml); }
    for (const [path, content] of before.documents) await queue.enqueue(path, content);
    assistant.lastBeforeState = null;
    $('#assistant-revert').classList.add('hidden');
    await updateInspector();
    showToast('Assistant change reverted');
  } catch (error) { showToast(`Could not revert assistant change: ${error.message}`); }
});

$('#element-label').addEventListener('change', (event) => updateBusinessProperty('name', event.target.value));
$('#element-id').addEventListener('change', async (event) => {
  const oldId = selectedElement?.businessObject?.id;
  const newId = event.target.value.trim();
  if (!oldId || newId === oldId) return;
  if (!/^[A-Za-z0-9_-]+$/.test(newId)) return showToast('IDs may contain letters, digits, underscore, and hyphen');
  try {
    const nodeStatus = normalizeNodeStatus(activeTab?.nodeStatuses.get(oldId));
    await api('/api/rename-documentation', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ diagram_path: activeTab.path, old_id: oldId, new_id: newId }) });
    await updateBusinessProperty('id', newId);
    if (nodeStatus !== 'open') {
      activeTab.nodeStatuses.delete(oldId);
      activeTab.nodeStatuses.set(newId, nodeStatus);
    }
    await updateInspector();
  } catch (error) { event.target.value = oldId; showToast(error.message); }
});
$('#called-element').addEventListener('change', (event) => {
  if (selectedElement?.businessObject?.$type !== 'bpmn:CallActivity') return;
  try { updateBusinessProperty('calledElement', normalizeCompositionPath(event.target.value)); }
  catch (error) { showToast(error.message); }
});
$('#node-status').addEventListener('change', (event) => {
  if (!activeTab || !isStatusEligible(selectedElement)) return;
  const status = normalizeNodeStatus(event.target.value);
  if (status === 'open') activeTab.nodeStatuses.delete(selectedElement.id);
  else activeTab.nodeStatuses.set(selectedElement.id, status);
  $('#node-status-meaning').textContent = NODE_STATUSES[status].meaning;
  applyNodeStatus(activeTab, selectedElement);
});

async function openElementComposition(element = selectedElement) {
  try {
    const path = compositionPathFor(element, activeTab.path);
    const result = await api('/api/compositions', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ path }) });
    await openDiagram(result.diagram);
  } catch (error) { showToast(error.message); }
}

$('#open-composition').addEventListener('click', () => openElementComposition());
function syncFullscreenControl() {
  const fullscreen = Boolean(document.fullscreenElement);
  const button = $('#fit-view');
  const label = fullscreen ? 'Exit full screen' : 'Enter full screen';
  button.title = label;
  button.setAttribute('aria-label', label);
  button.setAttribute('aria-pressed', String(fullscreen));
  replaceIcon(button, fullscreen ? 'minimize-2' : 'maximize-2');
  globalThis.requestAnimationFrame(() => {
    const canvas = activeTab?.modeler.get('canvas');
    canvas?.resized();
    canvas?.zoom('fit-viewport');
  });
}

$('#fit-view').addEventListener('click', async () => {
  try {
    if (document.fullscreenElement) {
      if (!document.exitFullscreen) throw new Error('Exiting full screen is not supported by this browser');
      await document.exitFullscreen();
    } else {
      const app = $('#app');
      if (!app.requestFullscreen) throw new Error('Full screen is not supported by this browser');
      await app.requestFullscreen();
    }
  } catch (error) { showToast(error.message); syncFullscreenControl(); }
});
document.addEventListener('fullscreenchange', syncFullscreenControl);
document.addEventListener('fullscreenerror', () => showToast('The browser could not change full screen mode'));

function syncMarkdownControl() {
  const button = $('#toggle-markdown');
  const label = editingMarkdown ? 'Read Markdown' : 'Edit Markdown';
  button.title = label;
  button.setAttribute('aria-label', label);
  button.setAttribute('aria-pressed', String(editingMarkdown));
  button.classList.toggle('active', editingMarkdown);
  replaceIcon(button, editingMarkdown ? 'book-open' : 'pencil-line');
}

$('#toggle-markdown').addEventListener('click', async () => {
  try {
    if (editingMarkdown) {
      await flushMarkdown();
      await renderMarkdown($('#markdown-source').value);
      editingMarkdown = false;
    } else {
      editingMarkdown = true;
    }
    $('#markdown-source').classList.toggle('hidden', !editingMarkdown);
    $('#markdown-rendered').classList.toggle('hidden', editingMarkdown);
    syncMarkdownControl();
    if (editingMarkdown) $('#markdown-source').focus();
  } catch (error) { showToast(error.message); }
});
$('#markdown-source').addEventListener('input', scheduleMarkdownSave);

async function initialize() {
  document.title = 'Software Schematic';
  try {
    const metadata = await api('/api/project');
    document.title = projectDocumentTitle(metadata?.name);
    assistantProvider = metadata?.assistant_provider || 'fake';
  } catch {}
  await openDiagram('main.bpmn');
}

syncMarkdownControl();
initialize().catch((error) => showToast(error.message));
