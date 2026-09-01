import { describe, expect, it, vi } from 'vitest';
import { architecturalName, cmmnFolderForPackageName, cmmnPathForPackageName, collisionKey, compositionBreadcrumbs, compositionFolderForQualifiedName, compositionIdentity, compositionPathFor, compositionSlug, diagramKind, diagramPathForQualifiedName, documentationPath, isRootDiagram, NODE_STATUSES, normalizeCompositionPath, normalizeNodeStatus, owningPackageName, packageNameForCmmnPath, projectDocumentTitle, qualifiedMemberName, qualifiedNameForCompositionFolder, qualifiedProcessName, qualifiedSymbolFor, resolveBpmnElementName, resolveCmmnElementName, RevisionQueue, selectProjectAnchor, setArchitecturalName, validateCmmnElementName, validateElementName, validateMemberName, validatePackageName, validateProcessName, validateQualifiedProcessName } from '../src/core.js';

describe('documentation paths', () => {
  it('maps diagrams and elements to deterministic Markdown', () => {
    expect(documentationPath('main.cmmn')).toBe('main.md');
    expect(documentationPath('order/main.bpmn', 'Task_1')).toBe('order/docs/Task_1.md');
    expect(documentationPath('cybling/sdk/main.cmmn', 'PlanItem_1')).toBe('cybling/sdk/docs/PlanItem_1.md');
  });
});

describe('composition identity', () => {
  it('uses folder names and paths instead of repeated base filenames', () => {
    expect(compositionIdentity('main.cmmn')).toEqual({ folder: '', name: 'Main', displayPath: 'schematics' });
    expect(compositionIdentity('sales/order/main.bpmn')).toEqual({
      folder: 'sales/order',
      name: 'Order',
      displayPath: 'schematics/sales/order',
    });
    expect(compositionIdentity('cybling/sdk/main.cmmn')).toEqual({ folder: 'cybling/sdk', name: 'Sdk', displayPath: 'schematics/cybling/sdk' });
  });
  it('builds navigable breadcrumbs from Main through the current folder', () => {
    expect(compositionBreadcrumbs('birth/main.bpmn')).toEqual([
      { name: 'Main', diagramPath: 'main.cmmn' },
      { name: 'Birth', diagramPath: 'birth/main.bpmn' },
    ]);
    expect(compositionBreadcrumbs('people/early-childhood/main.bpmn').map((item) => item.name)).toEqual([
      'Main', 'People', 'Early Childhood',
    ]);
  });
});

describe('composition paths', () => {
  it('normalizes call activity paths and rejects traversal', () => {
    expect(normalizeCompositionPath('schematics/sales/quote/')).toBe('sales/quote');
    expect(() => normalizeCompositionPath('../private')).toThrow();
  });
  it('retains legacy slug normalization only for migration', () => {
    expect(compositionSlug(' Birth ')).toBe('birth');
    expect(compositionSlug('Order Fulfillment')).toBe('order-fulfillment');
    expect(() => compositionSlug('---')).toThrow();
  });
});

