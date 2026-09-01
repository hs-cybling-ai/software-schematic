import { describe, expect, it, vi } from 'vitest';
import { buildContextSnapshot, isAssistantEligible, normalizeAssistantPath, proposalGroups, stableRevision, validateProposal } from '../src/assistant.js';

function modeler(nodes = []) {
  return { get(name) { if (name === 'elementRegistry') return { getAll: () => nodes }; throw new Error(name); } };
}

function snapshot() {
  return { version: '2.0', requestId: 'r1', sourceRevision: 'rev', diagramPath: 'main.bpmn', processName: null, graph: { nodes: [{ id: 'Task_1', type: 'bpmn:Task', name: 'sales.Order#work', label: 'Work', status: 'open' }], flows: [] } };
}

describe('assistant contracts', () => {
  it('normalizes confined paths and computes deterministic revisions', () => {
    expect(normalizeAssistantPath('schematics/order/main.bpmn')).toBe('order/main.bpmn');
    expect(() => normalizeAssistantPath('../secret')).toThrow(/escapes/);
    expect(stableRevision({ b: 2 })).toBe(stableRevision({ b: 2 }));
  });

  it('builds node and diagram snapshots after flushing pending edits', async () => {
    const flush = vi.fn();
    const element = { id: 'Task_1', businessObject: { $type: 'bpmn:Task', name: 'Work' } };
    const tab = { path: 'main.bpmn', modeler: modeler([element]), nodeStatuses: new Map([['Task_1', 'modify']]) };
    const value = await buildContextSnapshot({ scope: 'node', tab, primaryNode: element, diagramMarkdown: '# Main', nodeMarkdown: '# Work', flush });
    expect(flush).toHaveBeenCalledOnce();
    expect(value.primaryNodeId).toBe('Task_1');
    expect(value.graph.nodes[0].status).toBe('modify');
    expect(value.graph.nodes[0]).toEqual({ id: 'Task_1', name: null, label: 'Work', type: 'bpmn:Task', status: 'modify' });
    expect(value.sourceRevision).toHaveLength(16);
  });

  it('includes first-class edge identity in the semantic graph', async () => {
    const flow = { id: 'Flow_7', waypoints: [], businessObject: { $type: 'bpmn:SequenceFlow', name: 'Approved', architecturalName: 'sales.Order#paymentApproved', sourceRef: { id: 'Task_1' }, targetRef: { id: 'Task_2' } } };
    const tab = { path: 'sales/Order/main.bpmn', modeler: modeler([flow]), nodeStatuses: new Map([['Flow_7', 'locked']]) };
    const value = await buildContextSnapshot({ scope: 'diagram', tab, diagramMarkdown: '# Order' });
    expect(value.graph.flows[0]).toEqual({ id: 'Flow_7', type: 'bpmn:SequenceFlow', name: 'sales.Order#paymentApproved', label: 'Approved', status: 'locked', source: 'Task_1', target: 'Task_2' });
  });

  it('builds bounded CMMN business-need context through the active adapter', async () => {
    const graph = {
      nodes: [{ id: 'PlanItem_Birth', type: 'cmmn:ProcessTask', name: 'cybling.sdk.Birth', label: 'Birth Design', status: 'open' }],
      flows: [{ id: 'Association_1', type: 'cmmn:Association', name: 'cybling#birthTrace', label: '', status: 'modify', source: 'PlanItem_Need', target: 'PlanItem_Birth' }],
    };
    const adapter = { normalizedElements: vi.fn(() => graph) };
    const tab = { path: 'cybling/main.cmmn', modeler: modeler(), adapter, nodeStatuses: new Map() };
    const value = await buildContextSnapshot({ scope: 'diagram', tab, diagramMarkdown: '# Cybling need' });

    expect(adapter.normalizedElements).toHaveBeenCalledOnce();
    expect(value).toMatchObject({ diagramKind: 'cmmn', packageName: 'cybling', processName: null, contextRole: 'business-need', graph });
    expect(JSON.stringify(value)).not.toContain('/Users/');
  });

  it('rejects malformed, unsupported, duplicate, escaped, locked, and stale plans', () => {
    const base = snapshot();
    const plan = { version: '2.0', requestId: 'r1', sourceRevision: 'rev', summary: 'Change', operations: [] };
    expect(validateProposal(plan, base)).toBe(plan);
    expect(() => validateProposal({ ...plan, operations: [{ type: 'shell' }] }, base)).toThrow(/Unsupported/);
    expect(() => validateProposal({ ...plan, operations: [{ type: 'update_node_label', nodeId: 'Task_1', label: 'No', rawXml: '<bpmn />' }] }, base)).toThrow(/raw diagram XML/);
    expect(() => validateProposal({ ...plan, operations: [{ type: 'create_process', qualifiedName: '../x' }] }, base)).toThrow(/Package|Qualified/);
    expect(() => validateProposal({ ...plan, operations: [{ type: 'create_process', qualifiedName: 'sales.Order', path: 'chosen/by/provider' }] }, base)).toThrow(/cannot supply composition paths/);
    expect(() => validateProposal({ ...plan, operations: [{ type: 'create_process', qualifiedName: 'sales.Order' }, { type: 'open_process', qualifiedName: 'sales.ORder' }] }, base)).toThrow(/collision/);
    expect(() => validateProposal({ ...plan, operations: [{ type: 'add_flow_node', nodeId: 'Task_1', bpmnType: 'bpmn:Task' }] }, base)).toThrow(/Duplicate/);
    expect(() => validateProposal({ ...plan, operations: [{ type: 'update_node_name', diagramPath: 'main.bpmn', nodeId: 'Task_1', name: 'work' }] }, base)).toThrow(/Qualified/);
    const locked = structuredClone(base); locked.graph.nodes[0].status = 'locked';
    expect(() => validateProposal({ ...plan, operations: [{ type: 'update_node_label', nodeId: 'Task_1', label: 'No' }] }, locked)).toThrow(/Locked/);
    expect(() => validateProposal(plan, base, { currentRevision: 'later' })).toThrow(/stale/);
  });

  it('groups readable previews and limits eligibility to supported shapes', () => {
    const groups = proposalGroups({ operations: [{ type: 'update_node_label', diagramPath: 'main.bpmn', nodeId: 'Task_1', label: 'Review' }] });
    expect(groups.get('main.bpmn')[0]).toContain('Review');
    expect(isAssistantEligible({ businessObject: { $type: 'bpmn:Task' } })).toBe(true);
    expect(isAssistantEligible({ businessObject: { $type: 'cmmn:PlanItem', definitionRef: { $type: 'cmmn:ProcessTask' } } })).toBe(true);
    expect(isAssistantEligible({ waypoints: [], businessObject: { $type: 'bpmn:SequenceFlow' } })).toBe(false);
  });

  it('validates bounded CMMN plan-item, connection, and Process Task operations', () => {
    const base = {
      version: '2.0', requestId: 'r1', sourceRevision: 'rev', diagramPath: 'cybling/main.cmmn', diagramKind: 'cmmn', packageName: 'cybling',
      graph: { nodes: [{ id: 'PlanItem_Need', type: 'cmmn:HumanTask', name: 'cybling#captureNeed', label: 'Capture need', status: 'open' }], flows: [] },
    };
    const plan = {
      version: '2.0', requestId: 'r1', sourceRevision: 'rev', summary: 'Trace need to design', operations: [
        { type: 'add_plan_item', diagramPath: 'cybling/main.cmmn', nodeId: 'PlanItem_Birth', cmmnType: 'cmmn:ProcessTask', name: 'cybling.sdk.Birth', label: 'Birth design' },
        { type: 'connect_cmmn', diagramPath: 'cybling/main.cmmn', connectionId: 'Association_Birth', sourceId: 'PlanItem_Need', targetId: 'PlanItem_Birth' },
        { type: 'update_node_name', diagramPath: 'cybling/main.cmmn', nodeId: 'Association_Birth', name: 'cybling#birthTrace' },
        { type: 'set_process_reference', diagramPath: 'cybling/main.cmmn', nodeId: 'PlanItem_Birth', qualifiedName: 'cybling.sdk.Birth' },
      ],
    };

    expect(validateProposal(plan, base)).toBe(plan);
    expect(() => validateProposal({ ...plan, operations: [{ ...plan.operations[0], name: 'not-qualified' }] }, base)).toThrow(/Name/);
    expect(() => validateProposal({ ...plan, operations: [{ ...plan.operations[1], targetId: 'Missing' }] }, base)).toThrow(/unknown/);
    expect(() => validateProposal({ ...plan, operations: [{ type: 'add_plan_item', nodeId: 'PlanItem_2', cmmnType: 'cmmn:Stage' }] }, snapshot())).toThrow(/only be added to a CMMN/);
    const lockedEdge = structuredClone(base);
    lockedEdge.graph.flows.push({ id: 'Association_Locked', type: 'cmmn:Association', name: 'cybling#lockedTrace', status: 'locked', source: 'PlanItem_Need', target: 'PlanItem_Need' });
    expect(() => validateProposal({ ...plan, operations: [{ type: 'update_node_name', nodeId: 'Association_Locked', name: 'cybling#changedTrace' }] }, lockedEdge)).toThrow(/Locked/);
  });
});
