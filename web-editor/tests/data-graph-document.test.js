import { describe, expect, it } from 'vitest';
import { effectiveConnections, parseDocument, serializeDocument, starterDocument, validateDocument } from '../src/data-graph/DataGraphDocument.js';
import { TYPE_REGISTRY } from '../src/data-graph/DataGraphTypes.js';

const graph = () => ({
  format: 'diagram-studio/data-graph', version: 2,
  nodes: [
    { id: 'person', type: 'objectNode', label: 'Person', x: 100, y: 100 },
    { id: 'works_for', type: 'edgeNode', label: 'WORKS_FOR', collectionType: 'set', x: 300, y: 180 },
    { id: 'company', type: 'objectNode', label: 'Company', x: 500, y: 100 },
    { id: 'coworker', type: 'objectNode', label: 'Coworker', x: 500, y: 300 },
    { id: 'add_company', type: 'mutationNode', label: 'add', mutationType: 'function', x: 300, y: 360 }
  ],
  edges: [
    { id: 'works_for_domain', type: 'domain', source: 'person', target: 'works_for', waypoints: [] },
    { id: 'works_for_range_company', type: 'range', source: 'works_for', target: 'company', waypoints: [] },
    { id: 'works_for_range_coworker', type: 'range', source: 'works_for', target: 'coworker', waypoints: [] },
    { id: 'add_company_modifier_works_for', type: 'modifier', source: 'add_company', target: 'works_for', waypoints: [] }
  ],
  properties: [{ id: 'company_urn', type: 'property', ownerId: 'add_company', label: 'companyUrn', dataType: 'uri', x: 420, y: 380 }],
  contextStore: { kind: 'sqlite', uri: 'test.context.sqlite', schemaVersion: 1, diagramId: 'test-diagram', revision: 0 }
});

