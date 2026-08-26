export const ASSISTANT_SCHEMA_VERSION = '1.0';
export const MAX_CONTEXT_BYTES = 256 * 1024;
export const MAX_OPERATIONS = 64;
const ID = /^[A-Za-z][A-Za-z0-9_-]*$/;
const TYPES = new Set(['bpmn:Task', 'bpmn:UserTask', 'bpmn:ServiceTask', 'bpmn:ManualTask', 'bpmn:SubProcess', 'bpmn:CallActivity', 'bpmn:StartEvent', 'bpmn:EndEvent', 'bpmn:ExclusiveGateway', 'bpmn:ParallelGateway']);
const OPERATIONS = new Set(['replace_node_type', 'update_node_label', 'set_composition_link', 'create_composition', 'open_composition', 'add_flow_node', 'connect_sequence_flow', 'replace_diagram_markdown', 'replace_node_markdown']);

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

export function semanticGraph(modeler, statuses = new Map()) {
  const elements = modeler.get('elementRegistry').getAll();
  const nodes = elements.filter((item) => item.businessObject?.$type && !item.waypoints && !item.labelTarget && !['bpmn:Process', 'bpmn:Collaboration', 'bpmn:Definitions'].includes(item.businessObject.$type)).map((item) => ({
    id: item.id,
    type: item.businessObject.$type,
    label: item.businessObject.name || '',
    status: statuses.get(item.id) || 'open',
    composition: item.businessObject.calledElement || null,
  })).sort((a, b) => a.id.localeCompare(b.id));
  const flows = elements.filter((item) => item.businessObject?.$type === 'bpmn:SequenceFlow').map((item) => ({
    id: item.id,
    source: item.source?.id || item.businessObject.sourceRef?.id,
    target: item.target?.id || item.businessObject.targetRef?.id,
    label: item.businessObject.name || '',
  })).sort((a, b) => a.id.localeCompare(b.id));
  return { nodes, flows };
}

export async function buildContextSnapshot({ scope, tab, primaryNode, diagramMarkdown, nodeMarkdown = '', flush = async () => {} }) {
  await flush();
  const graph = semanticGraph(tab.modeler, tab.nodeStatuses);
  const snapshot = {
    version: ASSISTANT_SCHEMA_VERSION,
    scope,
    diagramPath: normalizeAssistantPath(tab.path),
    primaryNodeId: scope === 'node' ? primaryNode?.id || null : null,
    graph,
    diagramMarkdown: diagramMarkdown || '',
    nodeMarkdown: scope === 'node' ? nodeMarkdown || '' : '',
    truncated: [],
  };
  const structuralBytes = new TextEncoder().encode(JSON.stringify({ ...snapshot, diagramMarkdown: '', nodeMarkdown: '' })).length;
  if (structuralBytes > MAX_CONTEXT_BYTES) throw new Error('Diagram structure exceeds the assistant context limit');
  for (const field of ['diagramMarkdown', 'nodeMarkdown']) {
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
  const nodes = new Map(snapshot.graph.nodes.map((node) => [node.id, node]));
  const ids = new Set(nodes.keys());
  for (const operation of proposal.operations) {
    if (!OPERATIONS.has(operation.type)) throw new Error(`Unsupported operation: ${operation.type}`);
    if (operation.diagramPath) normalizeAssistantPath(operation.diagramPath);
    if (operation.path) normalizeAssistantPath(operation.path);
    if (operation.nodeId) {
      if (!ID.test(operation.nodeId)) throw new Error(`Invalid node ID: ${operation.nodeId}`);
      const target = nodes.get(operation.nodeId);
      if (target?.status === 'locked') throw new Error(`Locked node cannot be changed: ${operation.nodeId}`);
      if (!target && operation.type !== 'add_flow_node') throw new Error(`Unknown node: ${operation.nodeId}`);
    }
    if (operation.type === 'add_flow_node') {
      if (!ID.test(operation.nodeId) || ids.has(operation.nodeId)) throw new Error(`Duplicate or invalid created ID: ${operation.nodeId}`);
      if (!TYPES.has(operation.bpmnType)) throw new Error(`Unsupported BPMN type: ${operation.bpmnType}`);
      ids.add(operation.nodeId);
    }
    if (operation.type === 'replace_node_type' && !TYPES.has(operation.bpmnType)) throw new Error(`Unsupported BPMN replacement: ${operation.bpmnType}`);
    if (operation.type === 'connect_sequence_flow') {
      if (!ID.test(operation.flowId) || ids.has(operation.flowId)) throw new Error(`Duplicate or invalid flow ID: ${operation.flowId}`);
      if (!ids.has(operation.sourceId) || !ids.has(operation.targetId)) throw new Error('Sequence flow references an unknown node');
      ids.add(operation.flowId);
    }
  }
  return proposal;
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

function describeOperation(operation) {
  switch (operation.type) {
    case 'replace_node_type': return `Replace ${operation.nodeId} with ${operation.bpmnType.replace('bpmn:', '')}`;
    case 'update_node_label': return `Rename ${operation.nodeId} to “${operation.label}”`;
    case 'set_composition_link': return `Link ${operation.nodeId} to ${operation.path}`;
    case 'create_composition': return `Create composition ${operation.path}`;
    case 'open_composition': return `Open composition ${operation.path}`;
    case 'add_flow_node': return `Add ${operation.bpmnType.replace('bpmn:', '')} ${operation.nodeId}${operation.label ? ` (“${operation.label}”)` : ''}`;
    case 'connect_sequence_flow': return `Connect ${operation.sourceId} → ${operation.targetId}`;
    case 'replace_diagram_markdown': return 'Replace diagram Markdown';
    case 'replace_node_markdown': return `Replace Markdown for ${operation.nodeId}`;
    default: return operation.type;
  }
}

export function isAssistantEligible(element) {
  return Boolean(element?.businessObject?.$type && !element.waypoints && !element.labelTarget && TYPES.has(element.businessObject.$type));
}
