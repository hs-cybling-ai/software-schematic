import 'bpmn-js/dist/assets/diagram-js.css';
import 'bpmn-js/dist/assets/bpmn-font/css/bpmn.css';
import MarkdownIt from 'markdown-it';
import DOMPurify from 'dompurify';
import mermaid from 'mermaid';
import { createIcons, ArrowUp, BookOpen, ExternalLink, Maximize2, Minimize2, PencilLine, Sparkles, Square, X } from 'lucide';
import { architecturalName, cmmnFolderForPackageName, cmmnPathForPackageName, compositionBreadcrumbs, compositionFolderForQualifiedName, compositionIdentity, compositionPathFor, diagramKind, documentationPath, isRootDiagram, mergeMarkdown, NODE_STATUSES, normalizeNodeStatus, owningProcessName, packageNameForCmmnPath, projectDocumentTitle, qualifiedSymbolFor, resolveBpmnElementName, resolveCmmnElementName, RevisionQueue, selectProjectAnchor, validatePackageName, validateQualifiedProcessName } from './core.js';
import { createDiagramAdapter } from './diagram-adapters.ts';
import { approveProposal, assistantComposerIntent, assertRevertRevision, ASSISTANT_SCHEMA_VERSION, buildContextSnapshot, createProposalBeforeState, DIAGRAM_OPERATION_REGISTRY, executeModelerOperation, isAssistantEligible, proposalGroups, stableRevision, validateProposal } from './assistant.js';
import { applyNodeStatus, hydrateNodeStatuses, importTabXML, isStatusEligible as isTabStatusEligible, statusFor } from './status-rendering.js';
import { saveStatusPresentation } from './save-status.js';
import './styles.css';

const md = new MarkdownIt({ html: false, linkify: true, typographer: true });
const defaultFence = md.renderer.rules.fence;
md.renderer.rules.fence = (tokens, index, options, env, self) => {
  const token = tokens[index];
  if (token.info.trim() === 'mermaid') return `<pre class="mermaid">${md.utils.escapeHtml(token.content)}</pre>`;
  return defaultFence(tokens, index, options, env, self);
};
mermaid.initialize({ startOnLoad: false, theme: 'dark', securityLevel: 'strict', themeVariables: { primaryColor: '#24324a', primaryTextColor: '#eef4ff', lineColor: '#8da2c5', fontFamily: 'Inter, ui-sans-serif, system-ui' } });
const icons = { ArrowUp, BookOpen, ExternalLink, Maximize2, Minimize2, PencilLine, Sparkles, Square, X };
createIcons({ icons });

const $ = (selector) => document.querySelector(selector);
const tabs = new Map();
let activeTab = null;
let selectedElement = null;
let editingMarkdown = false;
let markdownTimer = null;
let graphRefreshTimer = null;
let lastSaveStatusToastKey = null;
let assistantProvider = null;
let assistantCapabilities = null;
let projectAnchorPath = 'main.cmmn';
let activeRevisionConflict = null;
const daemonParams = new URLSearchParams(location.search);
const daemon = daemonParams.get('daemonToken') ? {
  token: daemonParams.get('daemonToken'), projectId: daemonParams.get('projectId'), generation: daemonParams.get('generation'),
  sessionId: globalThis.crypto?.randomUUID?.() || `browser-${Date.now()}`, lastEventId: 0, timer: null,
} : null;
let runtimeProposal = null;
let currentBuildRequest = null;
let selectedBuildBump = null;
let selectedBuildBaseVersion = null;
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
  if (contextPad.registerProvider.length === 1) contextPad.registerProvider(this);
  else contextPad.registerProvider(500, this);
}
AssistantContextPadProvider.$inject = ['contextPad'];

const compositionPaletteModule = {
  __init__: ['compositionPaletteProvider', 'compositionAutoPlaceProvider', 'assistantContextPadProvider'],
  compositionPaletteProvider: ['type', CompositionPaletteProvider],
  compositionAutoPlaceProvider: ['type', CompositionAutoPlaceProvider],
  assistantContextPadProvider: ['type', AssistantContextPadProvider],
};

function CmmnAssistantPaletteProvider(palette) {
  this.getPaletteEntries = () => ({
    'ai.diagram-assistant': {
      group: 'tools', className: 'ai-assistant-entry', title: 'Suggest changes to the complete business-need diagram',
      action: { click: (event) => window.dispatchEvent(new CustomEvent('ssw:assistant', { detail: { scope: 'diagram', invoker: event.currentTarget || event.target?.closest?.('.entry') || event.target } })) },
    },
  });
  if (palette.registerProvider.length === 1) palette.registerProvider(this);
  else palette.registerProvider(500, this);
}
CmmnAssistantPaletteProvider.$inject = ['palette'];

const cmmnAssistantModule = {
  __init__: ['cmmnAssistantPaletteProvider', 'assistantContextPadProvider'],
  cmmnAssistantPaletteProvider: ['type', CmmnAssistantPaletteProvider],
  assistantContextPadProvider: ['type', AssistantContextPadProvider],
};

async function api(path, options = {}) {
  const response = await fetch(path, options);
  if (!response.ok) {
    let message = `${response.status} ${response.statusText}`;
    let detail = null;
    try { detail = await response.json(); message = detail.error || message; } catch {}
    const error = new Error(message);
    error.status = response.status;
    if (detail) Object.assign(error, detail);
    throw error;
  }
  if (response.status === 204) return null;
  const type = response.headers.get('content-type') || '';
  return type.includes('application/json') ? response.json() : response.text();
}

function runtimePath(path, extra = {}) {
  const query = new URLSearchParams({ token: daemon.token, projectId: daemon.projectId, generation: daemon.generation, ...extra });
  return `${path}?${query}`;
}

async function publishBrowserSession() {
  if (!daemon) return;
  const documents = Object.fromEntries([...queue.baseRevisions.entries()].filter(([, revision]) => revision && revision !== 'missing'));
  const dirtyDocuments = queue.pendingPaths();
  for (const tab of tabs.values()) if (tab.diagramTimer && !dirtyDocuments.includes(tab.path)) dirtyDocuments.push(tab.path);
  if (markdownTimer && activeTab) dirtyDocuments.push(documentationPath(activeTab.path, activeTab.adapter.elementId(selectedElement)));
  await api(runtimePath('/api/runtime/sessions'), { method: 'PUT', headers: { 'content-type': 'application/json' }, body: JSON.stringify({
    protocolVersion: '1.0', sessionId: daemon.sessionId, daemonGeneration: daemon.generation,
    activeDiagram: activeTab?.path || null, selectedEntityRefs: selectedElement?.id ? [selectedElement.id] : [], documents,
    dirtyDocuments: [...new Set(dirtyDocuments)].sort(), lastEventId: daemon.lastEventId, heartbeatEpochMs: Date.now(),
  }) });
}

async function runtimeDecision(kind, diagnostic = null) {
  if (!runtimeProposal) return null;
  const decision = await api(runtimePath(`/api/runtime/proposals/${encodeURIComponent(runtimeProposal.proposalId)}/decisions`), {
    method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ protocolVersion: '1.0', proposalId: runtimeProposal.proposalId, expectedStateRevision: runtimeProposal.stateRevision, sessionId: daemon.sessionId, kind, diagnostic }),
  });
  runtimeProposal = decision;
  return decision;
}

