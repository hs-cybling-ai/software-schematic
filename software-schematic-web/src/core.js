export function normalizeCompositionPath(value) {
  const normalized = value.trim().replaceAll('\\', '/').replace(/^schematics\//, '').replace(/\/+$/, '');
  if (!normalized || normalized.startsWith('/') || normalized.split('/').some((part) => !part || part === '.' || part === '..')) {
    throw new Error('Use a relative composition folder such as order or sales/quote');
  }
  return normalized;
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
  const folder = diagramPath.endsWith('/main.bpmn')
    ? diagramPath.slice(0, -'/main.bpmn'.length)
    : diagramPath === 'main.bpmn' ? '' : diagramPath.replace(/\/[^/]+\.bpmn$/, '');
  return {
    folder,
    name: folder ? compositionDisplayName(folder.split('/').at(-1)) : 'Main',
    displayPath: folder ? `schematics/${folder}` : 'schematics',
  };
}

export function compositionDisplayName(folderName) {
  return folderName.split('-').filter(Boolean).map((word) => word.charAt(0).toUpperCase() + word.slice(1)).join(' ');
}

export function compositionBreadcrumbs(diagramPath) {
  const { folder } = compositionIdentity(diagramPath);
  const items = [{ name: 'Main', diagramPath: 'main.bpmn' }];
  if (!folder) return items;
  const segments = folder.split('/');
  segments.forEach((segment, index) => {
    items.push({
      name: compositionDisplayName(segment),
      diagramPath: `${segments.slice(0, index + 1).join('/')}/main.bpmn`,
    });
  });
  return items;
}

export const ROOT_DIAGRAM_PATH = 'main.bpmn';

export function isRootDiagram(path) {
  return path === ROOT_DIAGRAM_PATH;
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
  const folder = diagramPath.includes('/') ? diagramPath.slice(0, diagramPath.lastIndexOf('/')) : '';
  const business = element?.businessObject;
  if (!business) throw new Error('Select a pool, lane, or call activity');
  if (business.$type === 'bpmn:CallActivity') return normalizeCompositionPath(business.calledElement || '');
  if (business.$type === 'bpmn:SubProcess') return [folder, compositionSlug(business.name || '')].filter(Boolean).join('/');
  if (business.$type === 'bpmn:Participant') return [folder, business.id].filter(Boolean).join('/');
  if (business.$type === 'bpmn:Lane') {
    const participant = findParticipant(element);
    return [folder, participant?.businessObject?.id, business.id].filter(Boolean).join('/');
  }
  throw new Error('Only pools, lanes, subprocesses, and call activities have compositions');
}

function findParticipant(element) {
  let current = element.parent;
  while (current) {
    if (current.businessObject?.$type === 'bpmn:Participant') return current;
    current = current.parent;
  }
  return null;
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
