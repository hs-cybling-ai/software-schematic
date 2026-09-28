import { architecturalName, collisionKey, compositionFolderForQualifiedName, diagramKind, memberNameFromElementName, owningProcessName, packageNameForCmmnPath, validateCmmnElementName, validateElementName, validateQualifiedProcessName } from './core.js';

export const ASSISTANT_SCHEMA_VERSION = '2.0';
export const MAX_CONTEXT_BYTES = 256 * 1024;
export const MAX_OPERATIONS = 64;

export function assistantComposerIntent(event, value, phase = 'interview') {
  if (event?.key !== 'Enter') return 'none';
  if (event.shiftKey) return 'newline';
  if (event.isComposing || event.keyCode === 229) return 'composing';
  if (phase !== 'interview') return 'none';
  return String(value || '').trim() ? 'send' : 'empty';
}

const ID = /^[A-Za-z][A-Za-z0-9_-]*$/;
const TYPES = new Set(['bpmn:Task', 'bpmn:UserTask', 'bpmn:ServiceTask', 'bpmn:ManualTask', 'bpmn:SubProcess', 'bpmn:CallActivity', 'bpmn:StartEvent', 'bpmn:EndEvent', 'bpmn:ExclusiveGateway', 'bpmn:ParallelGateway']);
const CMMN_TYPES = new Set(['cmmn:Task', 'cmmn:HumanTask', 'cmmn:ProcessTask', 'cmmn:CaseTask', 'cmmn:Stage', 'cmmn:Milestone', 'cmmn:EventListener']);
export const DIAGRAM_OPERATION_REGISTRY = Object.freeze({
  version: ASSISTANT_SCHEMA_VERSION,
  operations: Object.freeze({
    replace_node_type: { diagrams: ['bpmn'], preview: true, apply: true, undo: true, rollback: true },
    update_node_label: { diagrams: ['bpmn', 'cmmn'], preview: true, apply: true, undo: true, rollback: true },
    update_node_name: { diagrams: ['bpmn', 'cmmn'], preview: true, apply: true, undo: true, rollback: true },
    set_node_status: { diagrams: ['bpmn', 'cmmn'], preview: true, apply: true, undo: true, rollback: true },
    set_process_reference: { diagrams: ['bpmn', 'cmmn'], preview: true, apply: true, undo: true, rollback: true },
    create_process: { diagrams: ['bpmn'], preview: true, apply: true, undo: true, rollback: true },
    open_process: { diagrams: ['bpmn'], preview: true, apply: true, undo: true, rollback: true },
    rename_process: { diagrams: ['bpmn'], preview: true, apply: true, undo: true, rollback: true },
    add_flow_node: { diagrams: ['bpmn'], preview: true, apply: true, undo: true, rollback: true },
    connect_sequence_flow: { diagrams: ['bpmn'], preview: true, apply: true, undo: true, rollback: true },
    add_participant: { diagrams: ['bpmn'], preview: true, apply: true, undo: true, rollback: true },
    connect_message_flow: { diagrams: ['bpmn'], preview: true, apply: true, undo: true, rollback: true },
    move_element: { diagrams: ['bpmn', 'cmmn'], preview: true, apply: true, undo: true, rollback: true },
    remove_element: { diagrams: ['bpmn', 'cmmn'], preview: true, apply: true, undo: true, rollback: true },
    disconnect_flow: { diagrams: ['bpmn', 'cmmn'], preview: true, apply: true, undo: true, rollback: true },
    add_plan_item: { diagrams: ['cmmn'], preview: true, apply: true, undo: true, rollback: true },
    connect_cmmn: { diagrams: ['cmmn'], preview: true, apply: true, undo: true, rollback: true },
    replace_diagram_markdown: { diagrams: ['bpmn', 'cmmn'], preview: true, apply: true, undo: true, rollback: true },
    replace_node_markdown: { diagrams: ['bpmn', 'cmmn'], preview: true, apply: true, undo: true, rollback: true },
    replace_edge_markdown: { diagrams: ['bpmn', 'cmmn'], preview: true, apply: true, undo: true, rollback: true },
  }),
});
const OPERATIONS = new Set(Object.keys(DIAGRAM_OPERATION_REGISTRY.operations));

export function createProposalBeforeState(sourceRevision) {
  return {
    diagrams: new Map(),
    documents: new Map(),
    createdCompositions: [],
    openedTabs: [],
    processRenames: [],
    schematicRevision: null,
    revision: sourceRevision,
  };
}

export function approveProposal(proposal, snapshot, currentRevision) {
  return Object.freeze({
    version: ASSISTANT_SCHEMA_VERSION,
    sourceRevision: snapshot.sourceRevision,
    proposal: validateProposal(proposal, snapshot, { currentRevision }),
  });
}

