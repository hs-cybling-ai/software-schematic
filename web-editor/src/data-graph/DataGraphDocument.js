import { defaultSubtype, subtypeField, typeDefinition } from './DataGraphTypes.js';

export const DATA_GRAPH_FORMAT = 'diagram-studio/data-graph';
export const DATA_GRAPH_VERSION = 2;
export const NODE_TYPES = new Set(['objectNode', 'edgeNode', 'mutationNode']);
export const EDGE_TYPES = new Set(['domain', 'range', 'modifier', 'objectLink']);

const fail = message => { throw new Error(`Invalid Data Graph document: ${message}`); };
const object = (value, name) => { if (!value || typeof value !== 'object' || Array.isArray(value)) fail(`${name} must be an object`); return value; };
const string = (value, name) => { if (typeof value !== 'string' || !value.trim()) fail(`${name} must be a non-empty string`); return value; };
const number = (value, name) => { if (typeof value !== 'number' || !Number.isFinite(value)) fail(`${name} must be a finite number`); return value; };
const array = (value, name) => { if (!Array.isArray(value)) fail(`${name} must be an array`); return value; };
const point = (raw, name) => { const value = object(raw, name); return { x: number(value.x, `${name}.x`), y: number(value.y, `${name}.y`) }; };
const route = (raw, name) => array(raw ?? [], name).map((value, index) => point(value, `${name}[${index}]`));
const normalizedSubtype = (kind, value, name) => { const key = value ?? defaultSubtype(kind); if (!typeDefinition(kind, key)) fail(`unsupported ${name} ${String(key)}`); return key; };

export function starterDocument() { return { format: DATA_GRAPH_FORMAT, version: DATA_GRAPH_VERSION, nodes: [], edges: [], properties: [], contextStore: { kind: 'sqlite', uri: 'diagram.context.sqlite', schemaVersion: 1, diagramId: 'starter-diagram', revision: 0 } }; }

export function validateDocument(input) {
  const source = object(input, 'root');
  if (source.format !== DATA_GRAPH_FORMAT) fail(`format must be ${DATA_GRAPH_FORMAT}`);
  if (source.version !== DATA_GRAPH_VERSION) fail(`unsupported version ${String(source.version)}`);
  if ('relationships' in source || 'attachments' in source || 'controls' in source) fail('obsolete relationship aggregates are not supported');
  const rawStore = object(source.contextStore, 'contextStore');
  if (rawStore.kind !== 'sqlite') fail('contextStore.kind must be sqlite');
  const uri = string(rawStore.uri, 'contextStore.uri');
  if (uri.includes('/') || uri.includes('\\') || !uri.endsWith('.context.sqlite')) fail('contextStore.uri must be a sibling .context.sqlite filename');
  if (rawStore.schemaVersion !== 1) fail(`unsupported contextStore schema ${String(rawStore.schemaVersion)}`);
  const diagramId = string(rawStore.diagramId, 'contextStore.diagramId');
  if (!Number.isSafeInteger(rawStore.revision) || rawStore.revision < 0) fail('contextStore.revision must be a non-negative integer');
  const contextStore = { kind: 'sqlite', uri, schemaVersion: 1, diagramId, revision: rawStore.revision };
  const ids = new Set(); const claimId = (id, name) => { string(id, name); if (ids.has(id)) fail(`duplicate id ${id}`); ids.add(id); };
  const nodes = array(source.nodes, 'nodes').map((rawNode, index) => {
    const node = object(rawNode, `nodes[${index}]`); claimId(node.id, `nodes[${index}].id`); if (!NODE_TYPES.has(node.type)) fail(`unsupported node type ${String(node.type)}`);
    const result = { id: node.id, type: node.type, label: string(node.label, `${node.id}.label`), x: number(node.x, `${node.id}.x`), y: number(node.y, `${node.id}.y`) };
    if (node.type === 'edgeNode') return { ...result, collectionType: normalizedSubtype('edgeNode', node.collectionType, 'collectionType') };
    if (node.type === 'mutationNode') return { ...result, mutationType: normalizedSubtype('mutationNode', node.mutationType, 'mutationType') };
    return result;
  });
  const nodeById = new Map(nodes.map(node => [node.id, node]));
  const objectLinkKeys = new Set();
  const edges = array(source.edges, 'edges').map((rawEdge, index) => {
    const edge = object(rawEdge, `edges[${index}]`); claimId(edge.id, `edges[${index}].id`); if (!EDGE_TYPES.has(edge.type)) fail(`unsupported edge type ${String(edge.type)}`);
    string(edge.source, `${edge.id}.source`); string(edge.target, `${edge.id}.target`); const sourceNode = nodeById.get(edge.source); const targetNode = nodeById.get(edge.target); if (!sourceNode || !targetNode) fail(`${edge.id} has a dangling endpoint`);
    if (edge.type === 'domain' && (sourceNode.type !== 'objectNode' || targetNode.type !== 'edgeNode')) fail(`${edge.id} domain must connect objectNode to edgeNode`);
    if (edge.type === 'range' && (sourceNode.type !== 'edgeNode' || targetNode.type !== 'objectNode')) fail(`${edge.id} range must connect edgeNode to objectNode`);
    if (edge.type === 'modifier' && (sourceNode.type !== 'mutationNode' || !['objectNode', 'edgeNode'].includes(targetNode.type))) fail(`${edge.id} modifier must connect mutationNode to objectNode or edgeNode`);
    let linkType;
    if (edge.type === 'objectLink') {
      if (sourceNode.type !== 'objectNode' || targetNode.type !== 'objectNode') fail(`${edge.id} objectLink must connect objectNode to objectNode`);
      if (edge.source === edge.target) fail(`${edge.id} objectLink cannot reference its source`);
      linkType = normalizedSubtype('objectLink', edge.linkType, 'linkType');
      const key = `${edge.source}\u0000${edge.target}\u0000${linkType}`;
      if (objectLinkKeys.has(key)) fail(`${edge.id} duplicates an objectLink`);
      objectLinkKeys.add(key);
    }
    return { id: edge.id, type: edge.type, source: edge.source, target: edge.target, waypoints: route(edge.waypoints, `${edge.id}.waypoints`), ...(linkType ? { linkType } : {}) };
  });
  const edgeById = new Map(edges.map(edge => [edge.id, edge]));
  const domains = new Map(); const ranges = new Map();
  edges.filter(edge => ['domain', 'range'].includes(edge.type)).forEach(edge => { const edgeNodeId = edge.type === 'domain' ? edge.target : edge.source; const collection = edge.type === 'domain' ? domains : ranges; collection.set(edgeNodeId, [...(collection.get(edgeNodeId) ?? []), edge]); });
  nodes.filter(node => node.type === 'edgeNode').forEach(node => {
    if ((domains.get(node.id)?.length ?? 0) !== 1) fail(`${node.id} must have exactly one domain edge`);
    const targetIds = (ranges.get(node.id) ?? []).map(edge => edge.target); if (new Set(targetIds).size !== targetIds.length) fail(`${node.id} has duplicate range targets`);
    if (node.collectionType === 'scalar' && targetIds.length > 1) fail(`${node.id} scalar edge node cannot have more than one range`);
  });
  const ownerIds = new Set([...nodeById.keys(), ...edgeById.keys()]);
  const properties = array(source.properties, 'properties').map((rawProperty, index) => {
    const property = object(rawProperty, `properties[${index}]`); claimId(property.id, `properties[${index}].id`); if (property.type !== 'property') fail(`unsupported property type ${String(property.type)}`);
    string(property.ownerId, `${property.id}.ownerId`); if (!ownerIds.has(property.ownerId)) fail(`${property.id} has a dangling owner`); const field = subtypeField(property.type);
    return { id: property.id, type: property.type, ownerId: property.ownerId, label: string(property.label, `${property.id}.label`), x: number(property.x, `${property.id}.x`), y: number(property.y, `${property.id}.y`), [field]: normalizedSubtype(property.type, property[field], field) };
  });
  return { format: DATA_GRAPH_FORMAT, version: DATA_GRAPH_VERSION, nodes, edges, properties, contextStore };
}

