import BpmnModeler from 'bpmn-js/lib/Modeler';
import sswBpmnModdle from './ssw-moddle.json';
import sswCmmnModdle from './ssw-cmmn-moddle.json';
import { architecturalName, DIAGRAM_PACKAGE_ATTRIBUTE, diagramKind, packageNameForCmmnPath, resolveBpmnElementName, validatePackageName } from './core.js';

export type DiagramKind = 'bpmn' | 'cmmn';

export interface DiagramAdapter {
  kind: DiagramKind;
  modeler: any;
  importXML(xml: string): Promise<any>;
  saveXML(options?: Record<string, unknown>): Promise<{ xml: string }>;
  destroy(): void;
  businessType(element: any): string;
  displayType(element: any): string;
  elementId(element: any): string;
  elementLabel(element: any): string;
  isSelectable(element: any): boolean;
  isStatusEligible(element: any): boolean;
  isComposable(element: any): boolean;
  diagramName(path: string): string | null;
  elementName(element: any): string;
  updateId(element: any, id: string): void;
  updateLabel(element: any, label: string): void;
  updateName(element: any, name: string): void;
  normalizedElements(statuses: Map<string, string>): { nodes: any[]; flows: any[] };
}

function commonType(element: any): string {
  return element?.businessObject?.$type || '';
}

function cmmnSemantic(element: any): any {
  const business = element?.businessObject || element;
  return business?.$type === 'cmmndi:CMMNEdge' && business.cmmnElementRef ? business.cmmnElementRef : business;
}

function cmmnDefinition(element: any): any {
  const business = cmmnSemantic(element);
  return business?.$type === 'cmmn:PlanItem' ? business.definitionRef : business;
}

export function cmmnLabelFor(element: any): string {
  return cmmnSemantic(element)?.name || cmmnDefinition(element)?.name || '';
}

export function updateCmmnProperties(modeler: any, element: any, properties: Record<string, unknown>): void {
  const semantic = cmmnSemantic(element);
  const modeling = modeler.get('modeling');
  if (element?.businessObject && semantic !== element.businessObject) modeling.updateProperties(semantic, properties, element);
  else modeling.updateProperties(element, properties);
}

function cmmnPackageName(modeler: any, diagramPath: string): string {
  const definitions = modeler.getDefinitions?.();
  const declared = definitions?.[DIAGRAM_PACKAGE_ATTRIBUTE] || definitions?.$attrs?.['ssw:packageName'];
  return declared ? validatePackageName(declared) : packageNameForCmmnPath(diagramPath);
}

function callbackImport(modeler: any, xml: string): Promise<{ warnings: any[] }> {
  return new Promise((resolve, reject) => modeler.importXML(xml, (error: Error | null, warnings: any[] = []) => error ? reject(error) : resolve({ warnings })));
}

function callbackSave(modeler: any, options: Record<string, unknown>): Promise<{ xml: string }> {
  return new Promise((resolve, reject) => modeler.saveXML(options, (error: Error | null, xml: string) => error ? reject(error) : resolve({ xml })));
}

function normalized(modeler: any, statuses: Map<string, string>, kind: DiagramKind) {
  const elements = modeler.get('elementRegistry').getAll();
  const ignored = kind === 'bpmn'
    ? new Set(['bpmn:Process', 'bpmn:Collaboration', 'bpmn:Definitions'])
    : new Set(['cmmn:Definitions', 'cmmn:Case', 'cmmn:CasePlanModel']);
  const records = elements.filter((item: any) => item.businessObject?.$type && !item.labelTarget && !ignored.has(item.businessObject.$type));
  const flows = records.filter((item: any) => Boolean(item.waypoints)).map((item: any) => ({
    id: item.id,
    type: kind === 'cmmn' ? cmmnSemantic(item)?.$type || item.businessObject.$type : item.businessObject.$type,
    name: architecturalName(kind === 'cmmn' ? cmmnSemantic(item) : item.businessObject) || null,
    label: (kind === 'cmmn' ? cmmnSemantic(item)?.name : item.businessObject.name) || '',
    status: statuses.get(item.id) || 'open',
    source: item.source?.id || item.businessObject.sourceRef?.id || null,
    target: item.target?.id || item.businessObject.targetRef?.id || null,
  })).sort((a: any, b: any) => a.id.localeCompare(b.id));
  const nodes = records.filter((item: any) => !item.waypoints).map((item: any) => ({
    id: item.id,
    type: kind === 'cmmn' ? cmmnDefinition(item)?.$type || item.businessObject.$type : item.businessObject.$type,
    name: architecturalName(item.businessObject) || architecturalName(cmmnDefinition(item)) || null,
    label: item.businessObject.name || cmmnDefinition(item)?.name || '',
    status: statuses.get(item.id) || 'open',
  })).sort((a: any, b: any) => a.id.localeCompare(b.id));
  return { nodes, flows };
}