export function assertRevertRevision(expectedRevision, currentRevision) {
  if (expectedRevision && currentRevision !== expectedRevision) {
    throw new Error('The schematic changed after this assistant proposal. Review the newer work before reverting.');
  }
}

export function executeModelerOperation(operation, tab, { renderStatus = () => {} } = {}) {
  const registry = tab.modeler.get('elementRegistry');
  const modeling = tab.modeler.get('modeling');
  if (operation.type === 'replace_node_type') tab.modeler.get('bpmnReplace').replaceElement(registry.get(operation.nodeId), { type: operation.bpmnType });
  else if (operation.type === 'update_node_label') tab.adapter.updateLabel(registry.get(operation.nodeId), operation.label);
  else if (operation.type === 'update_node_name') tab.adapter.updateName(registry.get(operation.nodeId), operation.name);
  else if (operation.type === 'set_node_status') {
    const element = registry.get(operation.nodeId);
    tab.adapter.updateStatus(element, operation.status);
    if (operation.status === 'open') tab.nodeStatuses.delete(operation.nodeId);
    else tab.nodeStatuses.set(operation.nodeId, operation.status);
    renderStatus(tab, element);
  }
  else if (operation.type === 'set_process_reference') tab.adapter.updateName(registry.get(operation.nodeId), operation.qualifiedName);
  else if (operation.type === 'add_flow_node') {
    const elementFactory = tab.modeler.get('elementFactory'); const root = tab.modeler.get('canvas').getRootElement();
    const shape = elementFactory.createShape({ type: operation.bpmnType, id: operation.nodeId });
    const created = modeling.createShape(shape, { x: operation.x || 180, y: operation.y || 160 }, root);
    modeling.updateProperties(created, { name: operation.label || undefined, architecturalName: operation.name || undefined });
  } else if (operation.type === 'connect_sequence_flow') modeling.connect(registry.get(operation.sourceId), registry.get(operation.targetId), { type: 'bpmn:SequenceFlow', id: operation.flowId });
  else if (operation.type === 'add_participant') {
    const elementFactory = tab.modeler.get('elementFactory'); const root = tab.modeler.get('canvas').getRootElement();
    const participant = elementFactory.createParticipantShape({ type: 'bpmn:Participant', id: operation.participantId, isExpanded: false });
    const created = modeling.createShape(participant, { x: operation.x || 500, y: operation.y || 420 }, root);
    modeling.updateProperties(created, { name: operation.label });
  } else if (operation.type === 'connect_message_flow') {
    const connection = modeling.connect(registry.get(operation.sourceId), registry.get(operation.targetId), { type: 'bpmn:MessageFlow', id: operation.flowId });
    modeling.updateProperties(connection, { name: operation.label });
  }
  else if (operation.type === 'move_element') {
    const element = registry.get(operation.elementId);
    modeling.moveElements([element], { x: operation.x - element.x, y: operation.y - element.y });
  } else if (operation.type === 'remove_element') modeling.removeElements([registry.get(operation.elementId)]);
  else if (operation.type === 'disconnect_flow') modeling.removeConnection(registry.get(operation.flowId));
  else if (operation.type === 'add_plan_item') {
    const elementFactory = tab.modeler.get('elementFactory'); const root = tab.modeler.get('canvas').getRootElement();
    const shape = elementFactory.createPlanItemShape(operation.cmmnType);
    const created = modeling.createShape(shape, { x: operation.x || 220, y: operation.y || 180 }, root);
    modeling.updateProperties(created, { id: operation.nodeId, name: operation.label || undefined, architecturalName: operation.name || undefined });
  } else if (operation.type === 'connect_cmmn') {
    const connection = modeling.connect(registry.get(operation.sourceId), registry.get(operation.targetId), { type: 'cmmn:Association' });
    if (connection && operation.connectionId) modeling.updateProperties(connection, { id: operation.connectionId });
  } else return false;
  return true;
}

export const INTERVIEW_TOPICS = Object.freeze([
  'actors', 'triggers', 'outcomes', 'happyPath', 'alternatives', 'failures', 'contracts',
  'boundaries', 'dependencies', 'nonFunctionalConstraints', 'acceptanceEvidence', 'status',
]);

export const ENDPOINT_TOPICS = Object.freeze([
  'methodOrOperation', 'routeOrTopic', 'caller', 'receiver', 'payloadPurpose', 'trustBoundary',
  'successOutcome', 'timeoutRetry', 'failurePath',
]);