async function reviewRuntimeProposal(record) {
  if (!record || runtimeProposal?.proposalId === record.proposalId) return;
  await openDiagram(record.diagramPath);
  await openAssistant({ scope: 'diagram', invoker: $('#assistant-revert') });
  const snapshot = await currentAssistantSnapshot();
  snapshot.requestId = record.proposalId;
  assistant.snapshot = snapshot;
  assistant.proposal = validateProposal({ version: record.operationRegistryVersion, requestId: record.proposalId, sourceRevision: snapshot.sourceRevision, summary: record.summary, assumptions: record.assumptions, warnings: record.warnings, operations: record.operations }, snapshot);
  runtimeProposal = record;
  renderProposal(assistant.proposal);
  setAssistantPhase('preview');
}

async function pollRuntime() {
  if (!daemon) return;
  try {
    await publishBrowserSession();
    const text = await api(runtimePath('/api/runtime/events', { after: daemon.lastEventId }));
    for (const block of text.split('\n\n').filter(Boolean)) {
      const id = Number(block.match(/^id: (\d+)/m)?.[1] || 0); const data = block.match(/^data: (.+)$/m)?.[1];
      if (id) daemon.lastEventId = Math.max(daemon.lastEventId, id);
      if (data) { const event = JSON.parse(data); if (event.kind === 'proposalSubmitted' && event.proposalId) await reviewRuntimeProposal(await api(runtimePath(`/api/runtime/proposals/${encodeURIComponent(event.proposalId)}`))); }
    }
    const plans = await api(runtimePath('/api/runtime/build-plans'));
    if (currentBuildRequest?.manifest?.planId) {
      const summary = plans.find((plan) => plan.planId === currentBuildRequest.manifest.planId);
      if (summary) currentBuildRequest = { ...currentBuildRequest, progress: { ...currentBuildRequest.progress, state: summary.state, progressRevision: summary.progressRevision } };
    } else if (plans.length) currentBuildRequest = { manifest: { planId: plans[0].planId, scopeLabel: plans[0].scopeLabel, version: plans[0].version }, progress: { state: plans[0].state, progressRevision: plans[0].progressRevision } };
    renderBuildStatus(currentBuildRequest);
  } catch (error) {
    if ([400, 401, 403].includes(error.status)) showToast('Software Schematic daemon changed. Save your work and reload this page.');
  } finally { daemon.timer = globalThis.setTimeout(pollRuntime, 1000); }
}

function renderBuildStatus(request) {
  const status = $('#build-status');
  const state = request?.progress?.state || request?.state || 'designing';
  const stateLabel = { designing: 'Designing', ready: 'Ready', building: 'Building…', complete: 'Complete', failed: 'Build failed', stale: 'Design changed' }[state] || 'Designing';
  const scopeLabel = request?.manifest?.scopeLabel;
  const version = request?.manifest?.version;
  const label = scopeLabel && version ? `${scopeLabel} v${version} · ${stateLabel}` : stateLabel;
  status.textContent = label;
  status.className = `build-status ${state}`;
  status.title = request?.progress?.diagnostic || request?.diagnostic || label;
}

async function waitForGraphPublication() {
  for (let attempt = 0; attempt < 50; attempt += 1) {
    const status = await api('/api/graph-refresh');
    if (status.status === 'failed' || status.status === 'rejected') throw new Error(status.diagnostic || 'The model could not be published.');
    if (!['queued', 'processing'].includes(status.status)) return status;
    await new Promise((resolve) => globalThis.setTimeout(resolve, 100));
  }
  throw new Error('The model is still publishing. Try Play again in a moment.');
}

$('#play-build').addEventListener('click', async () => {
  if (!daemon) return showToast('Open this project with ./ssw before starting a build.');
  const button = $('#play-build'); button.disabled = true;
  try {
    for (const tab of tabs.values()) await flushDiagram(tab);
    await flushMarkdown();
    await queue.waitForAll();
    await publishBrowserSession();
    await waitForGraphPublication();
    currentBuildRequest = await api(runtimePath('/api/runtime/build-plans'), {
      method: 'POST', headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ diagramPath: activeTab.path, selectedEntityRef: selectedElement?.id || null, semanticBump: selectedBuildBump, expectedPriorVersion: selectedBuildBaseVersion }),
    });
    selectedBuildBump = null;
    selectedBuildBaseVersion = null;
    $('#build-version-controls').classList.add('hidden');
    for (const control of document.querySelectorAll('[data-build-bump]')) control.setAttribute('aria-pressed', 'false');
    renderBuildStatus(currentBuildRequest);
    showToast(`${currentBuildRequest.manifest.scopeLabel} v${currentBuildRequest.manifest.version} is ready to build.`);
  } catch (error) {
    const latest = error.message.match(/latest version (\d+\.\d+\.\d+)/i)?.[1];
    if (latest) selectedBuildBaseVersion = latest;
    if (/choose major, minor, or fix/i.test(error.message)) $('#build-version-controls').classList.remove('hidden');
    showToast(error.message);
  }
  finally { button.disabled = false; }
});

for (const control of document.querySelectorAll('[data-build-bump]')) {
  control.setAttribute('aria-pressed', 'false');
  control.addEventListener('click', () => {
    selectedBuildBump = control.dataset.buildBump;
    for (const peer of document.querySelectorAll('[data-build-bump]')) peer.setAttribute('aria-pressed', String(peer === control));
    $('#play-build').focus();
  });
}

async function readFile(path, optional = false) {
  try {
    const document = await api(`/api/file?path=${encodeURIComponent(path)}`);
    queue.setBaseRevision(path, document.contentRevision);
    queue.setBaseContent(path, document.content);
    return document.content;
  }
  catch (error) {
    if (optional && error.status === 404) { queue.setBaseRevision(path, 'missing'); return ''; }
    throw error;
  }
}

function writeFile(path, content, revision, expectedRevision) {
  return api('/api/file', { method: 'PUT', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ path, content, revision, expectedRevision }) });
}

async function pollGraphRefresh() {
  graphRefreshTimer = null;
  try {
    const [graphRefresh, embedding] = await Promise.all([
      api('/api/graph-refresh'),
      api('/api/embedding-status'),
    ]);
    setSaveState('saved', { graphRefresh, embedding });
  } catch (error) {
    setSaveState('saved', { graphRefresh: { status: 'notRunning', diagnostic: error.message } });
  }
}

function scheduleGraphRefreshPoll() {
  if (graphRefreshTimer) return;
  graphRefreshTimer = globalThis.setTimeout(pollGraphRefresh, 400);
}

function setSaveState(state, detail) {
  const control = $('#save-status');
  const presentation = saveStatusPresentation(state, detail);
  if (presentation.title) control.title = presentation.title;
  else control.removeAttribute('title');
  if (presentation.poll) scheduleGraphRefreshPoll();
  if (state === 'pending') lastSaveStatusToastKey = null;
  if (state === 'failed' && detail?.code === 'revisionConflict') {
    showRevisionConflict(detail).catch((error) => showToast(error.message));
  } else if (presentation.toast && presentation.toastKey !== lastSaveStatusToastKey) {
    lastSaveStatusToastKey = presentation.toastKey;
    showToast(presentation.toast);
  }
  control.className = `save-status ${presentation.displayState}`;
  control.lastElementChild.textContent = presentation.label;
}