const byId = (a, b) => a.id.localeCompare(b.id);
export function serializeDocument(document) { const valid = validateDocument(document); return `${JSON.stringify({ ...valid, nodes: [...valid.nodes].sort(byId), edges: [...valid.edges].sort(byId), properties: [...valid.properties].sort(byId) }, null, 2)}\n`; }
export function parseDocument(text) { if (typeof text !== 'string') fail('source must be text'); let parsed; try { parsed = JSON.parse(text); } catch (error) { fail(`malformed JSON (${error.message})`); } return validateDocument(parsed); }
export function effectiveConnections(document, objectId) {
  const valid = validateDocument(document);
  const object = valid.nodes.find(node => node.id === objectId && node.type === 'objectNode');
  if (!object) fail(`${objectId} must reference an objectNode`);
  const nodesById = new Map(valid.nodes.map(node => [node.id, node]));
  const domainsBySource = new Map();
  const linksBySource = new Map();
  valid.edges.forEach(edge => {
    if (edge.type === 'domain') domainsBySource.set(edge.source, [...(domainsBySource.get(edge.source) ?? []), edge]);
    if (edge.type === 'objectLink') linksBySource.set(edge.source, [...(linksBySource.get(edge.source) ?? []), edge]);
  });
  const visited = new Set(); const relationships = new Map();
  const visit = id => {
    if (visited.has(id)) return;
    visited.add(id);
    [...(domainsBySource.get(id) ?? [])].sort(byId).forEach(edge => relationships.set(edge.target, nodesById.get(edge.target)));
    [...(linksBySource.get(id) ?? [])].sort(byId).forEach(edge => visit(edge.target));
  };
  visit(objectId);
  return [...relationships.values()].filter(Boolean).sort(byId);
}
export function midpointOnPath(points) { if (!Array.isArray(points) || points.length < 2) fail('edge path needs at least two points'); const lengths = points.slice(1).map((value, index) => Math.hypot(value.x - points[index].x, value.y - points[index].y)); const half = lengths.reduce((sum, length) => sum + length, 0) / 2; let travelled = 0; for (let index = 0; index < lengths.length; index += 1) { if (travelled + lengths[index] >= half) { const ratio = lengths[index] === 0 ? 0 : (half - travelled) / lengths[index]; return { x: points[index].x + (points[index + 1].x - points[index].x) * ratio, y: points[index].y + (points[index + 1].y - points[index].y) * ratio }; } travelled += lengths[index]; } return { ...points.at(-1) }; }