const INTERVIEW_QUESTIONS = Object.freeze({
  actors: 'Who performs or participates in this flow?',
  triggers: 'What starts this flow?',
  outcomes: 'What successful business outcome completes the flow?',
  happyPath: 'What are the essential phases of the successful path?',
  alternatives: 'Which alternate paths or decisions must the diagram show?',
  failures: 'What failure outcomes and recovery behavior must the diagram show?',
  contracts: 'Which data, messages, or interface contracts govern the flow?',
  boundaries: 'Which systems or trust boundaries does the flow cross?',
  dependencies: 'Which upstream or downstream capabilities does this flow depend on?',
  nonFunctionalConstraints: 'Which performance, security, reliability, or compliance constraints are material?',
  acceptanceEvidence: 'What evidence will show that this flow works as intended?',
  status: 'What implementation status should the affected activities carry?',
});

const ENDPOINT_QUESTIONS = Object.freeze({
  methodOrOperation: 'What method, operation, command, or event is used?',
  routeOrTopic: 'What route, topic, queue, or logical endpoint is called?',
  caller: 'Which activity or system initiates the interaction?',
  receiver: 'Which service or system receives it?',
  payloadPurpose: 'What data is transferred, and what is its purpose?',
  trustBoundary: 'What authentication, authorization, or trust boundary applies?',
  successOutcome: 'What response or acknowledgement represents success?',
  timeoutRetry: 'What timeout, retry, and delivery behavior applies?',
  failurePath: 'How is failure represented and handled in the process?',
});

export function createInterviewState(snapshot) {
  return {
    version: ASSISTANT_SCHEMA_VERSION,
    diagramPath: normalizeAssistantPath(snapshot.diagramPath),
    sourceRevision: snapshot.sourceRevision,
    round: 0,
    decisions: {},
    crossesSystemBoundary: null,
    serviceEndpoints: [],
    unresolved: [],
  };
}

function normalizeDecision(value) {
  if (value && typeof value === 'object' && value.notApplicable === true) return { disposition: 'notApplicable', value: value.reason || '' };
  const normalized = Array.isArray(value) ? value.map(String).map((item) => item.trim()).filter(Boolean) : String(value ?? '').trim();
  if (!normalized || (Array.isArray(normalized) && normalized.length === 0)) return null;
  return { disposition: 'answered', value: normalized };
}

function normalizeEndpoint(endpoint, index) {
  const id = String(endpoint.id || `endpoint-${index + 1}`).trim();
  if (!id) throw new Error('Service endpoint IDs cannot be empty');
  const normalized = { id, label: String(endpoint.label || endpoint.methodOrOperation || id).trim() };
  for (const topic of ENDPOINT_TOPICS) normalized[topic] = String(endpoint[topic] || '').trim();
  return normalized;
}

export function recordInterviewAnswers(state, answers) {
  if (state?.version !== ASSISTANT_SCHEMA_VERSION) throw new Error('Unsupported design interview version');
  const next = structuredClone(state);
  next.round += 1;
  for (const topic of INTERVIEW_TOPICS) {
    if (!Object.hasOwn(answers, topic)) continue;
    const decision = normalizeDecision(answers[topic]);
    if (decision) next.decisions[topic] = decision;
    else delete next.decisions[topic];
  }
  if (Object.hasOwn(answers, 'crossesSystemBoundary')) next.crossesSystemBoundary = Boolean(answers.crossesSystemBoundary);
  if (Array.isArray(answers.serviceEndpoints)) next.serviceEndpoints = answers.serviceEndpoints.map(normalizeEndpoint);
  if (Array.isArray(answers.unresolved)) next.unresolved = answers.unresolved.map(String).map((item) => item.trim()).filter(Boolean);
  return next;
}

export function assessInterviewCoverage(state) {
  if (state?.version !== ASSISTANT_SCHEMA_VERSION) throw new Error('Unsupported design interview version');
  const missing = INTERVIEW_TOPICS.filter((topic) => !state.decisions[topic]);
  const endpointGaps = [];
  if (state.crossesSystemBoundary === true && state.serviceEndpoints.length === 0) {
    endpointGaps.push({ endpointId: null, missing: ['serviceEndpoints'], question: 'Which service endpoints or message channels cross the system boundary?' });
  }
  for (const endpoint of state.serviceEndpoints) {
    const endpointMissing = ENDPOINT_TOPICS.filter((topic) => !endpoint[topic]);
    if (endpointMissing.length) endpointGaps.push({
      endpointId: endpoint.id,
      missing: endpointMissing,
      question: `${endpoint.label || endpoint.id}: ${ENDPOINT_QUESTIONS[endpointMissing[0]]}`,
    });
  }
  const questions = missing.map((topic) => INTERVIEW_QUESTIONS[topic]);
  questions.push(...endpointGaps.map((gap) => gap.question));
  questions.push(...state.unresolved.map((decision) => `Resolve or explicitly defer: ${decision}`));
  return {
    ready: questions.length === 0,
    missing,
    endpointGaps,
    unresolved: [...state.unresolved],
    questions,
    nextQuestion: questions[0] || null,
  };
}