async function showRevisionConflict(error) {
  activeRevisionConflict = error;
  $('#revision-conflict-detail').textContent = `${error.path} was updated after this editor loaded it. Your local work is preserved; reload the durable version or explicitly reapply your local version.`;
  $('#revision-conflict-preview').classList.add('hidden');
  $('#conflict-reapply').disabled = false;
  $('#conflict-reapply').textContent = 'Reapply local changes';
  if (error.path.endsWith('.md') && typeof error.baseContent === 'string') {
    const current = await api(`/api/file?path=${encodeURIComponent(error.path)}`);
    const merge = mergeMarkdown(error.baseContent, error.localContent, current.content);
    Object.assign(error, { currentContent: current.content, currentRevision: current.contentRevision, mergedContent: merge.content });
    if (merge.status === 'conflict') {
      $('#revision-conflict-detail').textContent = `${error.path} has overlapping Markdown edits. Local work is preserved; reload current content or resolve the overlap manually.`;
      $('#conflict-reapply').disabled = true;
    } else {
      $('#revision-conflict-preview').textContent = merge.content;
      $('#revision-conflict-preview').classList.remove('hidden');
      $('#conflict-reapply').textContent = merge.status === 'merged' ? 'Apply merged preview' : 'Apply preview';
    }
  }
  $('#revision-conflict').classList.remove('hidden');
}

function closeRevisionConflict() {
  activeRevisionConflict = null;
  $('#revision-conflict').classList.add('hidden');
  $('#revision-conflict-preview').textContent = '';
}

async function reloadConflictedDocument() {
  const conflict = activeRevisionConflict;
  if (!conflict) return;
  const content = await readFile(conflict.path);
  if (conflict.path.endsWith('.bpmn') || conflict.path.endsWith('.cmmn')) {
    const tab = tabs.get(conflict.path);
    if (tab) await importTabXML(tab, content);
  } else if (activeTab && documentationPath(activeTab.path, activeTab.adapter.elementId(selectedElement)) === conflict.path) {
    $('#markdown-source').value = content;
    await renderMarkdown(content);
  }
  closeRevisionConflict();
  setSaveState('saved');
}

async function reapplyConflictedDocument() {
  const conflict = activeRevisionConflict;
  if (!conflict) return;
  if (conflict.currentRevision && conflict.mergedContent !== null) {
    queue.setBaseRevision(conflict.path, conflict.currentRevision);
    queue.setBaseContent(conflict.path, conflict.currentContent);
    await queue.enqueue(conflict.path, conflict.mergedContent);
  } else {
    await readFile(conflict.path);
    await queue.enqueue(conflict.path, conflict.localContent);
  }
  closeRevisionConflict();
}

$('#conflict-reload').addEventListener('click', () => reloadConflictedDocument().catch((error) => showToast(error.message)));
$('#conflict-reapply').addEventListener('click', () => reapplyConflictedDocument().catch((error) => showToast(error.message)));

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

function isStatusEligible(element, tab = activeTab) {
  return isTabStatusEligible(tab, element);
}