describe('normalized Data Graph ontology document', () => {
  it('uses scalar as the accessible blank-mark edge-node default', () => { expect(TYPE_REGISTRY.edgeNode.default).toBe('scalar'); expect(TYPE_REGISTRY.edgeNode.entries[0]).toMatchObject({ key: 'scalar', mark: '', maxTargets: 1 }); });
  it('uses directional Unicode function marks', () => { expect(TYPE_REGISTRY.mutationNode.entries.map(({ key, mark }) => ({ key, mark }))).toEqual([{ key: 'query', mark: '←ƒ' }, { key: 'function', mark: 'ƒ→' }, { key: 'transformation', mark: 'ƒ' }]); expect(TYPE_REGISTRY.mutationNode.entries.map(entry => entry.ariaLabel)).toEqual(expect.arrayContaining([expect.stringContaining('returns items'), expect.stringContaining('updates the collection'), expect.stringContaining('internal domain object state')])); });
  it('exposes URI as a first-class property datatype', () => { expect(TYPE_REGISTRY.property.entries).toContainEqual({ key: 'uri', label: 'URI', mark: 'URI', ariaLabel: 'URI datatype' }); expect(validateDocument(graph()).properties[0].dataType).toBe('uri'); });
  it('registers sameAs and subclassOf object-link modifiers', () => { expect(TYPE_REGISTRY.objectLink).toMatchObject({ field: 'linkType', default: 'sameAs' }); expect(TYPE_REGISTRY.objectLink.entries.map(entry => entry.key)).toEqual(['sameAs', 'subclassOf']); });
  it('contains topology and a SQLite context-store manifest', () => expect(Object.keys(validateDocument(graph()))).toEqual(['format', 'version', 'nodes', 'edges', 'properties', 'contextStore']));
  it('validates domain, range, and modifier topology plus mutation parameters', () => { const valid = validateDocument(graph()); expect(valid.edges.map(edge => edge.type)).toEqual(['domain', 'range', 'range', 'modifier']); expect(valid.properties[0].ownerId).toBe('add_company'); expect(valid.nodes.find(node => node.id === 'add_company').mutationType).toBe('function'); });
  it('allows a mutation modifier to target an object node directly', () => { const value = graph(); const modifier = value.edges.find(edge => edge.type === 'modifier'); modifier.id = 'add_company_modifier_person'; modifier.target = 'person'; expect(validateDocument(value).edges.find(edge => edge.type === 'modifier')).toMatchObject({ source: 'add_company', target: 'person' }); });
  it('allows a temporarily targetless scalar edge node', () => { const value = graph(); value.nodes.find(node => node.id === 'works_for').collectionType = undefined; value.edges = value.edges.filter(edge => edge.type === 'domain'); expect(validateDocument(value).nodes.find(node => node.id === 'works_for').collectionType).toBe('scalar'); });
  it('allows properties owned by topology edges', () => { const value = graph(); value.properties[0].ownerId = 'works_for_domain'; expect(validateDocument(value).properties[0].ownerId).toBe('works_for_domain'); });
  it('serializes deterministically without mutating input', () => { const input = graph(); const before = JSON.parse(JSON.stringify(input)); const first = serializeDocument(input); expect(serializeDocument(parseDocument(first))).toBe(first); expect(input).toEqual(before); });
  it('validates an empty starter document', () => expect(validateDocument(starterDocument())).toEqual(starterDocument()));
  it('round trips typed object links deterministically', () => { const value = graph(); value.edges.push({ id: 'coworker_same_as_person', type: 'objectLink', source: 'coworker', target: 'person', linkType: 'sameAs', waypoints: [] }); const serialized = serializeDocument(value); expect(parseDocument(serialized).edges.find(edge => edge.type === 'objectLink')).toMatchObject({ source: 'coworker', target: 'person', linkType: 'sameAs' }); });
  it('resolves inherited connections without materializing them', () => { const value = graph(); value.edges.push({ id: 'coworker_same_as_person', type: 'objectLink', source: 'coworker', target: 'person', linkType: 'sameAs', waypoints: [] }); expect(effectiveConnections(value, 'coworker').map(node => node.id)).toEqual(['works_for']); expect(value.nodes.filter(node => node.type === 'edgeNode')).toHaveLength(1); });
  it('terminates cycles and deduplicates diamond inheritance', () => { const value = graph(); value.nodes.push({ id: 'manager', type: 'objectNode', label: 'Manager', x: 700, y: 300 }); value.edges.push(
    { id: 'coworker_person', type: 'objectLink', source: 'coworker', target: 'person', linkType: 'sameAs', waypoints: [] },
    { id: 'coworker_manager', type: 'objectLink', source: 'coworker', target: 'manager', linkType: 'subclassOf', waypoints: [] },
    { id: 'manager_person', type: 'objectLink', source: 'manager', target: 'person', linkType: 'subclassOf', waypoints: [] },
    { id: 'person_coworker', type: 'objectLink', source: 'person', target: 'coworker', linkType: 'subclassOf', waypoints: [] }
  ); expect(effectiveConnections(value, 'coworker').map(node => node.id)).toEqual(['works_for']); });
  it.each([
    ['obsolete relationships aggregate', value => { value.relationships = []; }],
    ['obsolete attachments aggregate', value => { value.attachments = []; }],
    ['object alias', value => { value.nodes[0].type = 'object'; }],
    ['reversed domain', value => { [value.edges[0].source, value.edges[0].target] = [value.edges[0].target, value.edges[0].source]; }],
    ['reversed range', value => { [value.edges[1].source, value.edges[1].target] = [value.edges[1].target, value.edges[1].source]; }],
    ['reversed modifier', value => { const edge = value.edges.find(item => item.type === 'modifier'); [edge.source, edge.target] = [edge.target, edge.source]; }],
    ['modifier targeting mutation', value => { value.edges.find(item => item.type === 'modifier').target = 'add_company'; }],
    ['object link with non-object endpoint', value => { value.edges.push({ id: 'bad_link', type: 'objectLink', source: 'works_for', target: 'person', linkType: 'sameAs', waypoints: [] }); }],
    ['object link self-reference', value => { value.edges.push({ id: 'bad_link', type: 'objectLink', source: 'person', target: 'person', linkType: 'sameAs', waypoints: [] }); }],
    ['unknown object link modifier', value => { value.edges.push({ id: 'bad_link', type: 'objectLink', source: 'coworker', target: 'person', linkType: 'unknown', waypoints: [] }); }],
    ['duplicate object link', value => { value.edges.push({ id: 'link_one', type: 'objectLink', source: 'coworker', target: 'person', linkType: 'sameAs', waypoints: [] }, { id: 'link_two', type: 'objectLink', source: 'coworker', target: 'person', linkType: 'sameAs', waypoints: [] }); }],
    ['missing domain', value => { value.edges = value.edges.filter(edge => edge.type !== 'domain'); }],
    ['duplicate domain', value => { value.edges.push({ id: 'other_domain', type: 'domain', source: 'company', target: 'works_for', waypoints: [] }); }],
    ['duplicate range target', value => { value.edges.push({ id: 'duplicate_range', type: 'range', source: 'works_for', target: 'company', waypoints: [] }); }],
    ['scalar overflow', value => { value.nodes.find(node => node.id === 'works_for').collectionType = 'scalar'; }],
    ['dangling property owner', value => { value.properties[0].ownerId = 'missing'; }]
  ])('rejects %s', (_name, mutate) => { const value = graph(); mutate(value); expect(() => validateDocument(value)).toThrow('Invalid Data Graph document'); });
});