export function buildEndpointAnnotations(endpoint) {
  const normalized = normalizeEndpoint(endpoint, 0);
  const missing = ENDPOINT_TOPICS.filter((topic) => !normalized[topic]);
  if (missing.length) throw new Error(`${normalized.label} is missing endpoint contract fields: ${missing.join(', ')}`);
  if (!isConciseLabel(normalized.label)) throw new Error('Endpoint Label must remain concise');
  const activityMarkdown = [
    `# ${normalized.label}`,
    '',
    `- Operation: ${normalized.methodOrOperation} ${normalized.routeOrTopic}`,
    `- Caller: ${normalized.caller}`,
    `- Receiver: ${normalized.receiver}`,
    `- Success: ${normalized.successOutcome}`,
    `- Timeout/retry: ${normalized.timeoutRetry}`,
    `- Failure path: ${normalized.failurePath}`,
  ].join('\n');
  const messageMarkdown = [
    `# ${normalized.label}`,
    '',
    `- Producer: ${normalized.caller}`,
    `- Consumer: ${normalized.receiver}`,
    `- Endpoint/topic: ${normalized.methodOrOperation} ${normalized.routeOrTopic}`,
    `- Payload purpose: ${normalized.payloadPurpose}`,
    `- Trust boundary: ${normalized.trustBoundary}`,
    `- Success acknowledgement: ${normalized.successOutcome}`,
    `- Timeout/retry: ${normalized.timeoutRetry}`,
    `- Failure semantics: ${normalized.failurePath}`,
  ].join('\n');
  return { label: normalized.label, activityMarkdown, messageMarkdown };
}