function businessLabel(element, tab = activeTab) {
  const adapted = tab?.adapter?.elementLabel(element);
  if (adapted) return adapted;
  const business = element?.businessObject;
  return business?.name || business?.definitionRef?.name || business?.id || business?.$type?.replace(/^[^:]+:/, '') || 'Diagram node';
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

async function openDiagram(path, originPath = null) {
  if (tabs.has(path)) {
    const existing = tabs.get(path);
    if (originPath && !existing.originPath) existing.originPath = originPath;
    return activateTab(existing);
  }
  const xml = await readFile(path);
  const container = document.createElement('div');
  container.className = 'diagram-canvas hidden';
  container.dataset.path = path;
  $('#canvases').append(container);
  let adapter;
  let imported;
  try {
    adapter = await createDiagramAdapter(path, container, { bpmn: [compositionPaletteModule], cmmn: [cmmnAssistantModule] });
    imported = await adapter.importXML(xml);
  } catch (error) {
    adapter?.destroy();
    container.remove();
    throw new Error(`Could not open ${path}: ${error.message}`);
  }
  const modeler = adapter.modeler;
  if (imported?.warnings?.length) showToast(`Opened with ${imported.warnings.length} CMMN import warning${imported.warnings.length === 1 ? '' : 's'}`);
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
  const tab = { path, adapter, modeler, container, tabElement, tabButton, nodeStatuses: new Map(), diagramTimer: null, originPath };
  hydrateNodeStatuses(tab);
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
    if (!element || !adapter.isComposable(element)) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    openElementComposition(element);
  }, true);
  eventBus.on('selection.changed', ({ newSelection }) => { if (activeTab === tab) selectElement(newSelection[0] || null); });
  eventBus.on('commandStack.changed', () => {
    scheduleDiagramSave(tab);
    if (activeTab === tab) $('#element-label').value = selectedElement ? tab.adapter.elementLabel(selectedElement) : '';
  });
  eventBus.on('element.dblclick', 5000, (event) => {
    const { element } = event;
    if (adapter.isComposable(element)) {
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
  $('#return-to-anchor')?.classList.toggle('hidden', !tab.originPath || !tabs.has(tab.originPath));
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
  tab.adapter.destroy();
  tab.container.remove();
  tab.tabElement.remove();
  if (wasActive) {
    activeTab = null;
    selectedElement = null;
    const next = ordered[index + 1] || ordered[index - 1] || tabs.get(projectAnchorPath);
    if (next && next !== tab) await activateTab(next);
  }
  return true;
}

async function discardTab(tab) {
  if (!tab || isRootDiagram(tab.path) || !tabs.has(tab.path)) return false;
  const ordered = [...tabs.values()];
  const index = ordered.indexOf(tab);
  const wasActive = activeTab === tab;
  tabs.delete(tab.path);
  clearTimeout(tab.diagramTimer);
  tab.nodeStatuses.clear();
  tab.adapter.destroy();
  tab.container.remove();
  tab.tabElement.remove();
  if (wasActive) {
    activeTab = null;
    selectedElement = null;
    const next = ordered[index + 1] || ordered[index - 1] || tabs.get(projectAnchorPath);
    if (next && next !== tab) await activateTab(next);
  }
  return true;
}

async function reloadTabsAfterProcessRename(oldQualifiedName, newQualifiedName) {
  const oldPath = `${compositionFolderForQualifiedName(oldQualifiedName)}/main.bpmn`;
  const newPath = `${compositionFolderForQualifiedName(newQualifiedName)}/main.bpmn`;
  const renamed = tabs.get(oldPath);
  if (renamed) {
    tabs.delete(oldPath);
    renamed.path = newPath;
    renamed.container.dataset.path = newPath;
    const identity = compositionIdentity(newPath);
    renamed.tabButton.title = identity.displayPath;
    renamed.tabButton.querySelector('span:last-child').textContent = identity.name;
    tabs.set(newPath, renamed);
  }
  for (const tab of tabs.values()) {
    const xml = await readFile(tab.path);
    await importTabXML(tab, xml);
  }
  if (activeTab) renderBreadcrumbs(activeTab.path);
}

function renderBreadcrumbs(diagramPath) {
  const container = $('#breadcrumbs');
  container.replaceChildren();
  const items = compositionBreadcrumbs(diagramPath, projectAnchorPath);
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
  selectedBuildBump = null;
  selectedBuildBaseVersion = null;
  $('#build-version-controls').classList.add('hidden');
  for (const control of document.querySelectorAll('[data-build-bump]')) control.setAttribute('aria-pressed', 'false');
  updateInspector().catch((error) => showToast(error.message));
}

async function updateInspector() {
  if (!activeTab) return;
  const business = selectedElement?.businessObject;
  const elementId = activeTab.adapter.elementId(selectedElement);
  const elementLabel = activeTab.adapter.elementLabel(selectedElement);
  const identity = compositionIdentity(activeTab.path);
  $('#element-id').value = elementId;
  $('#element-label').value = selectedElement ? elementLabel : '';
  $('#element-type').value = selectedElement ? activeTab.adapter.displayType(selectedElement) : activeTab.adapter.kind === 'cmmn' ? 'CMMN Package' : 'Diagram';
  const childDiagram = !selectedElement && !isRootDiagram(activeTab.path);
  const nameEligible = childDiagram || Boolean(selectedElement && activeTab.adapter.isSelectable(selectedElement));
  $('#element-name-field').classList.toggle('hidden', !nameEligible);
  const diagramQualifiedName = childDiagram ? (activeTab.adapter.kind === 'cmmn' ? packageNameForCmmnPath(activeTab.path) : owningProcessName(activeTab.path)) : '';
  $('#element-name').value = childDiagram ? diagramQualifiedName : activeTab.adapter.elementName(selectedElement);
  const elementName = activeTab.adapter.elementName(selectedElement);
  const docPath = documentationPath(activeTab.path, elementId);
  $('#documentation-title').textContent = elementLabel || elementId || identity.name;
  const composable = activeTab.adapter.isComposable(selectedElement) && Boolean(elementName) && !elementName.includes('#');
  $('#open-composition').classList.toggle('hidden', !composable);
  $('#element-name').disabled = !nameEligible;
  $('#element-id').disabled = !selectedElement;
  $('#element-label').disabled = false;
  $('#element-label').readOnly = !selectedElement;
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
  await renderImplementationContract(docPath, markdown, Boolean(selectedElement));
}

async function renderMarkdown(source) {
  $('#markdown-rendered').innerHTML = DOMPurify.sanitize(md.render(source));
  try { await mermaid.run({ nodes: $('#markdown-rendered').querySelectorAll('.mermaid') }); }
  catch (error) { showToast(`Mermaid: ${error.message}`); }
}

async function renderImplementationContract(logicalPath, logicalMarkdown, hasSelection) {
  const section = $('#implementation-contract');
  section.classList.add('hidden');
  $('#implementation-contract-body').replaceChildren();
  $('#contract-drift').textContent = '';
  if (!hasSelection || !logicalPath.endsWith('.md')) return;
  const contractPath = logicalPath.replace(/\.md$/, '-contract.md');
  const physical = await readFile(contractPath, true);
  if (!physical) return;
  const marker = '<!-- software-schematic-contract\n';
  const endMarker = '\n-->\n\n';
  if (!physical.startsWith(marker) || !physical.includes(endMarker)) return;
  const body = physical.slice(physical.indexOf(endMarker, marker.length) + endMarker.length);
  const differs = body.trim() !== logicalMarkdown.trim();
  $('#contract-drift').textContent = differs ? 'Differs from logical design' : 'Matches logical design';
  $('#implementation-contract-body').innerHTML = DOMPurify.sanitize(md.render(body));
  section.classList.remove('hidden');
  try { await mermaid.run({ nodes: $('#implementation-contract-body').querySelectorAll('.mermaid') }); } catch {}
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
    const { xml } = await tab.adapter.saveXML({ format: true });
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
  const path = activeTab ? documentationPath(activeTab.path, activeTab.adapter.elementId(selectedElement)) : '';
  if (!path) return;
  await queue.enqueue(path, $('#markdown-source').value);
}

async function updateBusinessProperty(property, value) {
  if (!selectedElement) return;
  if (property === 'name') activeTab.adapter.updateLabel(selectedElement, value || undefined);
  else if (property === 'id') activeTab.adapter.updateId(selectedElement, value);
  else activeTab.modeler.get('modeling').updateProperties(selectedElement, { [property]: value || undefined });
}

const assistant = { scope: null, element: null, invoker: null, controller: null, snapshot: null, proposal: null, submissionId: null, lastBeforeState: null, turns: [], invocationId: null, phase: 'closed' };

function renderAssistantTranscript() {
  const transcript = $('#assistant-transcript');
  transcript.replaceChildren();
  let latestAssistantIndex = -1;
  assistant.turns.forEach((turn, index) => { if (turn.role === 'assistant') latestAssistantIndex = index; });
  assistant.turns.forEach((turn, index) => {
    const item = document.createElement('article');
    item.className = 'assistant-turn';
    item.dataset.role = turn.role;
    const label = document.createElement('span');
    label.className = 'assistant-turn-label';
    label.textContent = turn.role === 'user' ? 'You' : turn.role === 'proposalSummary' ? 'Previous suggestion' : 'Design assistant';
    const message = document.createElement('p');
    message.className = 'assistant-turn-message';
    message.textContent = turn.text;
    item.append(label, message);
    if (index === latestAssistantIndex && index === assistant.turns.length - 1 && assistant.phase === 'interview' && !assistant.proposal) {
      const actions = document.createElement('div');
      actions.className = 'assistant-response-actions';
      const suggest = document.createElement('button');
      suggest.id = 'assistant-suggest';
      suggest.type = 'button';
      suggest.className = 'assistant-response-action';
      suggest.title = 'Generate structured updates from this conversation';
      suggest.innerHTML = '<i data-lucide="sparkles"></i><span>Suggest changes</span>';
      actions.append(suggest);
      item.append(actions);
    }
    transcript.append(item);
  });
  if (assistant.phase === 'requestingInterview') {
    const thinking = document.createElement('div');
    thinking.className = 'assistant-turn assistant-thinking';
    thinking.dataset.role = 'assistant';
    thinking.textContent = 'Design assistant is responding…';
    transcript.append(thinking);
  }
  createIcons({ icons });
  const thread = $('#assistant-thread');
  thread.scrollTop = thread.scrollHeight;
}

function assistantError(message = '') {
  const control = $('#assistant-error');
  control.textContent = message;
  control.classList.toggle('hidden', !message);
}

function syncAssistantComposer() {
  const interviewing = assistant.phase === 'interview';
  const stopping = assistant.phase === 'requestingInterview';
  const prompt = $('#assistant-prompt');
  const submit = $('#assistant-submit');
  prompt.disabled = !assistantProvider || (!interviewing && !stopping);
  submit.dataset.mode = stopping ? 'stop' : 'send';
  submit.title = stopping ? 'Stop generating' : 'Send message';
  submit.setAttribute('aria-label', submit.title);
  submit.disabled = stopping ? false : (!assistantProvider || !interviewing || !prompt.value.trim());
  replaceIcon(submit, stopping ? 'square' : 'arrow-up');
}

function setAssistantPhase(phase) {
  assistant.phase = phase;
  $('#assistant-transcript').setAttribute('aria-busy', String(phase === 'requestingInterview' || phase === 'requestingSuggestion'));
  syncAssistantComposer();
  renderAssistantTranscript();
}

function resetAssistantInvocation() {
  assistant.controller?.abort();
  assistant.controller = null;
  assistant.scope = null;
  assistant.element = null;
  assistant.snapshot = null;
  assistant.proposal = null;
  assistant.submissionId = null;
  assistant.turns = [];
  assistant.invocationId = null;
  assistant.phase = 'closed';
  renderAssistantTranscript();
  renderProposal(null);
}

async function discardExternalProposal() {
  if (runtimeProposal) {
    await runtimeDecision('reject');
    runtimeProposal = null;
  }
  await dismissSubmittedProposal();
}

async function closeAssistant({ discardExternal = true } = {}) {
  assistant.controller?.abort();
  if (discardExternal) await discardExternalProposal();
  $('#assistant-modal').classList.add('hidden');
  const invoker = assistant.invoker;
  resetAssistantInvocation();
  assistant.invoker = null;
  invoker?.focus?.();
}

async function openAssistant({ scope, elementId, invoker }) {
  assistant.controller?.abort();
  if (assistant.submissionId || runtimeProposal) await discardExternalProposal();
  const element = elementId ? activeTab?.modeler.get('elementRegistry').get(elementId) : null;
  if (scope === 'node' && !isAssistantEligible(element)) return;
  assistant.scope = scope;
  assistant.element = element;
  assistant.invoker = invoker || document.activeElement;
  assistant.invocationId = globalThis.crypto?.randomUUID?.() || `invocation-${Date.now()}`;
  const invocationId = assistant.invocationId;
  const invocationTab = activeTab;
  assistant.snapshot = null;
  assistant.proposal = null;
  assistant.submissionId = null;
  assistant.turns = [];
  assistantError();
  renderAssistantTranscript();
  renderProposal(null);
  $('#assistant-prompt').value = '';
  $('#assistant-scope').textContent = scope === 'node'
    ? `Node: ${businessLabel(element)} in ${compositionIdentity(activeTab.path).displayPath}`
    : scope === 'edge'
      ? `Edge: ${businessLabel(element)} in ${compositionIdentity(activeTab.path).displayPath}`
      : `Complete diagram: ${compositionIdentity(activeTab.path).displayPath}`;
  $('#assistant-disclosure').textContent = assistantProvider
    ? `Provider: ${assistantProvider}. It receives active diagram structure and relevant Markdown. No credential is sent to the browser.`
    : 'Assistant not configured. Run ./ssw auth login in this project, then restart SSW.';
  $('#assistant-modal').classList.remove('hidden');
  setAssistantPhase('opening');
  try {
    const snapshot = await currentAssistantSnapshot({ scope, element, tab: invocationTab });
    if (assistant.invocationId !== invocationId) return;
    assistant.snapshot = snapshot;
    setAssistantPhase('interview');
    $('#assistant-prompt').focus();
  } catch (error) {
    if (assistant.invocationId !== invocationId) return;
    assistantError(error.message);
    setAssistantPhase('interview');
  }
}

async function reviewSubmittedProposal(submission) {
  const snapshot = { ...submission.request.snapshot, requestId: submission.request.requestId };
  await openDiagram(snapshot.diagramPath);
  const primaryElementId = snapshot.primaryElementId || snapshot.primaryNodeId || snapshot.primaryEdgeId || null;
  if (primaryElementId && !activeTab.modeler.get('elementRegistry').get(primaryElementId)) {
    throw new Error(`Chat proposal target is no longer present: ${primaryElementId}`);
  }
  const scope = snapshot.primaryElementKind || (snapshot.primaryNodeId ? 'node' : snapshot.primaryEdgeId ? 'edge' : 'diagram');
  await openAssistant({ scope, elementId: primaryElementId, invoker: $('#assistant-revert') });
  assistant.snapshot = snapshot;
  assistant.proposal = validateProposal(submission.proposal, snapshot);
  assistant.submissionId = submission.request.requestId;
  renderProposal(assistant.proposal);
  setAssistantPhase('preview');
}

async function dismissSubmittedProposal() {
  if (!assistant.submissionId) return;
  await api('/api/assistant/inbox-dismissals', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ requestId: assistant.submissionId }),
  });
  assistant.submissionId = null;
}

