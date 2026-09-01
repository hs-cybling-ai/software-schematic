export function normalizeCompositionPath(value) {
  const normalized = value.trim().replaceAll('\\', '/').replace(/^schematics\//, '').replace(/\/+$/, '');
  if (!normalized || normalized.startsWith('/') || normalized.split('/').some((part) => !part || part === '.' || part === '..')) {
    throw new Error('Use a relative composition folder such as order or sales/quote');
  }
  return normalized;
}

export const ARCHITECTURAL_NAME_ATTRIBUTE = 'architecturalName';
export const DIAGRAM_PROCESS_ATTRIBUTE = 'processName';
export const DIAGRAM_PACKAGE_ATTRIBUTE = 'packageName';

const LOWER_CAMEL = /^[a-z][A-Za-z0-9]*$/;
const UPPER_CAMEL = /^[A-Z][A-Za-z0-9]*$/;

export function validatePackageName(value) {
  const name = String(value || '').trim();
  if (!name || !name.split('.').every((segment) => LOWER_CAMEL.test(segment))) {
    throw new Error('Package Name must use dot-separated lowerCamelCase segments, for example cybling.subscription');
  }
  return name;
}

export function validateProcessName(value) {
  const name = String(value || '').trim();
  if (!UPPER_CAMEL.test(name)) throw new Error('Process Name must use UpperCamelCase, for example SelectAndOutfit');
  return name;
}

export function validateMemberName(value) {
  const name = String(value || '').trim();
  if (!LOWER_CAMEL.test(name)) throw new Error('Task or event Name must use lowerCamelCase, for example processSubscription');
  return name;
}

export function validateElementName(value) {
  const name = String(value || '').trim();
  const [processName, memberName, extra] = name.split('#');
  if (extra !== undefined) throw new Error('Name may contain only one # member separator');
  validateQualifiedProcessName(processName);
  if (memberName !== undefined) validateMemberName(memberName);
  return name;
}

export function validateCmmnElementName(value) {
  const name = String(value || '').trim();
  const [packageName, memberName, extra] = name.split('#');
  if (extra !== undefined) throw new Error('Name may contain only one # member separator');
  validatePackageName(packageName);
  if (memberName === undefined) throw new Error('CMMN node or connection Names require #memberName');
  validateMemberName(memberName);
  return name;
}

export function processNameFromElementName(value) {
  return validateElementName(value).split('#')[0];
}

export function memberNameFromElementName(value) {
  const name = validateElementName(value);
  const separator = name.indexOf('#');
  return separator < 0 ? null : name.slice(separator + 1);
}

export function collisionKey(value) {
  return String(value || '').normalize('NFC').toLocaleLowerCase('en-US');
}

export function architecturalName(businessObject) {
  return businessObject?.[ARCHITECTURAL_NAME_ATTRIBUTE]
    || businessObject?.$attrs?.['ssw:architecturalName']
    || businessObject?.$attrs?.['ssw:name']
    || '';
}

export function setArchitecturalName(businessObject, value) {
  if (!businessObject) throw new Error('A BPMN business object is required');
  const name = String(value || '').trim();
  businessObject[ARCHITECTURAL_NAME_ATTRIBUTE] = name || undefined;
  return name;
}

export function qualifiedProcessName(packageName, processName) {
  return `${validatePackageName(packageName)}.${validateProcessName(processName)}`;
}

export function qualifiedMemberName(owningProcessName, memberName) {
  const owner = validateQualifiedProcessName(owningProcessName);
  return `${owner}#${validateMemberName(memberName)}`;
}

export function validateQualifiedProcessName(value) {
  const name = String(value || '').trim();
  const segments = name.split('.');
  if (segments.length < 2) throw new Error('Qualified process Name must include a package and process, for example cybling.SelectAndOutfit');
  const processName = segments.pop();
  return qualifiedProcessName(segments.join('.'), processName);
}

export function compositionFolderForQualifiedName(value) {
  return validateQualifiedProcessName(processNameFromElementName(value)).replaceAll('.', '/');
}

export function qualifiedNameForCompositionFolder(value) {
  const folder = normalizeCompositionPath(value).replace(/\/main\.bpmn$/, '');
  return validateQualifiedProcessName(folder.split('/').join('.'));
}

export function diagramPathForQualifiedName(value) {
  return `${compositionFolderForQualifiedName(value)}/main.bpmn`;
}

export function cmmnFolderForPackageName(value) {
  return validatePackageName(value).replaceAll('.', '/');
}

export function cmmnPathForPackageName(value) {
  return `${cmmnFolderForPackageName(value)}/main.cmmn`;
}

export function packageNameForCmmnPath(value) {
  const path = normalizeCompositionPath(value).replace(/\/main\.cmmn$/i, '');
  if (!path || path === 'main.cmmn') throw new Error('A named CMMN package must be beneath schematics');
  return validatePackageName(path.split('/').join('.'));
}

export function owningProcessName(diagramPath, explicitName = '') {
  if (explicitName) return validateQualifiedProcessName(explicitName);
  if (isRootDiagram(diagramPath)) return null;
  return qualifiedNameForCompositionFolder(compositionIdentity(diagramPath).folder);
}

export function owningPackageName(diagramPath, explicitProcessName = '') {
  const processName = owningProcessName(diagramPath, explicitProcessName);
  if (!processName) return null;
  return processName.split('.').slice(0, -1).join('.');
}

export function resolveBpmnElementName(value, { diagramPath = ROOT_DIAGRAM_PATH, diagramProcessName = '', reusable = false } = {}) {
  const name = String(value || '').trim();
  if (name.includes('#')) {
    const qualified = validateElementName(name);
    if (reusable) throw new Error('A reusable process Name cannot use a # member separator');
    return qualified;
  }
  if (name.includes('.')) {
    const qualified = validateQualifiedProcessName(name);
    if (!reusable) throw new Error('A node or edge Name must be a local member or use package.Process#memberName');
    return qualified;
  }
  const parentProcess = owningProcessName(diagramPath, diagramProcessName);
  if (!parentProcess) throw new Error('A short Name requires a named parent diagram; use a fully qualified Name here');
  if (!reusable) return qualifiedMemberName(parentProcess, name);
  return qualifiedProcessName(owningPackageName(diagramPath, diagramProcessName), name);
}

export function resolveCmmnElementName(value, { packageName = '', reusable = false } = {}) {
  const name = String(value || '').trim();
  const parentPackage = validatePackageName(packageName);
  if (reusable) return name.includes('.') ? validateQualifiedProcessName(name) : qualifiedProcessName(parentPackage, name);
  return name.includes('#') ? validateCmmnElementName(name) : `${parentPackage}#${validateMemberName(name)}`;
}

export function qualifiedSymbolFor(element, { diagramPath = ROOT_DIAGRAM_PATH, diagramProcessName = '', packageName = '' } = {}) {
  const business = element?.businessObject || element;
  const name = architecturalName(business);
  if (!business || !name) return null;
  return resolveBpmnElementName(name, {
    diagramPath,
    diagramProcessName,
    reusable: ['bpmn:CallActivity', 'bpmn:SubProcess'].includes(business.$type),
  });
}

export function compositionSlug(value) {
  const slug = value.trim().toLowerCase()
    .normalize('NFKD').replace(/[\u0300-\u036f]/g, '')
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
  if (!slug) throw new Error('Give the subprocess a label before opening its composition');
  return slug;
}

export function documentationPath(diagramPath, elementId) {
  const folder = diagramPath.includes('/') ? diagramPath.slice(0, diagramPath.lastIndexOf('/')) : '';
  return elementId ? `${folder ? `${folder}/` : ''}docs/${elementId}.md` : `${folder ? `${folder}/` : ''}main.md`;
}

export function compositionIdentity(diagramPath) {
  const folder = /\/main\.(?:bpmn|cmmn)$/i.test(diagramPath)
    ? diagramPath.replace(/\/main\.(?:bpmn|cmmn)$/i, '')
    : /^main\.(?:bpmn|cmmn)$/i.test(diagramPath) ? '' : diagramPath.replace(/\/[^/]+\.(?:bpmn|cmmn)$/i, '');
  return {
    folder,
    name: folder ? compositionDisplayName(folder.split('/').at(-1)) : 'Main',
    displayPath: folder ? `schematics/${folder}` : 'schematics',
  };
}

export function compositionDisplayName(folderName) {
  return folderName.split('-').filter(Boolean).map((word) => word.charAt(0).toUpperCase() + word.slice(1)).join(' ');
}

export function compositionBreadcrumbs(diagramPath, rootDiagramPath = ROOT_DIAGRAM_PATH) {
  const { folder } = compositionIdentity(diagramPath);
  const items = [{ name: 'Main', diagramPath: rootDiagramPath }];
  if (!folder) return items;
  const segments = folder.split('/');
  const extension = diagramPath.toLowerCase().endsWith('.cmmn') ? 'cmmn' : 'bpmn';
  segments.forEach((segment, index) => {
    items.push({
      name: compositionDisplayName(segment),
      diagramPath: `${segments.slice(0, index + 1).join('/')}/main.${extension}`,
    });
  });
  return items;
}

export const ROOT_DIAGRAM_PATH = 'main.cmmn';
export const LEGACY_ROOT_DIAGRAM_PATH = 'main.bpmn';

export function isRootDiagram(path) {
  return path === ROOT_DIAGRAM_PATH || path === LEGACY_ROOT_DIAGRAM_PATH;
}

export function selectProjectAnchor(diagrams) {
  const paths = new Set(Array.isArray(diagrams) ? diagrams : []);
  const hasCmmn = paths.has(ROOT_DIAGRAM_PATH);
  const hasBpmn = paths.has(LEGACY_ROOT_DIAGRAM_PATH);
  if (hasCmmn && hasBpmn) throw new Error('Competing project anchors: remove main.bpmn after representing its process on main.cmmn');
  if (hasCmmn) return ROOT_DIAGRAM_PATH;
  if (hasBpmn) return LEGACY_ROOT_DIAGRAM_PATH;
  throw new Error('Project anchor missing: expected schematics/main.cmmn');
}

export function diagramKind(path) {
  if (String(path).toLowerCase().endsWith('.cmmn')) return 'cmmn';
  if (String(path).toLowerCase().endsWith('.bpmn')) return 'bpmn';
  throw new Error(`Unsupported diagram type: ${path}`);
}

export const NODE_STATUSES = Object.freeze({
  open: Object.freeze({ label: 'Open', color: '#ffffff', meaning: 'No LLM authoring hint.' }),
  new: Object.freeze({ label: 'New', color: '#62c88a', meaning: 'Create the represented work or content.' }),
  locked: Object.freeze({ label: 'Locked', color: '#a7adb7', meaning: 'Do not change this node.' }),
  modify: Object.freeze({ label: 'Modify', color: '#f2a65a', meaning: 'Changes are allowed and expected.' }),
});

export function normalizeNodeStatus(status) {
  return Object.hasOwn(NODE_STATUSES, status) ? status : 'open';
}

export function projectDocumentTitle(projectName) {
  const name = typeof projectName === 'string' ? projectName.trim() : '';
  return name ? `Software Schematic - ${name}` : 'Software Schematic';
}

export function compositionPathFor(element, diagramPath) {
  const business = element?.businessObject;
  if (!business) throw new Error('Select a reusable process');
  if (!['bpmn:CallActivity', 'bpmn:SubProcess'].includes(business.$type)) throw new Error('Only reusable processes have compositions');
  const name = qualifiedSymbolFor(element, { diagramPath });
  if (memberNameFromElementName(name)) throw new Error('Only a process Name can open a composition');
  return compositionFolderForQualifiedName(name);
}

export class RevisionQueue {
  constructor(writer, onState = () => {}) {
    this.writer = writer;
    this.onState = onState;
    this.revisions = new Map();
    this.chains = new Map();
  }

  enqueue(path, content) {
    const revision = (this.revisions.get(path) || 0) + 1;
    this.revisions.set(path, revision);
    this.onState('pending');
    const prior = this.chains.get(path) || Promise.resolve();
    const next = prior.catch(() => {}).then(() => this.writer(path, content, revision)).then(() => {
      if (this.revisions.get(path) === revision) this.onState('saved');
    }).catch((error) => {
      if (this.revisions.get(path) === revision) this.onState('failed', error);
      throw error;
    });
    this.chains.set(path, next);
    return next;
  }

  waitFor(path) {
    return this.chains.get(path) || Promise.resolve();
  }
}
