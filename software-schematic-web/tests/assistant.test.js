import { describe, expect, it, vi } from 'vitest';
import { approveProposal, assistantComposerIntent, assessInterviewCoverage, assertRevertRevision, buildContextSnapshot, buildEndpointAnnotations, createInterviewState, createProposalBeforeState, describeOperation, DIAGRAM_OPERATION_REGISTRY, executeModelerOperation, INTERVIEW_TOPICS, isAssistantEligible, normalizeAssistantPath, proposalGroups, recordInterviewAnswers, stableRevision, validateProposal } from '../src/assistant.js';

function modeler(nodes = []) {
  return { get(name) { if (name === 'elementRegistry') return { getAll: () => nodes }; throw new Error(name); } };
}

function snapshot() {
  return { version: '2.0', requestId: 'r1', sourceRevision: 'rev', diagramPath: 'main.bpmn', processName: null, graph: { nodes: [{ id: 'Task_1', type: 'bpmn:Task', name: 'sales.Order#work', label: 'Work', status: 'open' }], flows: [] } };
}

describe('assistant contracts', () => {
  it('maps composer keyboard input without submitting newlines, composition, empty text, or inactive phases', () => {
    expect(assistantComposerIntent({ key: 'a' }, 'Explain the flow')).toBe('none');
    expect(assistantComposerIntent({ key: 'Enter', shiftKey: true }, 'Explain the flow')).toBe('newline');
    expect(assistantComposerIntent({ key: 'Enter', isComposing: true }, '設計')).toBe('composing');
    expect(assistantComposerIntent({ key: 'Enter', keyCode: 229 }, '設計')).toBe('composing');
    expect(assistantComposerIntent({ key: 'Enter' }, '   ')).toBe('empty');
    expect(assistantComposerIntent({ key: 'Enter' }, 'Explain the flow', 'preview')).toBe('none');
    expect(assistantComposerIntent({ key: 'Enter' }, 'Explain the flow')).toBe('send');
  });

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
    expect(value).toMatchObject({ primaryElementId: 'Task_1', primaryElementKind: 'node' });
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
    const scoped = await buildContextSnapshot({ scope: 'edge', tab, primaryElement: flow, diagramMarkdown: '# Order', edgeMarkdown: '# Approved flow' });
    expect(scoped).toMatchObject({ primaryElementId: 'Flow_7', primaryElementKind: 'edge', primaryEdgeId: 'Flow_7', edgeMarkdown: '# Approved flow' });
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
    expect(() => validateProposal({ ...plan, operations: [{ type: 'update_node_label', diagramPath: 'other/main.bpmn', nodeId: 'Task_1', label: 'No' }] }, base)).toThrow(/outside this proposal scope/);
    expect(() => validateProposal({ ...plan, operations: [{ type: 'create_process', qualifiedName: 'sales.Order' }, { type: 'open_process', qualifiedName: 'sales.ORder' }] }, base)).toThrow(/collision/);
    expect(() => validateProposal({ ...plan, operations: [{ type: 'add_flow_node', nodeId: 'Task_1', bpmnType: 'bpmn:Task' }] }, base)).toThrow(/Duplicate/);
    expect(() => validateProposal({ ...plan, operations: [{ type: 'update_node_name', diagramPath: 'main.bpmn', nodeId: 'Task_1', name: 'work' }] }, base)).toThrow(/Qualified/);
    expect(validateProposal({ ...plan, operations: [{ type: 'set_node_status', diagramPath: 'main.bpmn', nodeId: 'Task_1', status: 'new' }] }, base).operations[0].status).toBe('new');
    expect(() => validateProposal({ ...plan, operations: [{ type: 'set_node_status', diagramPath: 'main.bpmn', nodeId: 'Task_1', status: 'pending' }] }, base)).toThrow(/Unsupported node status/);
    const locked = structuredClone(base); locked.graph.nodes[0].status = 'locked';
    expect(() => validateProposal({ ...plan, operations: [{ type: 'update_node_label', nodeId: 'Task_1', label: 'No' }] }, locked)).toThrow(/Locked/);
    expect(() => validateProposal(plan, base, { currentRevision: 'later' })).toThrow(/stale/);
  });

  it('keeps element-scoped proposals acute while allowing an anchored child composition', () => {
    const base = snapshot();
    base.scope = 'node'; base.primaryElementId = 'Task_1'; base.primaryNodeId = 'Task_1'; base.diagramKind = 'bpmn';
    base.graph.nodes.push({ id: 'Task_2', type: 'bpmn:Task', name: 'sales.Order#peer', label: 'Peer', status: 'open' });
    const selected = { version: '2.0', requestId: 'r1', sourceRevision: 'rev', summary: 'Focus', operations: [
      { type: 'update_node_label', diagramPath: 'main.bpmn', nodeId: 'Task_1', label: 'Focused work' },
    ] };
    expect(validateProposal(selected, base)).toBe(selected);
    expect(() => validateProposal({ ...selected, operations: [{ ...selected.operations[0], nodeId: 'Task_2' }] }, base)).toThrow(/diagram-level assistance/);
    expect(() => validateProposal({ ...selected, operations: [{ type: 'connect_sequence_flow', diagramPath: 'main.bpmn', flowId: 'Flow_2', sourceId: 'Task_1', targetId: 'Task_2' }] }, base)).toThrow(/diagram-level assistance/);

    const child = { ...selected, summary: 'Create child', operations: [
      { type: 'set_process_reference', diagramPath: 'main.bpmn', nodeId: 'Task_1', qualifiedName: 'sales.FocusedWork' },
      { type: 'create_process', qualifiedName: 'sales.FocusedWork' },
      { type: 'add_flow_node', diagramPath: 'sales/FocusedWork/main.bpmn', nodeId: 'Child_1', bpmnType: 'bpmn:Task', name: 'sales.FocusedWork#step', label: 'Do step' },
      { type: 'replace_node_markdown', diagramPath: 'sales/FocusedWork/main.bpmn', nodeId: 'Child_1', markdown: '# Do step\n\nComplete the focused step.' },
    ] };
    expect(validateProposal(child, base)).toBe(child);

    const diagram = structuredClone(base); diagram.scope = 'diagram'; diagram.primaryElementId = null; diagram.primaryNodeId = null;
    expect(validateProposal({ ...selected, operations: [{ ...selected.operations[0], nodeId: 'Task_2' }] }, diagram)).toBeTruthy();
  });

  it('groups readable previews and limits eligibility to supported shapes', () => {
    const groups = proposalGroups({ operations: [{ type: 'update_node_label', diagramPath: 'main.bpmn', nodeId: 'Task_1', label: 'Review' }] });
    expect(groups.get('main.bpmn')[0]).toContain('Review');
    expect(isAssistantEligible({ businessObject: { $type: 'bpmn:Task' } })).toBe(true);
    expect(isAssistantEligible({ businessObject: { $type: 'cmmn:PlanItem', definitionRef: { $type: 'cmmn:ProcessTask' } } })).toBe(true);
    expect(isAssistantEligible({ waypoints: [], businessObject: { $type: 'bpmn:SequenceFlow' } })).toBe(false);
  });

  it('advertises executable structured operations and validates an S3 collaboration proposal', () => {
    expect(DIAGRAM_OPERATION_REGISTRY.version).toBe('2.0');
    for (const capability of Object.values(DIAGRAM_OPERATION_REGISTRY.operations)) {
      expect(capability).toMatchObject({ preview: true, apply: true, undo: true, rollback: true });
    }
    const base = snapshot(); base.diagramKind = 'bpmn';
    const plan = { version: '2.0', requestId: 'r1', sourceRevision: 'rev', summary: 'Add S3 collaboration', operations: [
      { type: 'add_participant', diagramPath: 'main.bpmn', participantId: 'Participant_S3', label: 'Amazon S3', x: 500, y: 420 },
      { type: 'connect_message_flow', diagramPath: 'main.bpmn', flowId: 'MessageFlow_Save', sourceId: 'Task_1', targetId: 'Participant_S3', label: 'putObject' },
      { type: 'replace_edge_markdown', diagramPath: 'main.bpmn', edgeId: 'MessageFlow_Save', markdown: '# putObject\n\nProducer: save data\nConsumer: S3\nPayload: object bytes and key.' },
    ] };
    expect(validateProposal(plan, base)).toBe(plan);
    expect(proposalGroups(plan).get('main.bpmn')).toEqual(expect.arrayContaining([expect.stringContaining('Amazon S3'), expect.stringContaining('Task_1')]));
    expect(() => validateProposal({ ...plan, operations: [{ ...plan.operations[1], targetId: 'Missing' }] }, base)).toThrow(/unknown/);
    expect(() => validateProposal({ ...plan, operations: plan.operations.slice(0, 2) }, base)).toThrow(/requires an edge Markdown contract/);
    expect(() => validateProposal({ ...plan, operations: [{ type: 'update_node_label', nodeId: 'Task_1', label: 'First validate every field then connect to the database then save all records and publish four events' }] }, base)).toThrow(/concise/);
  });

  it('previews every advertised operation and rejects preview gaps', () => {
    const samples = {
      replace_node_type: { nodeId: 'Task_1', bpmnType: 'bpmn:ServiceTask' },
      update_node_label: { nodeId: 'Task_1', label: 'Save data' },
      update_node_name: { nodeId: 'Task_1', name: 'sales.Order#saveData' },
      set_node_status: { nodeId: 'Task_1', status: 'modify' },
      set_process_reference: { nodeId: 'Task_1', qualifiedName: 'sales.SaveData' },
      create_process: { qualifiedName: 'sales.SaveData' },
      open_process: { qualifiedName: 'sales.SaveData' },
      rename_process: { oldQualifiedName: 'sales.Order', newQualifiedName: 'sales.RenamedOrder' },
      add_flow_node: { nodeId: 'Task_2', bpmnType: 'bpmn:Task', label: 'Store object' },
      connect_sequence_flow: { sourceId: 'Task_1', targetId: 'Task_2' },
      add_participant: { participantId: 'Participant_S3', label: 'Amazon S3' },
      connect_message_flow: { sourceId: 'Task_1', targetId: 'Participant_S3', label: 'putObject' },
      move_element: { elementId: 'Task_1', x: 100, y: 200 },
      remove_element: { elementId: 'Task_1' },
      disconnect_flow: { flowId: 'Flow_1' },
      add_plan_item: { nodeId: 'PlanItem_1', cmmnType: 'cmmn:Task', label: 'Capture need' },
      connect_cmmn: { sourceId: 'PlanItem_1', targetId: 'PlanItem_2' },
      replace_diagram_markdown: {},
      replace_node_markdown: { nodeId: 'Task_1' },
      replace_edge_markdown: { edgeId: 'Flow_1' },
    };
    expect(Object.keys(samples).sort()).toEqual(Object.keys(DIAGRAM_OPERATION_REGISTRY.operations).sort());
    for (const [type, sample] of Object.entries(samples)) expect(describeOperation({ type, ...sample })).not.toBe(type);
    expect(() => describeOperation({ type: 'missing_executor' })).toThrow(/Unsupported operation preview/);
  });

  it('shares approval, modeler execution, and revert guards with the browser', () => {
    const base = snapshot();
    const plan = { version: '2.0', requestId: 'r1', sourceRevision: 'rev', summary: 'Rename', operations: [{ type: 'update_node_label', diagramPath: 'main.bpmn', nodeId: 'Task_1', label: 'Save data' }] };
    expect(approveProposal(plan, base, 'rev').proposal).toBe(plan);
    expect(() => approveProposal(plan, base, 'newer')).toThrow(/stale/);
    expect(() => assertRevertRevision('accepted', 'newer')).toThrow(/changed after this assistant proposal/);
    expect(() => assertRevertRevision('accepted', 'accepted')).not.toThrow();
    const before = createProposalBeforeState('rev');
    expect(before).toMatchObject({ revision: 'rev', schematicRevision: null, createdCompositions: [], openedTabs: [], processRenames: [] });
    expect(before.diagrams).toBeInstanceOf(Map);
    expect(before.documents).toBeInstanceOf(Map);

    const element = { id: 'Task_1', x: 10, y: 20 };
    const modeling = { moveElements: vi.fn(), removeElements: vi.fn(), removeConnection: vi.fn(), connect: vi.fn(), createShape: vi.fn(), updateProperties: vi.fn() };
    const adapter = { updateLabel: vi.fn(), updateName: vi.fn(), updateStatus: vi.fn() };
    const tab = {
      adapter,
      nodeStatuses: new Map(),
      modeler: { get: (name) => ({ elementRegistry: { get: () => element }, modeling }[name]) },
    };
    expect(executeModelerOperation(plan.operations[0], tab)).toBe(true);
    expect(adapter.updateLabel).toHaveBeenCalledWith(element, 'Save data');
    expect(executeModelerOperation({ type: 'move_element', elementId: 'Task_1', x: 40, y: 70 }, tab)).toBe(true);
    expect(modeling.moveElements).toHaveBeenCalledWith([element], { x: 30, y: 50 });
    expect(executeModelerOperation({ type: 'replace_node_markdown' }, tab)).toBe(false);
  });

  it('resumes bounded interviews and asks one focused material question at a time', () => {
    const initial = createInterviewState(snapshot());
    expect(assessInterviewCoverage(initial)).toMatchObject({ ready: false, missing: INTERVIEW_TOPICS, nextQuestion: 'Who performs or participates in this flow?' });
    const answers = Object.fromEntries(INTERVIEW_TOPICS.map((topic) => [topic, topic === 'alternatives' ? { notApplicable: true, reason: 'No alternate path' } : `confirmed ${topic}`]));
    const resumed = recordInterviewAnswers(initial, answers);
    const roundTripped = JSON.parse(JSON.stringify(resumed));
    expect(roundTripped.round).toBe(1);
    expect(roundTripped.decisions.alternatives.disposition).toBe('notApplicable');
    expect(assessInterviewCoverage(roundTripped)).toMatchObject({ ready: true, missing: [], nextQuestion: null });
    const unresolved = recordInterviewAnswers(roundTripped, { unresolved: ['Choose the retry owner'] });
    expect(assessInterviewCoverage(unresolved).nextQuestion).toContain('Choose the retry owner');
  });

  it('checks service endpoint coverage and renders confirmed facts into owned Markdown', () => {
    const base = recordInterviewAnswers(createInterviewState(snapshot()), {
      ...Object.fromEntries(INTERVIEW_TOPICS.map((topic) => [topic, `confirmed ${topic}`])),
      crossesSystemBoundary: true,
    });
    expect(assessInterviewCoverage(base).nextQuestion).toContain('Which service endpoints');
    const incomplete = recordInterviewAnswers(base, { serviceEndpoints: [{ id: 's3-put', label: 'putObject', caller: 'Save data', receiver: 'S3' }] });
    expect(assessInterviewCoverage(incomplete)).toMatchObject({ ready: false, endpointGaps: [{ endpointId: 's3-put' }] });
    expect(assessInterviewCoverage(incomplete).nextQuestion).toContain('method, operation');

    const endpoint = {
      id: 's3-put', label: 'putObject', methodOrOperation: 'PUT', routeOrTopic: '/objects/{key}',
      caller: 'Save data', receiver: 'S3', payloadPurpose: 'Store object bytes under the requested key',
      trustBoundary: 'Workload identity with bucket-scoped write permission', successOutcome: 'ETag acknowledgement',
      timeoutRetry: '5 second timeout; retry twice with jitter', failurePath: 'Route to Store object failed',
    };
    const complete = recordInterviewAnswers(base, { serviceEndpoints: [endpoint] });
    expect(assessInterviewCoverage(complete).ready).toBe(true);
    expect(buildEndpointAnnotations(endpoint)).toMatchObject({ label: 'putObject' });
    expect(buildEndpointAnnotations(endpoint).activityMarkdown).toContain('PUT /objects/{key}');
    expect(buildEndpointAnnotations(endpoint).messageMarkdown).toContain('Payload purpose: Store object bytes');
  });

  it('validates move, removal, disconnection, and edge annotation operations', () => {
    const base = snapshot(); base.diagramKind = 'bpmn';
    base.graph.flows.push({ id: 'Flow_1', type: 'bpmn:SequenceFlow', name: null, label: '', status: 'open', source: 'Task_1', target: 'Task_1' });
    const plan = { version: '2.0', requestId: 'r1', sourceRevision: 'rev', summary: 'Refine layout', operations: [
      { type: 'move_element', diagramPath: 'main.bpmn', elementId: 'Task_1', x: 300, y: 180 },
      { type: 'replace_edge_markdown', diagramPath: 'main.bpmn', edgeId: 'Flow_1', markdown: '# Transfer\nConfirmed delivery.' },
      { type: 'disconnect_flow', diagramPath: 'main.bpmn', flowId: 'Flow_1' },
      { type: 'remove_element', diagramPath: 'main.bpmn', elementId: 'Task_1' },
    ] };
    expect(validateProposal(plan, base)).toBe(plan);
    expect(proposalGroups(plan).get('main.bpmn')).toHaveLength(4);
    const locked = structuredClone(base); locked.graph.flows[0].status = 'locked';
    expect(() => validateProposal({ ...plan, operations: [plan.operations[2]] }, locked)).toThrow(/Locked/);
  });

  it('requires activity and event details in owned Markdown instead of labels', () => {
    const base = snapshot(); base.diagramKind = 'bpmn';
    const event = { type: 'add_flow_node', diagramPath: 'main.bpmn', nodeId: 'Event_ObjectStored', bpmnType: 'bpmn:StartEvent', name: 'sales.Order#objectStored', label: 'Object stored', x: 120, y: 180 };
    const documented = { version: '2.0', requestId: 'r1', sourceRevision: 'rev', summary: 'Add event', operations: [
      event,
      { type: 'replace_node_markdown', diagramPath: 'main.bpmn', nodeId: 'Event_ObjectStored', markdown: '# Object stored\n\nTrigger: S3 object-created notification\nCorrelation: object key\nDelivery: at least once\nIdempotency: deduplicate by event ID.' },
    ] };
    expect(validateProposal(documented, base)).toBe(documented);
    expect(() => validateProposal({ ...documented, operations: [event] }, base)).toThrow(/requires owned Markdown/);
  });

  it('validates bounded CMMN plan-item, connection, and Process Task operations', () => {
    const base = {
      version: '2.0', requestId: 'r1', sourceRevision: 'rev', diagramPath: 'cybling/main.cmmn', diagramKind: 'cmmn', packageName: 'cybling',
      graph: { nodes: [{ id: 'PlanItem_Need', type: 'cmmn:HumanTask', name: 'cybling#captureNeed', label: 'Capture need', status: 'open' }], flows: [] },
    };
    const plan = {
      version: '2.0', requestId: 'r1', sourceRevision: 'rev', summary: 'Trace need to design', operations: [
        { type: 'add_plan_item', diagramPath: 'cybling/main.cmmn', nodeId: 'PlanItem_Birth', cmmnType: 'cmmn:ProcessTask', name: 'cybling.sdk.Birth', label: 'Birth design' },
        { type: 'replace_node_markdown', diagramPath: 'cybling/main.cmmn', nodeId: 'PlanItem_Birth', markdown: '# Birth design\n\nTrace the need into its BPMN design.' },
        { type: 'connect_cmmn', diagramPath: 'cybling/main.cmmn', connectionId: 'Association_Birth', sourceId: 'PlanItem_Need', targetId: 'PlanItem_Birth' },
        { type: 'update_node_name', diagramPath: 'cybling/main.cmmn', nodeId: 'Association_Birth', name: 'cybling#birthTrace' },
        { type: 'set_process_reference', diagramPath: 'cybling/main.cmmn', nodeId: 'PlanItem_Birth', qualifiedName: 'cybling.sdk.Birth' },
      ],
    };

    expect(validateProposal(plan, base)).toBe(plan);
    expect(() => validateProposal({ ...plan, operations: [{ ...plan.operations[0], name: 'not-qualified' }] }, base)).toThrow(/Name/);
    expect(() => validateProposal({ ...plan, operations: [{ ...plan.operations[2], targetId: 'Missing' }] }, base)).toThrow(/unknown/);
    expect(() => validateProposal({ ...plan, operations: [{ type: 'add_plan_item', nodeId: 'PlanItem_2', cmmnType: 'cmmn:Stage' }] }, snapshot())).toThrow(/not supported for BPMN|only be added to a CMMN/);
    const lockedEdge = structuredClone(base);
    lockedEdge.graph.flows.push({ id: 'Association_Locked', type: 'cmmn:Association', name: 'cybling#lockedTrace', status: 'locked', source: 'PlanItem_Need', target: 'PlanItem_Need' });
    expect(() => validateProposal({ ...plan, operations: [{ type: 'update_node_name', nodeId: 'Association_Locked', name: 'cybling#changedTrace' }] }, lockedEdge)).toThrow(/Locked/);
  });
});