window.addEventListener('ssw:assistant', (event) => openAssistant(event.detail).catch((error) => assistantError(error.message)));
$('#assistant-close').addEventListener('click', () => closeAssistant().catch((error) => assistantError(error.message)));
$('#assistant-modal').addEventListener('click', (event) => { if (event.target === $('#assistant-modal')) closeAssistant().catch((error) => assistantError(error.message)); });
document.addEventListener('keydown', (event) => { if (event.key === 'Escape' && !$('#assistant-modal').classList.contains('hidden')) closeAssistant().catch((error) => assistantError(error.message)); });

async function currentAssistantSnapshot({ scope = assistant.scope, element = assistant.element, tab = activeTab } = {}) {
  await flushDiagram(tab);
  await flushMarkdown();
  const diagramDoc = documentationPath(tab.path);
  const elementDoc = element ? documentationPath(tab.path, element.id) : '';
  const snapshot = await buildContextSnapshot({
    scope, tab, primaryElement: element,
    diagramMarkdown: await readFile(diagramDoc, true),
    nodeMarkdown: scope === 'node' && elementDoc ? await readFile(elementDoc, true) : '',
    edgeMarkdown: scope === 'edge' && elementDoc ? await readFile(elementDoc, true) : '',
    flush: async () => {},
  });
  return snapshot;
}

function renderProposal(proposal) {
  const preview = $('#assistant-preview');
  preview.replaceChildren();
  if (!proposal) {
    preview.classList.add('hidden');
    return;
  }
  const title = document.createElement('h3');
  title.id = 'assistant-preview-title';
  title.textContent = 'Suggested updates';
  preview.append(title);
  const summary = document.createElement('p');
  summary.className = 'assistant-preview-summary';
  summary.textContent = proposal.summary || 'Review the suggested changes below.';
  preview.append(summary);
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
  const actions = document.createElement('footer');
  actions.className = 'assistant-preview-actions';
  const continueButton = document.createElement('button');
  continueButton.id = 'assistant-continue';
  continueButton.type = 'button';
  continueButton.className = 'secondary-button';
  continueButton.textContent = 'Continue interview';
  const approveButton = document.createElement('button');
  approveButton.id = 'assistant-approve';
  approveButton.type = 'button';
  approveButton.className = 'primary-button';
  approveButton.textContent = 'Approve changes';
  actions.append(continueButton, approveButton);
  preview.append(actions);
  preview.classList.remove('hidden');
  const thread = $('#assistant-thread');
  thread.scrollTop = thread.scrollHeight;
}

async function submitAssistantTurn() {
  if (assistant.phase !== 'interview') return;
  const prompt = $('#assistant-prompt').value.trim();
  if (!prompt) return assistantError('Ask a question or describe the design you want to refine.');
  assistant.turns.push({ role: 'user', text: prompt });
  $('#assistant-prompt').value = '';
  setAssistantPhase('requestingInterview'); assistantError();
  const controller = new AbortController(); assistant.controller?.abort(); assistant.controller = controller;
  try {
    const requestId = globalThis.crypto?.randomUUID?.() || `request-${Date.now()}`;
    const result = await api('/api/assistant/conversations', { method: 'POST', headers: { 'content-type': 'application/json' }, signal: controller.signal, body: JSON.stringify({ requestId, snapshot: assistant.snapshot, turns: assistant.turns }) });
    if (assistant.controller !== controller || controller.signal.aborted) return;
    assistant.turns.push({ role: 'assistant', text: result.reply });
  } catch (error) { if (error.name !== 'AbortError') assistantError(error.message); }
  finally { if (assistant.controller === controller) { assistant.controller = null; setAssistantPhase('interview'); } }
}