export async function createDiagramAdapter(path: string, container: HTMLElement, modules: { bpmn?: any[]; cmmn?: any[] } = {}): Promise<DiagramAdapter> {
  const kind = diagramKind(path);
  if (kind === 'bpmn') {
    const modeler = new BpmnModeler({ container, additionalModules: modules.bpmn || [], moddleExtensions: { ssw: sswBpmnModdle } });
    return {
      kind,
      modeler,
      importXML: (xml) => modeler.importXML(xml),
      saveXML: (options = { format: true }) => modeler.saveXML(options),
      destroy: () => modeler.destroy(),
      businessType: commonType,
      displayType: (element) => commonType(element).replace('bpmn:', '') || 'Diagram',
      elementId: (element) => element?.businessObject?.id || element?.id || '',
      elementLabel: (element) => element?.businessObject?.name || '',
      isSelectable: (element) => Boolean(commonType(element) && !element?.labelTarget),
      isStatusEligible: (element) => Boolean(element && !element.labelTarget && commonType(element) && !['bpmn:Process', 'bpmn:Collaboration', 'bpmn:Definitions'].includes(commonType(element))),
      isComposable: (element) => ['bpmn:CallActivity', 'bpmn:SubProcess'].includes(commonType(element)),
      diagramName: () => null,
      elementName: (element) => architecturalName(element?.businessObject),
      updateId: (element, id) => modeler.get('modeling').updateProperties(element, { id }),
      updateLabel: (element, name) => modeler.get('modeling').updateProperties(element, { name }),
      updateName: (element, name) => {
        const reusable = ['bpmn:CallActivity', 'bpmn:SubProcess'].includes(commonType(element));
        modeler.get('modeling').updateProperties(element, {
          architecturalName: name,
          ...(reusable ? { calledElement: resolveBpmnElementName(name, { diagramPath: path, reusable: true }) } : {}),
        });
      },
      normalizedElements: (statuses) => normalized(modeler, statuses, kind),
    };
  }

  await Promise.all([
    import('cmmn-js/dist/assets/diagram-js.css'),
    import('cmmn-font/dist/css/cmmn.css'),
  ]);
  const imported: any = await import('cmmn-js/lib/Modeler');
  const CmmnModeler = imported.default || imported;
  const modeler = new CmmnModeler({ container, additionalModules: modules.cmmn || [], moddleExtensions: { ssw: sswCmmnModdle } });
  return {
    kind,
    modeler,
    importXML: (xml) => callbackImport(modeler, xml),
    saveXML: (options = { format: true }) => callbackSave(modeler, options),
    destroy: () => modeler.destroy(),
    businessType: (element) => cmmnDefinition(element)?.$type || commonType(element),
    displayType: (element) => (cmmnDefinition(element)?.$type || commonType(element)).replace('cmmn:', '') || 'Diagram',
    elementId: (element) => element?.businessObject?.id || element?.id || '',
    elementLabel: cmmnLabelFor,
    isSelectable: (element) => Boolean(commonType(element) && !element?.labelTarget),
    isStatusEligible: (element) => Boolean(element && !element.labelTarget && commonType(element) && !['cmmn:Definitions', 'cmmn:Case', 'cmmn:CasePlanModel'].includes(commonType(element))),
    isComposable: (element) => cmmnDefinition(element)?.$type === 'cmmn:ProcessTask',
    diagramName: (diagramPath) => cmmnPackageName(modeler, diagramPath),
    elementName: (element) => architecturalName(cmmnSemantic(element)) || architecturalName(cmmnDefinition(element)),
    updateId: (element, id) => modeler.get('modeling').updateProperties(element, { id }),
    updateLabel: (element, name) => updateCmmnProperties(modeler, element, { name }),
    updateName: (element, name) => updateCmmnProperties(modeler, element, { architecturalName: name }),
    normalizedElements: (statuses) => normalized(modeler, statuses, kind),
  };
}

export function cmmnDefinitionFor(element: any) {
  return cmmnDefinition(element);
}

export function cmmnMetadataFor(element: any) {
  return cmmnSemantic(element);
}