describe('architectural names', () => {
  it('round-trips extension-backed Name independently from ID and Label', () => {
    const business = { id: 'Activity_1', name: 'Select and outfit subscription' };
    setArchitecturalName(business, 'cybling.subscription.SelectAndOutfit#selectPlan');
    expect(architecturalName(business)).toBe('cybling.subscription.SelectAndOutfit#selectPlan');
    expect(business.id).toBe('Activity_1');
    expect(business.name).toBe('Select and outfit subscription');
  });

  it('validates package, process, and member Java-style forms with actionable errors', () => {
    expect(validatePackageName('cybling.subscription')).toBe('cybling.subscription');
    expect(validateProcessName('SelectAndOutfit')).toBe('SelectAndOutfit');
    expect(validateMemberName('processSubscription2')).toBe('processSubscription2');
    expect(() => validatePackageName('Cybling.subscription')).toThrow(/lowerCamelCase/);
    expect(() => validateProcessName('selectAndOutfit')).toThrow(/UpperCamelCase/);
    expect(() => validateMemberName('ProcessSubscription')).toThrow(/lowerCamelCase/);
    expect(collisionKey('ConfigureSubscription')).toBe(collisionKey('configuresubscription'));
  });

  it('retains complete process and member Name support', () => {
    expect(qualifiedProcessName('cybling.subscription', 'SelectAndOutfit')).toBe('cybling.subscription.SelectAndOutfit');
    expect(qualifiedMemberName('cybling.CyblingLifecycle', 'init')).toBe('cybling.CyblingLifecycle#init');
    expect(qualifiedMemberName('subscription.SubscriptionLifecycle', 'init')).toBe('subscription.SubscriptionLifecycle#init');
    const process = { businessObject: { $type: 'bpmn:CallActivity', name: 'Friendly label', architecturalName: 'cybling.subscription.SelectAndOutfit' } };
    expect(qualifiedSymbolFor(process)).toBe('cybling.subscription.SelectAndOutfit');
    expect(compositionPathFor(process, 'main.bpmn')).toBe('cybling/subscription/SelectAndOutfit');
    const member = { $type: 'bpmn:Task', architecturalName: 'cybling.subscription.SelectAndOutfit#outfitCybling' };
    expect(qualifiedSymbolFor(member, { diagramPath: 'cybling/subscription/SelectAndOutfit/main.bpmn' })).toBe('cybling.subscription.SelectAndOutfit#outfitCybling');
    process.businessObject.calledElement = 'shared.SharedProcess';
    expect(qualifiedSymbolFor(process)).toBe('cybling.subscription.SelectAndOutfit');
    expect(validateElementName('cybling.subscription.SelectAndOutfit#paymentApproved')).toBe('cybling.subscription.SelectAndOutfit#paymentApproved');
  });

  it('resolves short BPMN node and edge Names against the parent process', () => {
    const diagramPath = 'cybling/sdk/Birth/main.bpmn';
    expect(owningPackageName(diagramPath)).toBe('cybling.sdk');
    expect(resolveBpmnElementName('prepareIdentity', { diagramPath })).toBe('cybling.sdk.Birth#prepareIdentity');
    expect(resolveBpmnElementName('PrepareIdentity', { diagramPath, reusable: true })).toBe('cybling.sdk.PrepareIdentity');
    expect(resolveBpmnElementName('shared.identity.PrepareIdentity', { diagramPath, reusable: true })).toBe('shared.identity.PrepareIdentity');
    expect(resolveBpmnElementName('shared.Identity#prepare', { diagramPath })).toBe('shared.Identity#prepare');
    expect(() => resolveBpmnElementName('prepareIdentity', { diagramPath: 'main.bpmn' })).toThrow(/named parent/);
  });

  it('resolves short CMMN node, edge, and Process Task Names against the package anchor', () => {
    expect(resolveCmmnElementName('birthNeed', { packageName: 'cybling.sdk' })).toBe('cybling.sdk#birthNeed');
    expect(resolveCmmnElementName('Birth', { packageName: 'cybling.sdk', reusable: true })).toBe('cybling.sdk.Birth');
    expect(resolveCmmnElementName('wallet.Onboard', { packageName: 'cybling.sdk', reusable: true })).toBe('wallet.Onboard');
    expect(resolveCmmnElementName('shared#need', { packageName: 'cybling.sdk' })).toBe('shared#need');
  });

  it('maps qualified process Names to confined portable folders in both directions', () => {
    const qualified = 'cybling.subscription.SelectAndOutfit';
    expect(compositionFolderForQualifiedName(qualified)).toBe('cybling/subscription/SelectAndOutfit');
    expect(diagramPathForQualifiedName(qualified)).toBe('cybling/subscription/SelectAndOutfit/main.bpmn');
    expect(qualifiedNameForCompositionFolder('cybling/subscription/SelectAndOutfit')).toBe(qualified);
    expect(qualifiedNameForCompositionFolder('cybling/subscription/SelectAndOutfit/main.bpmn')).toBe(qualified);
    expect(validateQualifiedProcessName(qualified)).toBe(qualified);
    expect(() => compositionFolderForQualifiedName('../private.Process')).toThrow();
    expect(() => qualifiedNameForCompositionFolder('cybling/../SelectAndOutfit')).toThrow();
  });

  it('maps sparse CMMN package anchors independently from descendant BPMN processes', () => {
    expect(cmmnFolderForPackageName('cybling.sdk')).toBe('cybling/sdk');
    expect(cmmnPathForPackageName('cybling.sdk')).toBe('cybling/sdk/main.cmmn');
    expect(packageNameForCmmnPath('cybling/sdk/main.cmmn')).toBe('cybling.sdk');
    expect(diagramPathForQualifiedName('cybling.sdk.Birth')).toBe('cybling/sdk/Birth/main.bpmn');
    expect(validateCmmnElementName('cybling#businessNeed')).toBe('cybling#businessNeed');
    expect(() => validateCmmnElementName('cybling')).toThrow(/#memberName/);
    expect(diagramKind('cybling/main.cmmn')).toBe('cmmn');
    expect(diagramKind('cybling/sdk/Birth/main.bpmn')).toBe('bpmn');
  });
});

describe('RevisionQueue', () => {
  it('serializes writes and only reports the latest revision saved', async () => {
    const states = [];
    const writer = vi.fn(async () => {});
    const queue = new RevisionQueue(writer, (state) => states.push(state));
    await Promise.all([queue.enqueue('main.md', 'one'), queue.enqueue('main.md', 'two')]);
    expect(writer.mock.calls.map((call) => call.slice(1))).toEqual([['one', 1], ['two', 2]]);
    expect(states.at(-1)).toBe('saved');
  });
  it('allows lifecycle cleanup to await the latest path write', async () => {
    let release;
    const writer = vi.fn(() => new Promise((resolve) => { release = resolve; }));
    const queue = new RevisionQueue(writer);
    queue.enqueue('main.bpmn', '<xml />');
    const waiting = queue.waitFor('main.bpmn');
    await vi.waitFor(() => expect(writer).toHaveBeenCalledOnce());
    release();
    await waiting;
  });
});

describe('workspace UI state', () => {
  it('selects CMMN as the sole root while retaining a legacy BPMN fallback', () => {
    expect(selectProjectAnchor(['main.cmmn', 'cybling/sdk/Birth/main.bpmn'])).toBe('main.cmmn');
    expect(selectProjectAnchor(['main.bpmn'])).toBe('main.bpmn');
    expect(() => selectProjectAnchor(['main.cmmn', 'main.bpmn'])).toThrow(/Competing project anchors/);
    expect(() => selectProjectAnchor([])).toThrow(/anchor missing/);
    expect(isRootDiagram('main.cmmn')).toBe(true);
    expect(isRootDiagram('main.bpmn')).toBe(true);
    expect(isRootDiagram('orders/main.bpmn')).toBe(false);
  });

  it('normalizes node statuses and publishes the requested color/meaning registry', () => {
    expect(normalizeNodeStatus('locked')).toBe('locked');
    expect(normalizeNodeStatus('unsupported')).toBe('open');
    expect(NODE_STATUSES.new.color).toBe('#62c88a');
    expect(NODE_STATUSES.locked.color).toBe('#a7adb7');
    expect(NODE_STATUSES.modify.color).toBe('#f2a65a');
    expect(NODE_STATUSES.open.color).toBe('#ffffff');
    expect(NODE_STATUSES.locked.meaning).toContain('Do not change');
  });

  it('builds project-aware titles with a safe fallback and Unicode text', () => {
    expect(projectDocumentTitle('order-service')).toBe('Software Schematic - order-service');
    expect(projectDocumentTitle(' Café Platform ')).toBe('Software Schematic - Café Platform');
    expect(projectDocumentTitle('')).toBe('Software Schematic');
  });
});