function stopAssistantResponse() {
  if (assistant.phase !== 'requestingInterview') return;
  const controller = assistant.controller;
  controller?.abort();
  if (assistant.controller === controller) assistant.controller = null;
  setAssistantPhase('interview');
  $('#assistant-prompt').focus();
}

$('#assistant-submit').addEventListener('click', () => {
  if (assistant.phase === 'requestingInterview') stopAssistantResponse();
  else submitAssistantTurn().catch((error) => assistantError(error.message));
});

$('#assistant-prompt').addEventListener('input', syncAssistantComposer);
$('#assistant-prompt').addEventListener('keydown', (event) => {
  const intent = assistantComposerIntent(event, event.currentTarget.value, assistant.phase);
  if (intent === 'send') {
    event.preventDefault();
    submitAssistantTurn().catch((error) => assistantError(error.message));
  } else if (intent === 'empty') event.preventDefault();
});

async function requestAssistantSuggestion() {
  if (assistant.phase !== 'interview' || !assistant.turns.some((turn) => turn.role === 'assistant')) return;
  setAssistantPhase('requestingSuggestion'); assistantError();
  const controller = new AbortController(); assistant.controller?.abort(); assistant.controller = controller;
  try {
    const snapshot = await currentAssistantSnapshot();
    snapshot.requestId = globalThis.crypto?.randomUUID?.() || `request-${Date.now()}`;
    assistant.snapshot = snapshot;
    const result = await api('/api/assistant/proposals', { method: 'POST', headers: { 'content-type': 'application/json' }, signal: controller.signal, body: JSON.stringify({ requestId: snapshot.requestId, prompt: 'Suggest updates from the current invocation context.', snapshot, turns: assistant.turns }) });
    if (assistant.controller !== controller || controller.signal.aborted) return;
    assistant.proposal = validateProposal(result.proposal, snapshot);
    setAssistantPhase('preview');
    renderProposal(assistant.proposal);
  } catch (error) {
    if (error.name !== 'AbortError') assistantError(error.message);
    if (!controller.signal.aborted) setAssistantPhase('interview');
  } finally { if (assistant.controller === controller) assistant.controller = null; }
}

async function continueAssistantInterview() {
  if (!assistant.proposal) return;
  const proposal = assistant.proposal;
  setAssistantPhase('opening');
  try {
    await discardExternalProposal();
    assistant.snapshot = await currentAssistantSnapshot();
    const descriptions = [...proposalGroups(proposal).values()].flat();
    assistant.turns.push({ role: 'proposalSummary', text: [proposal.summary || 'Suggested changes', ...descriptions].join('\n').slice(0, 16 * 1024) });
    assistant.proposal = null;
    renderProposal(null);
    setAssistantPhase('interview');
    $('#assistant-prompt').focus();
  } catch (error) { assistantError(error.message); setAssistantPhase('preview'); }
}

$('#assistant-thread').addEventListener('click', (event) => {
  const button = event.target.closest('button');
  if (button?.id === 'assistant-suggest') requestAssistantSuggestion().catch((error) => assistantError(error.message));
  else if (button?.id === 'assistant-continue') continueAssistantInterview().catch((error) => assistantError(error.message));
  else if (button?.id === 'assistant-approve') approveAssistantChanges(button).catch((error) => assistantError(error.message));
});

async function applyAssistantProposal(proposal) {
  const before = createProposalBeforeState(assistant.snapshot.sourceRevision);
  const diagramPaths = new Set([activeTab.path, ...proposal.operations.map((op) => op.diagramPath).filter(Boolean)]);
  for (const path of diagramPaths) {
    const tab = tabs.get(path);
    if (tab) before.diagrams.set(path, (await tab.adapter.saveXML({ format: true })).xml);
  }
  const markdownOps = proposal.operations.filter((op) => ['replace_diagram_markdown', 'replace_node_markdown', 'replace_edge_markdown'].includes(op.type));
  for (const operation of markdownOps) {
    const path = operation.path || (operation.type === 'replace_node_markdown' ? documentationPath(operation.diagramPath, operation.nodeId) : operation.type === 'replace_edge_markdown' ? documentationPath(operation.diagramPath, operation.edgeId) : documentationPath(operation.diagramPath));
    const content = await readFile(path, true);
    before.documents.set(path, { content, existed: queue.baseRevision(path) !== 'missing' });
  }
  try {
    for (const operation of proposal.operations.filter((op) => op.type === 'create_process' || op.type === 'open_process')) {
      const result = await api('/api/compositions', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ kind: 'bpmn', qualified_name: operation.qualifiedName }) });
      const wasOpen = tabs.has(result.diagram);
      await openDiagram(result.diagram);
      if (result.created) await readFile(result.documentation);
      if (!wasOpen) before.openedTabs.push(result.diagram);
      if (!before.diagrams.has(result.diagram)) before.diagrams.set(result.diagram, result.created ? null : (await tabs.get(result.diagram).adapter.saveXML({ format: true })).xml);
      if (result.created) before.createdCompositions.push({ qualifiedName: operation.qualifiedName, diagramPath: result.diagram, revision: null });
    }
    for (const operation of proposal.operations) {
      if (['create_process', 'open_process'].includes(operation.type)) continue;
      const targetPath = operation.diagramPath || assistant.snapshot.diagramPath;
      const tab = tabs.get(targetPath);
      if (!tab) throw new Error(`Assistant operation targets an unopened diagram: ${targetPath}`);
      if (operation.type === 'rename_process') {
        for (const openTab of tabs.values()) await flushDiagram(openTab);
        await api('/api/process-renames', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ old_qualified_name: operation.oldQualifiedName, new_qualified_name: operation.newQualifiedName }) });
        before.processRenames.push({ oldQualifiedName: operation.oldQualifiedName, newQualifiedName: operation.newQualifiedName });
        await reloadTabsAfterProcessRename(operation.oldQualifiedName, operation.newQualifiedName);
      }
      else if (executeModelerOperation(operation, tab, { renderStatus: applyNodeStatus })) {}
      else if (operation.type === 'replace_diagram_markdown' || operation.type === 'replace_node_markdown' || operation.type === 'replace_edge_markdown') {
        const path = operation.path || (operation.type === 'replace_node_markdown' ? documentationPath(operation.diagramPath, operation.nodeId) : operation.type === 'replace_edge_markdown' ? documentationPath(operation.diagramPath, operation.edgeId) : documentationPath(operation.diagramPath));
        await queue.enqueue(path, operation.markdown);
      }
      else throw new Error(`No editor executor is registered for ${operation.type}`);
    }
    for (const tab of tabs.values()) if (diagramPaths.has(tab.path) || before.diagrams.has(tab.path)) await flushDiagram(tab);
    for (const composition of before.createdCompositions) {
      const result = await api('/api/composition-revisions', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ qualified_name: composition.qualifiedName }) });
      composition.revision = result.revision;
    }
    before.schematicRevision = (await api('/api/schematic-revision')).revision;
    assistant.lastBeforeState = before;
    $('#assistant-revert').classList.remove('hidden');
  } catch (error) {
    try { await rollbackAssistantChange(before, { requireUnchanged: false }); }
    catch (rollbackError) { error.message = `${error.message}; rollback also failed: ${rollbackError.message}`; }
    throw error;
  }
}

