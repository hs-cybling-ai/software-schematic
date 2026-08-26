import { describe, expect, it, vi } from 'vitest';
import { buildContextSnapshot, isAssistantEligible, normalizeAssistantPath, proposalGroups, stableRevision, validateProposal } from '../src/assistant.js';

function modeler(nodes = []) {
  return { get(name) { if (name === 'elementRegistry') return { getAll: () => nodes }; throw new Error(name); } };
}

function snapshot() {
  return { version: '1.0', requestId: 'r1', sourceRevision: 'rev', diagramPath: 'main.bpmn', graph: { nodes: [{ id: 'Task_1', type: 'bpmn:Task', label: 'Work', status: 'open' }], flows: [] } };
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
    expect(value.sourceRevision).toHaveLength(16);
  });

  it('rejects malformed, unsupported, duplicate, escaped, locked, and stale plans', () => {
    const base = snapshot();
    const plan = { version: '1.0', requestId: 'r1', sourceRevision: 'rev', summary: 'Change', operations: [] };
    expect(validateProposal(plan, base)).toBe(plan);
    expect(() => validateProposal({ ...plan, operations: [{ type: 'shell' }] }, base)).toThrow(/Unsupported/);
    expect(() => validateProposal({ ...plan, operations: [{ type: 'create_composition', path: '../x' }] }, base)).toThrow(/escapes/);
    expect(() => validateProposal({ ...plan, operations: [{ type: 'add_flow_node', nodeId: 'Task_1', bpmnType: 'bpmn:Task' }] }, base)).toThrow(/Duplicate/);
    const locked = structuredClone(base); locked.graph.nodes[0].status = 'locked';
    expect(() => validateProposal({ ...plan, operations: [{ type: 'update_node_label', nodeId: 'Task_1', label: 'No' }] }, locked)).toThrow(/Locked/);
    expect(() => validateProposal(plan, base, { currentRevision: 'later' })).toThrow(/stale/);
  });

  it('groups readable previews and limits eligibility to supported shapes', () => {
    const groups = proposalGroups({ operations: [{ type: 'update_node_label', diagramPath: 'main.bpmn', nodeId: 'Task_1', label: 'Review' }] });
    expect(groups.get('main.bpmn')[0]).toContain('Review');
    expect(isAssistantEligible({ businessObject: { $type: 'bpmn:Task' } })).toBe(true);
    expect(isAssistantEligible({ waypoints: [], businessObject: { $type: 'bpmn:SequenceFlow' } })).toBe(false);
  });
});
