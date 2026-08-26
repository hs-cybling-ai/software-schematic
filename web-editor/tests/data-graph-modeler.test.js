import { describe, expect, it, vi } from 'vitest';
import { DataGraphModeler } from '../src/data-graph/DataGraphModeler.js';

const starter = JSON.stringify({ format: 'diagram-studio/data-graph', version: 2, nodes: [], edges: [], properties: [], contextStore: { kind: 'sqlite', uri: 'test.context.sqlite', schemaVersion: 1, diagramId: 'test-diagram', revision: 0 } });
const make = () => { const container = document.createElement('div'); document.body.append(container); return { container, modeler: new DataGraphModeler({ container }) }; };
const centerOf = element => ({ x: element.x + element.width / 2, y: element.y + element.height / 2 });

describe('DataGraphModeler normalized ontology graph', () => {
  it('creates objectNode, edgeNode, domain, and range records with suffix IDs', async () => {
    const { modeler } = make(); await modeler.importJSON(starter); const source = modeler.addNode('objectNode'); const target = modeler.addNode('objectNode'); const relationship = modeler.connect(source.id, target.id); const saved = JSON.parse((await modeler.saveJSON()).json);
    expect(saved.nodes.find(node => node.id === source.id).type).toBe('objectNode'); expect(saved.nodes.find(node => node.id === relationship.control.id)).toMatchObject({ type: 'edgeNode', collectionType: 'scalar' });
    expect(saved.edges).toEqual(expect.arrayContaining([expect.objectContaining({ id: `${relationship.id}_domain`, type: 'domain', source: source.id, target: relationship.id }), expect.objectContaining({ id: `${relationship.id}_range_${target.id}`, type: 'range', source: relationship.id, target: target.id })])); modeler.destroy();
  });

  it('creates a temporarily targetless scalar edge node with a blank center', async () => {
    const { container, modeler } = make(); await modeler.importJSON(starter); const source = modeler.addNode('objectNode'); const relationship = modeler.addRelationship(source.id, { x: 420, y: 260 }); const saved = JSON.parse((await modeler.saveJSON()).json);
    expect(centerOf(relationship.control)).toEqual({ x: 420, y: 260 }); expect(relationship.control.businessObject.collectionType).toBe('scalar'); expect(container.querySelector('.data-graph-collection-text')).toBeNull(); expect(saved.edges).toHaveLength(1); expect(saved.edges[0].type).toBe('domain'); modeler.destroy();
  });

  it('enforces scalar range cardinality and supports collection fan-out', async () => {
    const { modeler } = make(); await modeler.importJSON(starter); const source = modeler.addNode('objectNode'); const company = modeler.addNode('objectNode'); const person = modeler.addNode('objectNode'); const relationship = modeler.addRelationship(source.id); modeler.addTarget(relationship.control.id, company.id); expect(() => modeler.addTarget(relationship.control.id, person.id)).toThrow('at most one'); modeler.setSubtype(relationship.control.id, 'set'); modeler.addTarget(relationship.control.id, person.id);
    const saved = JSON.parse((await modeler.saveJSON()).json); expect(saved.edges.filter(edge => edge.type === 'range').map(edge => edge.target)).toEqual([company.id, person.id].sort()); modeler.destroy();
  });

  it('allows two edge nodes to share a range object', async () => {
    const { modeler } = make(); await modeler.importJSON(starter); const source = modeler.addNode('objectNode'); const target = modeler.addNode('objectNode'); const first = modeler.addRelationship(source.id); const second = modeler.addRelationship(source.id); modeler.addTarget(first.control.id, target.id); modeler.addTarget(second.control.id, target.id); const saved = JSON.parse((await modeler.saveJSON()).json); expect(saved.edges.filter(edge => edge.type === 'range' && edge.target === target.id)).toHaveLength(2); modeler.destroy();
  });

  it('persists mutation nodes, modifier edges, and parameter properties', async () => {
    const { container, modeler } = make(); await modeler.importJSON(starter); const source = modeler.addNode('objectNode'); const relationship = modeler.addRelationship(source.id); const entries = modeler.get('dataGraphContextPad').getContextPadEntries(relationship.control); entries['append-edge-property'].action.click(); entries['append-mutation'].action.click(); const saved = JSON.parse((await modeler.saveJSON()).json);
    const mutation = saved.nodes.find(node => node.type === 'mutationNode'); expect(mutation).toMatchObject({ mutationType: 'query' }); expect(saved.edges).toContainEqual(expect.objectContaining({ id: `${mutation.id}_modifier_${relationship.id}`, type: 'modifier', source: mutation.id, target: relationship.id }));
    const mutationShape = modeler.get('elementRegistry').get(mutation.id); modeler.get('dataGraphContextPad').getContextPadEntries(mutationShape)['append-property'].action.click(); const parameter = modeler.get('elementRegistry').filter(element => element.businessObject?.ownerId === mutation.id)[0]; modeler.setSubtype(parameter.id, 'uri'); const withParameter = JSON.parse((await modeler.saveJSON()).json); expect(withParameter.properties).toContainEqual(expect.objectContaining({ ownerId: mutation.id, dataType: 'uri' })); expect(modeler.get('dataGraphTypeMenu').getPopupMenuEntries(parameter).uri.label).toContain('URI'); expect(container.querySelector('.data-graph-edge--mutation').getAttribute('marker-end')).toContain('mutation-target'); modeler.destroy();
  });

  it('attaches a function directly to an object node', async () => {
    const { modeler } = make(); await modeler.importJSON(starter); const object = modeler.addNode('objectNode'); const entries = modeler.get('dataGraphContextPad').getContextPadEntries(object); expect(entries).toHaveProperty('append-mutation'); entries['append-mutation'].action.click(); const saved = JSON.parse((await modeler.saveJSON()).json); const mutation = saved.nodes.find(node => node.type === 'mutationNode'); expect(saved.edges).toContainEqual(expect.objectContaining({ id: `${mutation.id}_modifier_${object.id}`, type: 'modifier', source: mutation.id, target: object.id })); modeler.destroy();
  });

  it('offers directional Unicode function marks in the mutation type chooser', async () => {
    const { container, modeler } = make(); await modeler.importJSON(starter); const mutation = modeler.addNode('mutationNode'); const menu = modeler.get('dataGraphTypeMenu').getPopupMenuEntries(mutation); expect(menu.query.imageHtml).toContain('←ƒ'); expect(menu.function.imageHtml).toContain('ƒ→'); expect(menu.transformation.imageHtml).toContain('ƒ'); const mark = container.querySelector('.data-graph-mutation-mark'); expect(mark.textContent).toBe('←ƒ'); expect(mark.classList).toContain('data-graph-type-mark'); modeler.destroy();
  });

  it('keeps multiple functions movable and resolves a drop away from other objects', async () => {
    const { modeler } = make(); await modeler.importJSON(starter);
    const object = modeler.addNode('objectNode');
    const entries = modeler.get('dataGraphContextPad').getContextPadEntries(object);
    entries['append-mutation'].action.click(); entries['append-mutation'].action.click();
    const mutations = modeler.get('elementRegistry').filter(element => element.businessObject?.type === 'mutationNode');
    expect(mutations).toHaveLength(2);
    const before = mutations.map(centerOf); expect(before[0]).not.toEqual(before[1]);
    modeler.moveNode(mutations[0].id, centerOf(object).x, centerOf(object).y);
    const moved = centerOf(mutations[0]);
    expect(Math.abs(moved.x - centerOf(object).x)).toBeGreaterThanOrEqual((mutations[0].width + object.width) / 2);
    modeler.moveNode(mutations[1].id, 720, 420);
    expect(centerOf(mutations[1])).toEqual({ x: 720, y: 420 });
    modeler.destroy();
  });

  it('imports normalized property ownership and restores edge-node geometry', async () => {
    const value = { format: 'diagram-studio/data-graph', version: 2, nodes: [{ id: 'person', type: 'objectNode', label: 'Person', x: 100, y: 100 }, { id: 'works_for', type: 'edgeNode', label: 'WORKS_FOR', collectionType: 'scalar', x: 300, y: 200 }, { id: 'company', type: 'objectNode', label: 'Company', x: 500, y: 100 }], edges: [{ id: 'works_for_domain', type: 'domain', source: 'person', target: 'works_for', waypoints: [] }, { id: 'works_for_range_company', type: 'range', source: 'works_for', target: 'company', waypoints: [] }], properties: [{ id: 'since', type: 'property', ownerId: 'works_for_range_company', label: 'since', dataType: 'date', x: 420, y: 280 }], contextStore: { kind: 'sqlite', uri: 'test.context.sqlite', schemaVersion: 1, diagramId: 'test-diagram', revision: 0 } };
    const { modeler } = make(); await modeler.importJSON(JSON.stringify(value)); expect(centerOf(modeler.get('elementRegistry').get('works_for'))).toEqual({ x: 300, y: 200 }); const saved = JSON.parse((await modeler.saveJSON()).json); expect(saved.properties[0].ownerId).toBe('works_for_range_company'); modeler.destroy();
  });

  it('offers edge-node completion and type actions', async () => {
    const { modeler } = make(); await modeler.importJSON(starter); const source = modeler.addNode('objectNode'); const relationship = modeler.addRelationship(source.id); const entries = modeler.get('dataGraphContextPad').getContextPadEntries(relationship.control); expect(entries).toHaveProperty('connect-object'); expect(entries).toHaveProperty('append-object'); expect(entries).toHaveProperty('append-edge-property'); const types = modeler.get('dataGraphTypeMenu').getPopupMenuEntries(relationship.control); expect(types.scalar.label).toContain('Scalar (0..1)'); modeler.destroy();
  });

  it('offers exactly Relationship and Object link as object connection choices', async () => {
    const { modeler } = make(); await modeler.importJSON(starter); const object = modeler.addNode('objectNode'); const entries = modeler.get('dataGraphContextPad').getContextPadEntries(object); expect(entries.relationship.title).toBe('Relationship'); expect(entries['object-link'].title).toBe('Object link'); expect(entries).not.toHaveProperty('connect'); modeler.destroy();
  });

  it('places a targeted relationship edge node at the object midpoint', async () => {
    const { modeler } = make(); await modeler.importJSON(starter); const source = modeler.addNode('objectNode'); const target = modeler.addNode('objectNode'); modeler.moveNode(source.id, 100, 100); modeler.moveNode(target.id, 500, 300); const relationship = modeler.connect(source.id, target.id); expect(centerOf(relationship.control)).toEqual({ x: 300, y: 200 }); modeler.destroy();
  });

  it('converts an ordinary object connection gesture into an atomic relationship', async () => {
    const { modeler } = make(); await modeler.importJSON(starter); const source = modeler.addNode('objectNode'); const target = modeler.addNode('objectNode'); source.businessObject._connectionIntent = 'relationship'; const attrs = modeler.get('rules').allowed('connection.create', { source, target }); modeler.get('modeling').connect(source, target, attrs); const registry = modeler.get('elementRegistry'); const edgeNode = registry.filter(element => element.businessObject?.type === 'edgeNode')[0]; expect(edgeNode).toBeTruthy(); expect(registry.filter(element => element.businessObject?.type === 'relationshipGesture')).toHaveLength(0); modeler.get('commandStack').undo(); expect(registry.get(edgeNode.id)).toBeUndefined(); modeler.get('commandStack').redo(); expect(registry.get(edgeNode.id)).toBeTruthy(); modeler.destroy();
  });

  it('creates, renders, edits, exports, imports, deletes, and restores typed object links', async () => {
    const { container, modeler } = make(); await modeler.importJSON(starter); const source = modeler.addNode('objectNode'); const target = modeler.addNode('objectNode'); const link = modeler.addObjectLink(source.id, target.id); const linkPath = () => container.querySelector('.data-graph-edge--object-link'); expect(linkPath().hasAttribute('marker-start')).toBe(false); expect(link.businessObject.linkType).toBe('sameAs'); expect(modeler.get('dataGraphContextPad').getContextPadEntries(link)).toHaveProperty('set-type'); modeler.setSubtype(link.id, 'subclassOf'); expect(linkPath().getAttribute('marker-start')).toContain('object-link-subclassOf'); expect(container.querySelectorAll('#data-graph-object-link-subclassOf circle')).toHaveLength(1); expect(link.businessObject.linkType).toBe('subclassOf'); modeler.get('commandStack').undo(); expect(link.businessObject.linkType).toBe('sameAs'); modeler.get('commandStack').redo();
    const json = (await modeler.saveJSON()).json; expect(JSON.parse(json).edges.find(edge => edge.id === link.id)).toMatchObject({ type: 'objectLink', linkType: 'subclassOf' }); await modeler.importJSON(json); const restored = modeler.get('elementRegistry').get(link.id); expect(restored.businessObject.linkType).toBe('subclassOf'); modeler.get('modeling').removeConnection(restored); expect(modeler.get('elementRegistry').get(link.id)).toBeUndefined(); modeler.get('commandStack').undo(); expect(modeler.get('elementRegistry').get(link.id)).toBeTruthy(); modeler.destroy();
  });

  it('rejects invalid and duplicate object links', async () => {
    const { modeler } = make(); await modeler.importJSON(starter); const source = modeler.addNode('objectNode'); const target = modeler.addNode('objectNode'); modeler.addObjectLink(source.id, target.id); expect(() => modeler.addObjectLink(source.id, target.id)).toThrow('already exists'); expect(() => modeler.addObjectLink(source.id, source.id)).toThrow('distinct'); const property = modeler.addNode('property', source.id); expect(() => modeler.addObjectLink(source.id, property.id)).toThrow('distinct'); modeler.destroy();
  });

  it('limits the Object link gesture to eligible object targets', async () => {
    const { modeler } = make(); await modeler.importJSON(starter); const source = modeler.addNode('objectNode'); const target = modeler.addNode('objectNode'); const property = modeler.addNode('property', source.id); source.businessObject._connectionIntent = 'objectLink'; expect(modeler.get('rules').allowed('connection.create', { source, target })).toMatchObject({ type: 'dg:objectLink', businessObject: { linkType: 'sameAs' } }); expect(modeler.get('rules').allowed('connection.create', { source, target: property })).toBe(false); expect(modeler.get('rules').allowed('connection.create', { source, target: source })).toBe(false); modeler.destroy();
  });

  it('resolves linked effective connections without changing export topology', async () => {
    const { modeler } = make(); await modeler.importJSON(starter); const person = modeler.addNode('objectNode'); const coworker = modeler.addNode('objectNode'); modeler.addRelationship(person.id); modeler.addObjectLink(coworker.id, person.id); const before = JSON.parse((await modeler.saveJSON()).json); expect(modeler.effectiveConnections(coworker.id)).toHaveLength(1); const after = JSON.parse((await modeler.saveJSON()).json); expect(after).toEqual(before); modeler.destroy();
  });

  it('undoes and redoes a complete targeted relationship atomically', async () => {
    const { modeler } = make(); await modeler.importJSON(starter); const source = modeler.addNode('objectNode'); const target = modeler.addNode('objectNode'); const relationship = modeler.connect(source.id, target.id); modeler.get('commandStack').undo(); expect(modeler.get('elementRegistry').get(`${relationship.id}-label`)).toBeUndefined(); expect(modeler.get('elementRegistry').get(relationship.id)).toBeUndefined(); expect(modeler.get('elementRegistry').get(relationship.segments[0].id)).toBeUndefined(); modeler.get('commandStack').redo(); expect(modeler.get('elementRegistry').get(relationship.id)).toBeTruthy(); modeler.destroy();
  });

  it('deletes one range independently', async () => {
    const { modeler } = make(); await modeler.importJSON(starter); const source = modeler.addNode('objectNode'); const a = modeler.addNode('objectNode'); const b = modeler.addNode('objectNode'); const relationship = modeler.addRelationship(source.id); modeler.setSubtype(relationship.control.id, 'set'); const first = modeler.addTarget(relationship.control.id, a.id); modeler.addTarget(relationship.control.id, b.id); modeler.get('modeling').removeConnection(first); const saved = JSON.parse((await modeler.saveJSON()).json); expect(saved.nodes.find(node => node.id === relationship.id)).toBeTruthy(); expect(saved.edges.filter(edge => edge.type === 'range')).toHaveLength(1); modeler.destroy();
  });

  it('deletes a domain cascade while retaining range object nodes', async () => {
    const { modeler } = make(); await modeler.importJSON(starter); const source = modeler.addNode('objectNode'); const target = modeler.addNode('objectNode'); const relationship = modeler.connect(source.id, target.id); modeler.get('modeling').removeConnection(relationship.segments[0]); expect(modeler.get('elementRegistry').get(relationship.id)).toBeUndefined(); expect(modeler.get('elementRegistry').get(target.id)).toBeTruthy(); modeler.get('commandStack').undo(); expect(modeler.get('elementRegistry').get(relationship.id)).toBeTruthy(); modeler.destroy();
  });

  it('marks topology changes dirty and supports undo/redo', async () => {
    const { modeler } = make(); await modeler.importJSON(starter); const changed = vi.fn(); modeler.on('commandStack.changed', changed); const source = modeler.addNode('objectNode'); const relationship = modeler.addRelationship(source.id); expect(changed).toHaveBeenCalled(); modeler.get('commandStack').undo(); modeler.get('commandStack').redo(); expect(modeler.get('elementRegistry').get(relationship.id)).toBeTruthy(); modeler.destroy();
  });

  it('moves every node kind independently and persists adjusted coordinates', async () => {
    const { modeler } = make(); await modeler.importJSON(starter); const object = modeler.addNode('objectNode'); const relationship = modeler.addRelationship(object.id); const mutation = modeler.addNode('mutationNode'); const property = modeler.addNode('property', object.id); const modeling = modeler.get('modeling'); const registry = modeler.get('elementRegistry'); modeling.connect(mutation, object, { id: `${mutation.id}_modifier_${object.id}`, type: 'dg:modifier', businessObject: { type: 'modifier' } }); modeling.connect(object, property, { id: `${property.id}_ownership`, type: 'dg:attachment', businessObject: { type: 'attachment', ownerId: object.id } });
    const expected = new Map([[object.id, { x: 140, y: 120 }], [relationship.id, { x: 310, y: 210 }], [mutation.id, { x: 460, y: 130 }], [property.id, { x: 520, y: 260 }]]); const label = registry.get(`${relationship.id}-label`); const labelBefore = centerOf(label); const changed = vi.fn(); modeler.on('commandStack.changed', changed); expected.forEach(({ x, y }, id) => modeler.moveNode(id, x, y)); expect(centerOf(label)).not.toEqual(labelBefore); expect(changed).toHaveBeenCalled();
    const saved = JSON.parse((await modeler.saveJSON()).json); expected.forEach((position, id) => { const record = [...saved.nodes, ...saved.properties].find(item => item.id === id); expect({ x: record.x, y: record.y }).toEqual(position); }); modeler.get('commandStack').undo(); expect(centerOf(registry.get(property.id))).not.toEqual(expected.get(property.id)); modeler.get('commandStack').redo(); expect(centerOf(registry.get(property.id))).toEqual(expected.get(property.id)); modeler.destroy();
  });

  it('edits, undoes, redoes, deletes, and restores per-node Markdown drafts', async () => {
    const { modeler } = make(); await modeler.importJSON(starter); const node = modeler.addNode('objectNode');
    modeler.setNodeMarkdown(node.id, '# Overview\nPerson context'); expect(modeler.nodeMarkdown(node.id)).toContain('Person context');
    modeler.get('commandStack').undo(); expect(modeler.nodeMarkdown(node.id)).toBe(''); modeler.get('commandStack').redo(); expect(modeler.nodeMarkdown(node.id)).toContain('Person context');
    modeler.get('modeling').removeShape(node); expect(modeler.contextSnapshot().tombstones).toContain(node.id); modeler.get('commandStack').undo(); expect(modeler.nodeMarkdown(node.id)).toContain('Person context'); modeler.destroy();
  });

  it('imports captured context drafts for inspector reopening', async () => {
    const { modeler } = make(); await modeler.importJSON(starter); const node = modeler.addNode('objectNode');
    modeler.importContextDrafts([{ contextId: `context-${node.id}`, nodeId: node.id, markdown: '# Saved\nContext\n', sections: [] }]);
    expect(modeler.nodeMarkdown(node.id)).toBe('# Saved\nContext\n'); modeler.destroy();
  });
});