function isCreatedCompositionPath(path, before) {
  return before.createdCompositions.some(({ qualifiedName }) => {
    const folder = compositionFolderForQualifiedName(qualifiedName);
    return path === folder || path.startsWith(`${folder}/`);
  });
}

async function rollbackAssistantChange(before, { requireUnchanged = true } = {}) {
  for (const tab of tabs.values()) await flushDiagram(tab);
  await flushMarkdown();

  let currentRevision = (await api('/api/schematic-revision')).revision;
  if (requireUnchanged) assertRevertRevision(before.schematicRevision, currentRevision);

  for (const rename of [...before.processRenames].reverse()) {
    await api('/api/process-renames', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({
        old_qualified_name: rename.newQualifiedName,
        new_qualified_name: rename.oldQualifiedName,
        expected_revision: currentRevision,
      }),
    });
    await reloadTabsAfterProcessRename(rename.newQualifiedName, rename.oldQualifiedName);
    currentRevision = (await api('/api/schematic-revision')).revision;
  }

  for (const [path, xml] of before.diagrams) {
    if (!xml || isCreatedCompositionPath(path, before) || !tabs.has(path)) continue;
    await importTabXML(tabs.get(path), xml);
    await queue.enqueue(path, xml);
  }
  for (const [path, document] of before.documents) {
    if (isCreatedCompositionPath(path, before)) continue;
    if (document.existed) {
      await queue.enqueue(path, document.content);
    } else if (queue.baseRevision(path) !== 'missing') {
      await api('/api/file-deletes', {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ path, expectedRevision: queue.baseRevision(path) }),
      });
      queue.forget(path);
    }
  }

  for (const composition of [...before.createdCompositions].reverse()) {
    const current = await api('/api/composition-revisions', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ qualified_name: composition.qualifiedName }),
    });
    await api('/api/composition-reverts', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ qualified_name: composition.qualifiedName, expected_revision: current.revision }),
    });
    await discardTab(tabs.get(composition.diagramPath));
    queue.forget(composition.diagramPath);
  }
  for (const path of before.openedTabs) {
    if (!isCreatedCompositionPath(path, before)) await discardTab(tabs.get(path));
  }
}

async function approveAssistantChanges(button) {
  if (assistant.phase !== 'preview' || !assistant.proposal) return;
  button.disabled = true; assistantError();
  try {
    const current = await currentAssistantSnapshot();
    const approval = approveProposal(assistant.proposal, assistant.snapshot, current.sourceRevision);
    if (runtimeProposal) await runtimeDecision('approve');
    await applyAssistantProposal(approval.proposal);
    if (runtimeProposal) {
      for (let index = 0; index < 4; index += 1) await runtimeDecision('applied');
      runtimeProposal = null;
    }
    await dismissSubmittedProposal();
    await updateInspector();
    await closeAssistant({ discardExternal: false }); showToast('Assistant changes applied. Use diagram undo or Revert assistant change if needed.');
  } catch (error) {
    if (runtimeProposal) { try { await runtimeDecision('failed', error.message); } catch {} }
    assistantError(error.message);
  }
  finally { button.disabled = false; }
}

$('#assistant-revert').addEventListener('click', async () => {
  const before = assistant.lastBeforeState;
  if (!before) return;
  try {
    await rollbackAssistantChange(before);
    assistant.lastBeforeState = null;
    $('#assistant-revert').classList.add('hidden');
    await updateInspector();
    showToast('Assistant change reverted');
  } catch (error) { showToast(`Could not revert assistant change: ${error.message}`); }
});

$('#element-label').addEventListener('input', (event) => updateBusinessProperty('name', event.target.value));

function validateAuthoredElementName(tab, element, value) {
  const authoredName = String(value || '').trim();
  const reusable = tab.adapter.isComposable(element);
  if (tab.adapter.kind === 'cmmn') {
    resolveCmmnElementName(authoredName, { packageName: tab.adapter.diagramName(tab.path), reusable });
  } else {
    resolveBpmnElementName(authoredName, { diagramPath: tab.path, reusable });
  }
  return authoredName;
}

$('#element-name').addEventListener('input', (event) => {
  if (!activeTab || !selectedElement) return;
  try {
    const authoredName = validateAuthoredElementName(activeTab, selectedElement, event.target.value);
    if (authoredName !== activeTab.adapter.elementName(selectedElement)) activeTab.adapter.updateName(selectedElement, authoredName);
  } catch {}
});

$('#element-name').addEventListener('change', async (event) => {
  const business = selectedElement?.businessObject;
  if (!business && activeTab && !isRootDiagram(activeTab.path)) {
    const isCmmn = activeTab.adapter.kind === 'cmmn';
    const oldQualifiedName = isCmmn ? packageNameForCmmnPath(activeTab.path) : owningProcessName(activeTab.path);
    try {
      const newQualifiedName = isCmmn ? validatePackageName(event.target.value) : validateQualifiedProcessName(event.target.value);
      const approved = globalThis.confirm(`Rename ${oldQualifiedName} to ${newQualifiedName}? This also renames its ${isCmmn ? 'package' : 'composition'} folder and local references.`);
      if (!approved) { event.target.value = oldQualifiedName; return; }
      for (const tab of tabs.values()) await flushDiagram(tab);
      await api(isCmmn ? '/api/package-renames' : '/api/process-renames', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(isCmmn ? { old_package_name: oldQualifiedName, new_package_name: newQualifiedName } : { old_qualified_name: oldQualifiedName, new_qualified_name: newQualifiedName }) });
      const oldFolder = isCmmn ? cmmnFolderForPackageName(oldQualifiedName) : compositionFolderForQualifiedName(oldQualifiedName);
      const newFolder = isCmmn ? cmmnFolderForPackageName(newQualifiedName) : compositionFolderForQualifiedName(newQualifiedName);
      const affectedTabs = [...tabs.values()].filter((tab) => tab.path === `${oldFolder}/main.${isCmmn ? 'cmmn' : 'bpmn'}` || (isCmmn && tab.path.startsWith(`${oldFolder}/`)));
      for (const tab of affectedTabs) {
        const oldPath = tab.path;
        const newPath = oldPath.startsWith(`${oldFolder}/`) ? `${newFolder}/${oldPath.slice(oldFolder.length + 1)}` : oldPath;
        tabs.delete(oldPath);
        tab.path = newPath;
        tab.container.dataset.path = newPath;
        await importTabXML(tab, await readFile(newPath));
        tabs.set(newPath, tab);
        const identity = compositionIdentity(newPath);
        tab.tabButton.title = identity.displayPath;
        tab.tabButton.querySelector('span:last-child').textContent = identity.name;
      }
      renderBreadcrumbs(activeTab.path);
      await updateInspector();
      showToast(`Renamed ${isCmmn ? 'package' : 'process'} to ${newQualifiedName}`);
    } catch (error) {
      event.target.value = oldQualifiedName;
      showToast(error.message);
    }
    return;
  }
  if (!business) return;
  const previous = activeTab.adapter.elementName(selectedElement);
  try {
    const authoredName = validateAuthoredElementName(activeTab, selectedElement, event.target.value);
    if (authoredName !== previous) activeTab.adapter.updateName(selectedElement, authoredName);
    await updateInspector();
  } catch (error) {
    event.target.value = previous;
    showToast(error.message);
  }
});
$('#element-id').addEventListener('change', async (event) => {
  const oldId = activeTab?.adapter?.elementId(selectedElement);
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
$('#node-status').addEventListener('change', (event) => {
  if (!activeTab || !isStatusEligible(selectedElement, activeTab)) return;
  const status = normalizeNodeStatus(event.target.value);
  if (status === 'open') activeTab.nodeStatuses.delete(selectedElement.id);
  else activeTab.nodeStatuses.set(selectedElement.id, status);
  activeTab.adapter.updateStatus(selectedElement, status);
  $('#node-status-meaning').textContent = NODE_STATUSES[status].meaning;
  applyNodeStatus(activeTab, selectedElement);
});

async function openElementComposition(element = selectedElement) {
  try {
    const originPath = activeTab.path;
    if (!activeTab.adapter.elementName(element)) {
      const name = await requestProcessName(element);
      if (!name) return;
    }
    const qualifiedName = activeTab.adapter.kind === 'cmmn'
      ? resolveCmmnElementName(activeTab.adapter.elementName(element), { packageName: activeTab.adapter.diagramName(activeTab.path), reusable: true })
      : (compositionPathFor(element, activeTab.path), qualifiedSymbolFor(element, { diagramPath: activeTab.path }));
    const result = await api('/api/compositions', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ kind: 'bpmn', qualified_name: qualifiedName }) });
    await openDiagram(result.diagram, activeTab.adapter.kind === 'cmmn' ? originPath : null);
  } catch (error) { showToast(error.message); }
}

