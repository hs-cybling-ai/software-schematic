import { NODE_STATUSES, normalizeNodeStatus } from './core.js';

export function isStatusEligible(tab, element) {
  return Boolean(tab?.adapter?.isStatusEligible(element));
}

export function statusFor(tab, element) {
  return normalizeNodeStatus(tab?.nodeStatuses.get(element?.id) || tab?.adapter?.elementStatus(element));
}

function statusElementLabel(tab, element) {
  const adapted = tab?.adapter?.elementLabel(element);
  if (adapted) return adapted;
  const business = element?.businessObject;
  return business?.name || business?.definitionRef?.name || business?.id || business?.$type?.replace(/^[^:]+:/, '') || 'Diagram node';
}

export function applyNodeStatus(tab, element) {
  if (!isStatusEligible(tab, element)) return;
  const canvas = tab.modeler.get('canvas');
  Object.keys(NODE_STATUSES).forEach((status) => canvas.removeMarker(element, `node-status-${status}`));
  const status = statusFor(tab, element);
  const definition = NODE_STATUSES[status];
  canvas.addMarker(element, `node-status-${status}`);
  const gfx = tab.modeler.get('elementRegistry').getGraphics(element);
  gfx?.setAttribute('aria-label', `${statusElementLabel(tab, element)}. ${definition.label}. ${definition.meaning}`);
}

export function hydrateNodeStatuses(tab) {
  if (!tab) return;
  tab.nodeStatuses.clear();
  tab.modeler.get('elementRegistry').getAll().forEach((element) => {
    if (!isStatusEligible(tab, element)) return;
    const status = normalizeNodeStatus(tab.adapter.elementStatus(element));
    if (status !== 'open') tab.nodeStatuses.set(element.id, status);
    applyNodeStatus(tab, element);
  });
}

export async function importTabXML(tab, xml) {
  const imported = await tab.adapter.importXML(xml);
  hydrateNodeStatuses(tab);
  return imported;
}
