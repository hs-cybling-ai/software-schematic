import { architecturalName, collisionKey, compositionFolderForQualifiedName, diagramKind, memberNameFromElementName, owningProcessName, packageNameForCmmnPath, validateCmmnElementName, validateElementName, validateQualifiedProcessName } from './core.js';

export const ASSISTANT_SCHEMA_VERSION = '2.0';
export const MAX_CONTEXT_BYTES = 256 * 1024;
export const MAX_OPERATIONS = 64;
const ID = /^[A-Za-z][A-Za-z0-9_-]*$/;
const TYPES = new Set(['bpmn:Task', 'bpmn:UserTask', 'bpmn:ServiceTask', 'bpmn:ManualTask', 'bpmn:SubProcess', 'bpmn:CallActivity', 'bpmn:StartEvent', 'bpmn:EndEvent', 'bpmn:ExclusiveGateway', 'bpmn:ParallelGateway']);
const CMMN_TYPES = new Set(['cmmn:Task', 'cmmn:HumanTask', 'cmmn:ProcessTask', 'cmmn:CaseTask', 'cmmn:Stage', 'cmmn:Milestone', 'cmmn:EventListener']);
const OPERATIONS = new Set(['replace_node_type', 'update_node_label', 'update_node_name', 'set_process_reference', 'create_process', 'open_process', 'rename_process', 'add_flow_node', 'connect_sequence_flow', 'add_plan_item', 'connect_cmmn', 'replace_diagram_markdown', 'replace_node_markdown']);

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
  const flows = elements.filter((item) => item.businessObject?.$type === 'bpmn:SequenceFlow').map((item) => ({
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

export async function buildContextSnapshot({ scope, tab, primaryNode, diagramMarkdown, nodeMarkdown = '', flush = async () => {} }) {
  await flush();
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
  const nodes = new Map([...snapshot.graph.nodes, ...snapshot.graph.flows].map((element) => [element.id, element]));
  const ids = new Set(nodes.keys());
  const processNames = new Map();
  for (const operation of proposal.operations) {
    if (!OPERATIONS.has(operation.type)) throw new Error(`Unsupported operation: ${operation.type}`);
    if (Object.hasOwn(operation, 'xml') || Object.hasOwn(operation, 'rawXml')) throw new Error('Providers cannot supply raw diagram XML');
    if (operation.diagramPath) normalizeAssistantPath(operation.diagramPath);
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
    if (operation.type === 'rename_process' && operation.oldQualifiedName !== snapshot.processName) throw new Error('Process rename does not match the active scoped identity');
    if (operation.type === 'replace_node_type' && !TYPES.has(operation.bpmnType)) throw new Error(`Unsupported BPMN replacement: ${operation.bpmnType}`);
    if (operation.type === 'connect_sequence_flow') {
      if (!ID.test(operation.flowId) || ids.has(operation.flowId)) throw new Error(`Duplicate or invalid flow ID: ${operation.flowId}`);
      if (!ids.has(operation.sourceId) || !ids.has(operation.targetId)) throw new Error('Sequence flow references an unknown node');
      ids.add(operation.flowId);
      nodes.set(operation.flowId, { id: operation.flowId, type: 'bpmn:SequenceFlow', name: null, label: '', status: 'open' });
    }
    if (operation.type === 'connect_cmmn') {
      if (snapshot.diagramKind !== 'cmmn') throw new Error('CMMN connections may only be added to a CMMN diagram');
      if (!ID.test(operation.connectionId) || ids.has(operation.connectionId)) throw new Error(`Duplicate or invalid connection ID: ${operation.connectionId}`);
      if (!ids.has(operation.sourceId) || !ids.has(operation.targetId)) throw new Error('CMMN connection references an unknown node');
      ids.add(operation.connectionId);
      nodes.set(operation.connectionId, { id: operation.connectionId, type: 'cmmn:Association', name: null, label: '', status: 'open' });
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
    case 'update_node_name': return `Change Name of ${operation.nodeId} to ${operation.name}`;
    case 'set_process_reference': return `Link ${operation.nodeId} to ${operation.qualifiedName}`;
    case 'create_process': return `Create process ${operation.qualifiedName} at schematics/${compositionFolderForQualifiedName(operation.qualifiedName)}`;
    case 'open_process': return `Open process ${operation.qualifiedName}`;
    case 'rename_process': return `Rename ${operation.oldQualifiedName} to ${operation.newQualifiedName}; move schematics/${compositionFolderForQualifiedName(operation.oldQualifiedName)} to schematics/${compositionFolderForQualifiedName(operation.newQualifiedName)}; keep relative documentation links unchanged`;
    case 'add_flow_node': return `Add ${operation.bpmnType.replace('bpmn:', '')} ${operation.nodeId}${operation.label ? ` (“${operation.label}”)` : ''}`;
    case 'connect_sequence_flow': return `Connect ${operation.sourceId} → ${operation.targetId}`;
    case 'add_plan_item': return `Add ${operation.cmmnType.replace('cmmn:', '')} ${operation.nodeId}${operation.label ? ` (“${operation.label}”)` : ''}`;
    case 'connect_cmmn': return `Connect CMMN elements ${operation.sourceId} → ${operation.targetId}`;
    case 'replace_diagram_markdown': return 'Replace diagram Markdown';
    case 'replace_node_markdown': return `Replace Markdown for ${operation.nodeId}`;
    default: return operation.type;
  }
}

export function isAssistantEligible(element) {
  const type = element?.businessObject?.definitionRef?.$type || element?.businessObject?.$type;
  return Boolean(type && !element.waypoints && !element.labelTarget && (TYPES.has(type) || CMMN_TYPES.has(type)));
}