$('#open-composition').addEventListener('click', () => openElementComposition());

let pendingProcessName = null;
function closeProcessNameDialog(value = null) {
  $('#process-name-modal').classList.add('hidden');
  const pending = pendingProcessName;
  pendingProcessName = null;
  pending?.resolve(value);
}

function requestProcessName(element) {
  if (!element?.businessObject || !activeTab?.adapter?.isComposable(element)) return Promise.resolve(null);
  if (pendingProcessName) closeProcessNameDialog();
  $('#process-name-modal .eyebrow').textContent = activeTab.adapter.kind === 'cmmn' ? 'Need-to-design link' : 'Reusable subprocess';
  $('#process-name-title').textContent = 'Name this process';
  $('#process-name-modal p').textContent = 'Enter a short process Name to inherit this diagram package, or enter a fully qualified Name to target another package.';
  $('#process-name-form button[type="submit"]').textContent = 'Open process';
  $('#process-name-input').value = '';
  $('#process-name-input').placeholder = 'Process';
  $('#process-name-error').textContent = '';
  $('#process-name-error').classList.add('hidden');
  $('#process-name-modal').classList.remove('hidden');
  globalThis.requestAnimationFrame(() => $('#process-name-input').focus());
  return new Promise((resolve) => { pendingProcessName = { element, adapter: activeTab.adapter, diagramPath: activeTab.path, resolve }; });
}

function requestCmmnPackageName() {
  if (pendingProcessName) closeProcessNameDialog();
  $('#process-name-modal .eyebrow').textContent = 'Business anchor';
  $('#process-name-title').textContent = 'Name this CMMN package';
  $('#process-name-modal p').textContent = 'Enter a dot-separated package Name. The CMMN business anchor is stored as main.cmmn in that package folder.';
  $('#process-name-form button[type="submit"]').textContent = 'Open CMMN anchor';
  $('#process-name-input').value = '';
  $('#process-name-input').placeholder = 'package.subpackage';
  $('#process-name-error').textContent = '';
  $('#process-name-error').classList.add('hidden');
  $('#process-name-modal').classList.remove('hidden');
  globalThis.requestAnimationFrame(() => $('#process-name-input').focus());
  return new Promise((resolve) => { pendingProcessName = { mode: 'cmmn-package', resolve }; });
}

$('#process-name-form').addEventListener('submit', (event) => {
  event.preventDefault();
  if (!pendingProcessName) return;
  try {
    const name = $('#process-name-input').value.trim();
    if (pendingProcessName.mode === 'cmmn-package') validatePackageName(name);
    else if (pendingProcessName.adapter?.kind === 'cmmn') resolveCmmnElementName(name, { packageName: pendingProcessName.adapter.diagramName(pendingProcessName.diagramPath), reusable: true });
    else resolveBpmnElementName(name, { diagramPath: pendingProcessName.diagramPath, reusable: true });
    pendingProcessName.adapter?.updateName(pendingProcessName.element, name);
    closeProcessNameDialog(name);
  } catch (error) {
    $('#process-name-error').textContent = error.message;
    $('#process-name-error').classList.remove('hidden');
  }
});
$('#process-name-cancel').addEventListener('click', () => closeProcessNameDialog());
$('#process-name-modal').addEventListener('click', (event) => { if (event.target === $('#process-name-modal')) closeProcessNameDialog(); });
document.addEventListener('keydown', (event) => { if (event.key === 'Escape' && !$('#process-name-modal').classList.contains('hidden')) closeProcessNameDialog(); });

$('#new-cmmn').addEventListener('click', async () => {
  try {
    const packageName = await requestCmmnPackageName();
    if (!packageName) return;
    const result = await api('/api/compositions', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ kind: 'cmmn', package_name: packageName }) });
    await openDiagram(result.diagram);
  } catch (error) { showToast(error.message); }
});

$('#return-to-anchor').addEventListener('click', () => {
  const origin = activeTab?.originPath ? tabs.get(activeTab.originPath) : null;
  if (origin) activateTab(origin);
});

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
    assistantProvider = metadata?.assistant_provider || null;
  } catch {}
  assistantCapabilities = await api('/api/assistant/capabilities');
  if (assistantCapabilities.version !== ASSISTANT_SCHEMA_VERSION) throw new Error('Diagram assistant capability version is incompatible');
  const localOperations = Object.keys(DIAGRAM_OPERATION_REGISTRY.operations).sort();
  const remoteOperations = Object.keys(assistantCapabilities.operations || {}).sort();
  if (JSON.stringify(localOperations) !== JSON.stringify(remoteOperations)) throw new Error('Diagram assistant operation registry does not match the editor');
  projectAnchorPath = selectProjectAnchor(await api('/api/diagrams'));
  await openDiagram(projectAnchorPath);
  const inbox = await api('/api/assistant/inbox');
  if (inbox.length) await reviewSubmittedProposal(inbox[0]);
  if (daemon) {
    const proposals = await api(runtimePath('/api/runtime/proposals'));
    const pending = proposals.find((proposal) => !['published', 'rejected', 'cancelled', 'conflicted', 'failed', 'rolledBack'].includes(proposal.state));
    if (pending) await reviewRuntimeProposal(pending);
    await pollRuntime();
  }
}

syncMarkdownControl();
initialize().catch((error) => showToast(error.message));