export function normalizeAssistantPath(value) {
  const path = String(value || '').trim().replaceAll('\\', '/').replace(/^schematics\//, '');
  if (!path || path.startsWith('/') || path.split('/').some((part) => !part || part === '.' || part === '..')) throw new Error(`Path escapes schematics: ${value}`);
  return path;
}

export function stableRevision(value) {
  const input = typeof value === 'string' ? value : JSON.stringify(value);
  let first = 0x811c9dc5;
  let second = 0x9e3779b9;
  for (let index = 0; index < input.length; index += 1) {
    first = Math.imul(first ^ input.charCodeAt(index), 0x01000193) >>> 0;
    second = Math.imul(second ^ input.charCodeAt(index), 0x85ebca6b) >>> 0;
  }
  return `${first.toString(16).padStart(8, '0')}${second.toString(16).padStart(8, '0')}`;
}

export function semanticGraph(modeler, statuses = new Map(), { adapter = null } = {}) {
  if (adapter?.normalizedElements) return adapter.normalizedElements(statuses);
  const elements = modeler.get('elementRegistry').getAll();
  const nodes = elements.filter((item) => item.businessObject?.$type && !item.waypoints && !item.labelTarget && !['bpmn:Process', 'bpmn:Collaboration', 'bpmn:Definitions'].includes(item.businessObject.$type)).map((item) => ({
    id: item.id,
    type: item.businessObject.$type,
    name: architecturalName(item.businessObject) || null,
    label: item.businessObject.name || '',
    status: statuses.get(item.id) || 'open',
  })).sort((a, b) => a.id.localeCompare(b.id));
  const flows = elements.filter((item) => item.waypoints && item.businessObject?.$type).map((item) => ({
    id: item.id,
    type: item.businessObject.$type,
    name: architecturalName(item.businessObject) || null,
    label: item.businessObject.name || '',
    status: statuses.get(item.id) || 'open',
    source: item.source?.id || item.businessObject.sourceRef?.id,
    target: item.target?.id || item.businessObject.targetRef?.id,
  })).sort((a, b) => a.id.localeCompare(b.id));
  return { nodes, flows };
}

export async function buildContextSnapshot({ scope, tab, primaryElement = null, primaryNode = primaryElement, diagramMarkdown, nodeMarkdown = '', edgeMarkdown = '', flush = async () => {} }) {
  await flush();
  const target = primaryElement || primaryNode || null;
  const elementScoped = scope === 'node' || scope === 'edge';
  const kind = diagramKind(tab.path);
  const processName = kind === 'bpmn' ? (() => { try { return owningProcessName(tab.path); } catch { return null; } })() : null;
  const packageName = kind === 'cmmn' ? (() => { try { return packageNameForCmmnPath(tab.path); } catch { return null; } })() : null;
  const graph = semanticGraph(tab.modeler, tab.nodeStatuses, { adapter: tab.adapter });
  const snapshot = {
    version: ASSISTANT_SCHEMA_VERSION,
    scope,
    diagramPath: normalizeAssistantPath(tab.path),
    diagramKind: kind,
    processName,
    packageName,
    contextRole: kind === 'cmmn' ? 'business-need' : 'design',
    primaryElementId: elementScoped ? target?.id || null : null,
    primaryElementKind: elementScoped ? scope : null,
    primaryNodeId: scope === 'node' ? target?.id || null : null,
    primaryEdgeId: scope === 'edge' ? target?.id || null : null,
    graph,
    diagramMarkdown: diagramMarkdown || '',
    nodeMarkdown: scope === 'node' ? nodeMarkdown || '' : '',
    edgeMarkdown: scope === 'edge' ? edgeMarkdown || '' : '',
    truncated: [],
  };
  const structuralBytes = new TextEncoder().encode(JSON.stringify({ ...snapshot, diagramMarkdown: '', nodeMarkdown: '', edgeMarkdown: '' })).length;
  if (structuralBytes > MAX_CONTEXT_BYTES) throw new Error('Diagram structure exceeds the assistant context limit');
  for (const field of ['diagramMarkdown', 'nodeMarkdown', 'edgeMarkdown']) {
    while (new TextEncoder().encode(JSON.stringify(snapshot)).length > MAX_CONTEXT_BYTES && snapshot[field]) {
      snapshot[field] = snapshot[field].slice(0, Math.floor(snapshot[field].length * 0.8));
      if (!snapshot.truncated.includes(field)) snapshot.truncated.push(field);
    }
  }
  if (new TextEncoder().encode(JSON.stringify(snapshot)).length > MAX_CONTEXT_BYTES) throw new Error('Required assistant context exceeds the configured limit');
  snapshot.sourceRevision = stableRevision(snapshot);
  return snapshot;
}

export function validateProposal(proposal, snapshot, { currentRevision = snapshot.sourceRevision } = {}) {
  if (!proposal || proposal.version !== ASSISTANT_SCHEMA_VERSION) throw new Error('Unsupported assistant proposal version');
  if (proposal.requestId !== snapshot.requestId) throw new Error('Proposal request correlation does not match');
  if (proposal.sourceRevision !== snapshot.sourceRevision || currentRevision !== snapshot.sourceRevision) throw new Error('Proposal is stale; regenerate it from the current diagram');
  if (!Array.isArray(proposal.operations) || proposal.operations.length > MAX_OPERATIONS) throw new Error(`Proposal exceeds the ${MAX_OPERATIONS}-operation limit`);
  const nodes = new Map([...snapshot.graph.nodes, ...snapshot.graph.flows].map((element) => [element.id, element]));
  const ids = new Set(nodes.keys());
  const processNames = new Map();
  const createdMessageFlows = new Set();
  const documentedEdges = new Set();
  const createdDocumentedNodes = new Set();
  const documentedNodes = new Set();
  const activeDiagramPath = normalizeAssistantPath(snapshot.diagramPath);
  const primaryElementId = snapshot.primaryElementId || snapshot.primaryNodeId || snapshot.primaryEdgeId || null;
  const elementScoped = Boolean(primaryElementId && snapshot.scope !== 'diagram');
  const anchoredProcessNames = new Set(proposal.operations
    .filter((operation) => operation.type === 'set_process_reference' && operation.nodeId === primaryElementId)
    .map((operation) => operation.qualifiedName));
  const existingPrimaryName = nodes.get(primaryElementId)?.name;
  if (existingPrimaryName && !existingPrimaryName.includes('#')) anchoredProcessNames.add(existingPrimaryName);
  const allowedDiagramPaths = new Set([activeDiagramPath]);
  for (const operation of proposal.operations) {
    if (['create_process', 'open_process'].includes(operation.type) && operation.qualifiedName) {
      if (elementScoped && !anchoredProcessNames.has(operation.qualifiedName)) throw new Error('Element-scoped proposals may only create or open a process directly referenced by the primary element; use diagram-level assistance for multi-node changes');
      allowedDiagramPaths.add(`${compositionFolderForQualifiedName(operation.qualifiedName)}/main.bpmn`);
    }
    if (operation.type === 'rename_process' && operation.newQualifiedName) {
      if (elementScoped) throw new Error('Element-scoped proposals cannot rename the parent process; use diagram-level assistance for multi-node changes');
      allowedDiagramPaths.add(`${compositionFolderForQualifiedName(operation.newQualifiedName)}/main.bpmn`);
    }
  }
  for (const operation of proposal.operations) {
    if (!OPERATIONS.has(operation.type)) throw new Error(`Unsupported operation: ${operation.type}`);
    if (elementScoped) {
      const operationDiagram = operation.diagramPath ? normalizeAssistantPath(operation.diagramPath) : activeDiagramPath;
      if (operationDiagram === activeDiagramPath) {
        const targetsPrimary = ['replace_node_type', 'update_node_label', 'update_node_name', 'set_node_status', 'set_process_reference', 'replace_node_markdown'].includes(operation.type)
          ? operation.nodeId === primaryElementId
          : ['move_element', 'remove_element'].includes(operation.type)
            ? operation.elementId === primaryElementId
            : operation.type === 'replace_edge_markdown'
              ? operation.edgeId === primaryElementId
              : ['create_process', 'open_process'].includes(operation.type)
                ? anchoredProcessNames.has(operation.qualifiedName)
                : false;
        if (!targetsPrimary) throw new Error('Element-scoped proposal targets an unrelated parent-diagram element; use diagram-level assistance for multi-node changes');
      }
    }
    if (Object.hasOwn(operation, 'xml') || Object.hasOwn(operation, 'rawXml')) throw new Error('Providers cannot supply raw diagram XML');
    if (operation.diagramPath) {
      const path = normalizeAssistantPath(operation.diagramPath);
      if (!allowedDiagramPaths.has(path)) throw new Error(`Diagram path is outside this proposal scope: ${path}`);
      const kind = path.endsWith('.cmmn') ? 'cmmn' : 'bpmn';
      if (!DIAGRAM_OPERATION_REGISTRY.operations[operation.type].diagrams.includes(kind)) throw new Error(`${operation.type} is not supported for ${kind.toUpperCase()}`);
    } else if (!['create_process', 'open_process', 'rename_process'].includes(operation.type)) {
      const kind = snapshot.diagramKind || diagramKind(snapshot.diagramPath);
      if (!DIAGRAM_OPERATION_REGISTRY.operations[operation.type].diagrams.includes(kind)) throw new Error(`${operation.type} is not supported for ${kind.toUpperCase()}`);
    }
    if (Object.hasOwn(operation, 'path')) throw new Error('Providers cannot supply composition paths; use a qualified process Name');
    if (operation.qualifiedName) validateQualifiedProcessName(operation.qualifiedName);
    if (operation.oldQualifiedName) validateQualifiedProcessName(operation.oldQualifiedName);
    if (operation.newQualifiedName) validateQualifiedProcessName(operation.newQualifiedName);
    const proposedProcessName = operation.qualifiedName || operation.newQualifiedName;
    if (proposedProcessName) {
      const key = collisionKey(proposedProcessName);
      const prior = processNames.get(key);
      if (prior && prior !== proposedProcessName) throw new Error(`Case-insensitive qualified Name collision: ${prior} and ${proposedProcessName}`);
      processNames.set(key, proposedProcessName);
    }
    if (operation.nodeId) {
      if (!ID.test(operation.nodeId)) throw new Error(`Invalid node ID: ${operation.nodeId}`);
      const target = nodes.get(operation.nodeId);
      if (target?.status === 'locked') throw new Error(`Locked node cannot be changed: ${operation.nodeId}`);
      if (!target && !['add_flow_node', 'add_plan_item'].includes(operation.type)) throw new Error(`Unknown node: ${operation.nodeId}`);
    }
    if (operation.type === 'add_flow_node') {
      if (!ID.test(operation.nodeId) || ids.has(operation.nodeId)) throw new Error(`Duplicate or invalid created ID: ${operation.nodeId}`);
      if (!TYPES.has(operation.bpmnType)) throw new Error(`Unsupported BPMN type: ${operation.bpmnType}`);
      ids.add(operation.nodeId);
      if (operation.name) {
        const name = validateElementName(operation.name);
        const reusable = ['bpmn:CallActivity', 'bpmn:SubProcess'].includes(operation.bpmnType);
        if (reusable === Boolean(memberNameFromElementName(name))) throw new Error(reusable ? 'Reusable process Names cannot contain #' : 'Node Names require #');
      }
      nodes.set(operation.nodeId, { id: operation.nodeId, type: operation.bpmnType, name: operation.name || null, label: operation.label || '', status: 'open' });
      createdDocumentedNodes.add(operation.nodeId);
    }
    if (operation.type === 'add_plan_item') {
      if (snapshot.diagramKind !== 'cmmn') throw new Error('CMMN plan items may only be added to a CMMN diagram');
      if (!ID.test(operation.nodeId) || ids.has(operation.nodeId)) throw new Error(`Duplicate or invalid created ID: ${operation.nodeId}`);
      if (!CMMN_TYPES.has(operation.cmmnType)) throw new Error(`Unsupported CMMN type: ${operation.cmmnType}`);
      if (operation.name) {
        if (operation.cmmnType === 'cmmn:ProcessTask') validateQualifiedProcessName(operation.name);
        else validateCmmnElementName(operation.name);
      }
      ids.add(operation.nodeId);
      nodes.set(operation.nodeId, { id: operation.nodeId, type: operation.cmmnType, name: operation.name || null, label: operation.label || '', status: 'open' });
      createdDocumentedNodes.add(operation.nodeId);
    }
    if (operation.type === 'update_node_name') {
      const target = nodes.get(operation.nodeId);
      if (snapshot.diagramKind === 'cmmn') {
        if (target?.type === 'cmmn:ProcessTask') validateQualifiedProcessName(operation.name);
        else validateCmmnElementName(operation.name);
      } else {
        const name = validateElementName(operation.name);
        const reusable = ['bpmn:CallActivity', 'bpmn:SubProcess'].includes(target?.type);
        if (reusable === Boolean(memberNameFromElementName(name))) throw new Error(reusable ? 'Reusable process Names cannot contain #' : 'Node Names require #');
      }
    }
    if (operation.type === 'set_node_status' && !['open', 'new', 'modify', 'locked'].includes(operation.status)) throw new Error(`Unsupported node status: ${operation.status}`);
    if (operation.type === 'rename_process' && operation.oldQualifiedName !== snapshot.processName) throw new Error('Process rename does not match the active scoped identity');
    if (operation.type === 'replace_node_type' && !TYPES.has(operation.bpmnType)) throw new Error(`Unsupported BPMN replacement: ${operation.bpmnType}`);
    if (operation.type === 'connect_sequence_flow') {
      if (!ID.test(operation.flowId) || ids.has(operation.flowId)) throw new Error(`Duplicate or invalid flow ID: ${operation.flowId}`);
      if (!ids.has(operation.sourceId) || !ids.has(operation.targetId)) throw new Error('Sequence flow references an unknown node');
      ids.add(operation.flowId);
      nodes.set(operation.flowId, { id: operation.flowId, type: 'bpmn:SequenceFlow', name: null, label: '', status: 'open' });
    }
    if (operation.type === 'add_participant') {
      if (snapshot.diagramKind === 'cmmn') throw new Error('BPMN participants may only be added to a BPMN diagram');
      if (!ID.test(operation.participantId) || ids.has(operation.participantId)) throw new Error(`Duplicate or invalid participant ID: ${operation.participantId}`);
      ids.add(operation.participantId);
      nodes.set(operation.participantId, { id: operation.participantId, type: 'bpmn:Participant', name: null, label: operation.label || '', status: 'open' });
    }
    if (operation.type === 'connect_message_flow') {
      if (snapshot.diagramKind === 'cmmn') throw new Error('BPMN message flows may only be added to a BPMN diagram');
      if (!ID.test(operation.flowId) || ids.has(operation.flowId)) throw new Error(`Duplicate or invalid message flow ID: ${operation.flowId}`);
      if (!ids.has(operation.sourceId) || !ids.has(operation.targetId)) throw new Error('Message flow references an unknown activity or participant');
      const source = nodes.get(operation.sourceId); const target = nodes.get(operation.targetId);
      if (source?.type !== 'bpmn:Participant' && target?.type !== 'bpmn:Participant') throw new Error('Message flow must connect an activity to an external participant');
      if (!isConciseLabel(operation.label)) throw new Error('Message flow Label must be a concise function call or data name');
      ids.add(operation.flowId);
      nodes.set(operation.flowId, { id: operation.flowId, type: 'bpmn:MessageFlow', name: null, label: operation.label, status: 'open', source: operation.sourceId, target: operation.targetId });
      createdMessageFlows.add(operation.flowId);
    }
    if (['update_node_label', 'add_flow_node', 'add_plan_item'].includes(operation.type) && operation.label && !isConciseLabel(operation.label)) throw new Error('Activity Label must remain concise; enumerate procedural steps in Markdown');
    if (operation.type === 'move_element' && !ids.has(operation.elementId)) throw new Error(`Unknown element: ${operation.elementId}`);
    if (operation.type === 'remove_element') {
      const target = nodes.get(operation.elementId);
      if (!target) throw new Error(`Unknown element: ${operation.elementId}`);
      if (target.status === 'locked') throw new Error(`Locked element cannot be removed: ${operation.elementId}`);
      ids.delete(operation.elementId); nodes.delete(operation.elementId);
    }
    if (operation.type === 'disconnect_flow') {
      const target = nodes.get(operation.flowId);
      if (!target || !['bpmn:SequenceFlow', 'bpmn:MessageFlow', 'cmmn:Association'].includes(target.type)) throw new Error(`Unknown connection: ${operation.flowId}`);
      if (target.status === 'locked') throw new Error(`Locked connection cannot be removed: ${operation.flowId}`);
      ids.delete(operation.flowId); nodes.delete(operation.flowId);
    }
    if (operation.type === 'replace_edge_markdown') {
      const target = nodes.get(operation.edgeId);
      if (!target || !target.source || !target.target) throw new Error(`Unknown edge: ${operation.edgeId}`);
      if (target.status === 'locked') throw new Error(`Locked edge cannot be documented: ${operation.edgeId}`);
      if (!operation.markdown.trim()) throw new Error(`Edge Markdown cannot be empty: ${operation.edgeId}`);
      documentedEdges.add(operation.edgeId);
    }
    if (operation.type === 'replace_node_markdown') {
      if (!operation.markdown.trim()) throw new Error(`Node Markdown cannot be empty: ${operation.nodeId}`);
      documentedNodes.add(operation.nodeId);
    }
    if (operation.type === 'connect_cmmn') {
      if (snapshot.diagramKind !== 'cmmn') throw new Error('CMMN connections may only be added to a CMMN diagram');
      if (!ID.test(operation.connectionId) || ids.has(operation.connectionId)) throw new Error(`Duplicate or invalid connection ID: ${operation.connectionId}`);
      if (!ids.has(operation.sourceId) || !ids.has(operation.targetId)) throw new Error('CMMN connection references an unknown node');
      ids.add(operation.connectionId);
      nodes.set(operation.connectionId, { id: operation.connectionId, type: 'cmmn:Association', name: null, label: '', status: 'open' });
    }
  }
  for (const flowId of createdMessageFlows) if (!documentedEdges.has(flowId)) throw new Error(`Message flow ${flowId} requires an edge Markdown contract in the same proposal`);
  for (const nodeId of createdDocumentedNodes) if (!documentedNodes.has(nodeId)) throw new Error(`Created activity or event ${nodeId} requires owned Markdown in the same proposal`);
  return proposal;
}

function isConciseLabel(value) {
  const label = String(value || '').trim();
  return Boolean(label) && label.length <= 80 && label.split(/\s+/).length <= 12 && !label.includes('\n');
}

export function proposalGroups(proposal) {
  const groups = new Map();
  for (const operation of proposal.operations) {
    const path = operation.diagramPath || operation.path || 'Current diagram';
    if (!groups.has(path)) groups.set(path, []);
    groups.get(path).push(describeOperation(operation));
  }
  return groups;
}

export function describeOperation(operation) {
  switch (operation.type) {
    case 'replace_node_type': return `Replace ${operation.nodeId} with ${operation.bpmnType.replace('bpmn:', '')}`;
    case 'update_node_label': return `Rename ${operation.nodeId} to “${operation.label}”`;
    case 'update_node_name': return `Change Name of ${operation.nodeId} to ${operation.name}`;
    case 'set_node_status': return `Set ${operation.nodeId} implementation status to ${operation.status}`;
    case 'set_process_reference': return `Link ${operation.nodeId} to ${operation.qualifiedName}`;
    case 'create_process': return `Create process ${operation.qualifiedName} at schematics/${compositionFolderForQualifiedName(operation.qualifiedName)}`;
    case 'open_process': return `Open process ${operation.qualifiedName}`;
    case 'rename_process': return `Rename ${operation.oldQualifiedName} to ${operation.newQualifiedName}; move schematics/${compositionFolderForQualifiedName(operation.oldQualifiedName)} to schematics/${compositionFolderForQualifiedName(operation.newQualifiedName)}; keep relative documentation links unchanged`;
    case 'add_flow_node': return `Add ${operation.bpmnType.replace('bpmn:', '')} ${operation.nodeId}${operation.label ? ` (“${operation.label}”)` : ''}`;
    case 'connect_sequence_flow': return `Connect ${operation.sourceId} → ${operation.targetId}`;
    case 'add_participant': return `Add external participant ${operation.participantId} (“${operation.label}”)`;
    case 'connect_message_flow': return `Connect “${operation.label}” message flow ${operation.sourceId} ⇄ ${operation.targetId}`;
    case 'move_element': return `Move ${operation.elementId} to (${operation.x}, ${operation.y})`;
    case 'remove_element': return `Remove ${operation.elementId}`;
    case 'disconnect_flow': return `Remove connection ${operation.flowId}`;
    case 'add_plan_item': return `Add ${operation.cmmnType.replace('cmmn:', '')} ${operation.nodeId}${operation.label ? ` (“${operation.label}”)` : ''}`;
    case 'connect_cmmn': return `Connect CMMN elements ${operation.sourceId} → ${operation.targetId}`;
    case 'replace_diagram_markdown': return 'Replace diagram Markdown';
    case 'replace_node_markdown': return `Replace Markdown for ${operation.nodeId}`;
    case 'replace_edge_markdown': return `Replace Markdown for edge ${operation.edgeId}`;
    default: throw new Error(`Unsupported operation preview: ${operation.type}`);
  }
}

export function isAssistantEligible(element) {
  const type = element?.businessObject?.definitionRef?.$type || element?.businessObject?.$type;
  return Boolean(type && !element.waypoints && !element.labelTarget && (TYPES.has(type) || CMMN_TYPES.has(type)));
}
