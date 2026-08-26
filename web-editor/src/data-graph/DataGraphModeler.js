import Diagram from 'diagram-js/lib/Diagram.js';
import BaseRenderer from 'diagram-js/lib/draw/BaseRenderer.js';
import BaseLayouter from 'diagram-js/lib/layout/BaseLayouter.js';
import RuleProvider from 'diagram-js/lib/features/rules/RuleProvider.js';
import CommandInterceptor from 'diagram-js/lib/command/CommandInterceptor.js';
import ModelingModule from 'diagram-js/lib/features/modeling/index.js';
import MoveModule from 'diagram-js/lib/features/move/index.js';
import CreateModule from 'diagram-js/lib/features/create/index.js';
import ConnectModule from 'diagram-js/lib/features/connect/index.js';
import BendpointsModule from 'diagram-js/lib/features/bendpoints/index.js';
import PaletteModule from 'diagram-js/lib/features/palette/index.js';
import ContextPadModule from 'diagram-js/lib/features/context-pad/index.js';
import PopupMenuModule from 'diagram-js/lib/features/popup-menu/index.js';
import KeyboardModule from 'diagram-js/lib/features/keyboard/index.js';
import KeyboardMoveModule from 'diagram-js/lib/navigation/keyboard-move/index.js';
import MoveCanvasModule from 'diagram-js/lib/navigation/movecanvas/index.js';
import ZoomScrollModule from 'diagram-js/lib/navigation/zoomscroll/index.js';
import EditorActionsModule from 'diagram-js/lib/features/editor-actions/index.js';
import HandToolModule from 'diagram-js/lib/features/hand-tool/index.js';
import LassoToolModule from 'diagram-js/lib/features/lasso-tool/index.js';
import GlobalConnectModule from 'diagram-js/lib/features/global-connect/index.js';
import AutoScrollModule from 'diagram-js/lib/features/auto-scroll/index.js';
import ConnectionPreviewModule from 'diagram-js/lib/features/connection-preview/index.js';
import GridSnappingModule from 'diagram-js/lib/features/grid-snapping/index.js';
import KeyboardMoveSelectionModule from 'diagram-js/lib/features/keyboard-move-selection/index.js';
import ClipboardModule from 'diagram-js/lib/features/clipboard/index.js';
import CopyPasteModule from 'diagram-js/lib/features/copy-paste/index.js';
import SpaceToolModule from 'diagram-js/lib/features/space-tool/index.js';
import AlignElementsModule from 'diagram-js/lib/features/align-elements/index.js';
import DistributeElementsModule from 'diagram-js/lib/features/distribute-elements/index.js';
import HoverFixModule from 'diagram-js/lib/features/hover-fix/index.js';
import LabelSupportModule from 'diagram-js/lib/features/label-support/index.js';
import DirectEditingModule from 'diagram-js-direct-editing';
import { effectiveConnections, midpointOnPath, parseDocument, serializeDocument, starterDocument } from './DataGraphDocument.js';
import { TYPE_REGISTRY, subtypeField, typeDefinition, withDefaultSubtype } from './DataGraphTypes.js';
import { contextDraft } from './MarkdownContext.js';

const SVG_NS = 'http://www.w3.org/2000/svg';
export const OBJECT_SIZE = 100;
export const COMPACT_SIZE = 50;
const LABEL_WIDTH = 140;
const LABEL_HEIGHT = 36;
const CONTROL_LABEL_GAP = 8;
const TYPE_MENU_ID = 'data-graph-types';
const svg = (name, attributes = {}) => {
  const element = document.createElementNS(SVG_NS, name);
  Object.entries(attributes).forEach(([key, value]) => element.setAttribute(key, String(value)));
  return element;
};
const typeOf = element => element.businessObject?.type ?? element.type;
export const connectionNotation = connection => {
  if (typeOf(connection) === 'objectLink') return 'object-link';
  if (typeOf(connection) === 'modifier') return 'mutation';
  const targetType = typeOf(connection.target);
  return targetType === 'property' ? 'property' : 'relationship';
};
const sizeFor = type => type === 'objectNode' ? OBJECT_SIZE : COMPACT_SIZE;
const center = element => element.waypoints ? midpointOnPath(element.waypoints) : ({ x: element.x + element.width / 2, y: element.y + element.height / 2 });
const idSequence = prefix => `${prefix}-${crypto.randomUUID?.() ?? Math.random().toString(36).slice(2)}`;
const businessObjectFor = (type, values = {}) => withDefaultSubtype(type, { type, label: type === 'objectNode' ? 'Object' : type, ...values });
const dock = (source, target) => {
  const from = center(source); const to = center(target); const dx = to.x - from.x; const dy = to.y - from.y; const length = Math.hypot(dx, dy) || 1;
  return source.waypoints ? from : { x: from.x + dx / length * source.width / 2, y: from.y + dy / length * source.height / 2 };
};
const overlaps = (position, size, shape, gap = 24) => Math.abs(position.x - center(shape).x) < (size + shape.width) / 2 + gap && Math.abs(position.y - center(shape).y) < (size + shape.height) / 2 + gap;
const collisionFreePosition = (position, size, elementRegistry, excludedId = null) => {
  const shapes = elementRegistry.getAll().filter(element => element.id !== excludedId && element.width && element.height && typeOf(element) !== 'edgeLabel');
  const available = candidate => !shapes.some(shape => overlaps(candidate, size, shape));
  if (available(position)) return position;
  for (const radius of [48, 72, 96, 120, 144, 168, 192, 240]) for (let step = 0; step < 16; step += 1) {
    const angle = step * Math.PI / 8; const candidate = { x: position.x + Math.cos(angle) * radius, y: position.y + Math.sin(angle) * radius };
    if (available(candidate)) return candidate;
  }
  return position;
};
const radialPlacement = (source, elementRegistry, size = OBJECT_SIZE) => {
  const origin = center(source); const shapes = elementRegistry.getAll().filter(element => element.width && element.height && typeOf(element) !== 'edgeLabel');
  for (const radius of [220, 300, 380]) for (let step = 0; step < 12; step += 1) {
    const angle = step * Math.PI / 6; const candidate = { x: origin.x + Math.cos(angle) * radius, y: origin.y + Math.sin(angle) * radius };
    if (!shapes.some(shape => overlaps(candidate, size, shape))) return candidate;
  }
  return { x: origin.x + 460, y: origin.y };
};
const createEdgeLabel = (modeling, labelTarget, position, edgeId, label, parent) => modeling.createLabel(labelTarget, position, {
  id: `${edgeId}-label`, type: 'label', width: LABEL_WIDTH, height: LABEL_HEIGHT, businessObject: { type: 'edgeLabel', edgeId, label }
}, parent);
const defaultControlLabelPosition = control => ({ x: center(control).x, y: control.y - CONTROL_LABEL_GAP - LABEL_HEIGHT / 2 });
const createLogicalRelationship = (modeling, elementFactory, source, target, parent, edgeNodeId = idSequence('edge'), label = 'RELATES_TO', terminalPosition = null) => {
  if (typeOf(source) !== 'objectNode' || (target && typeOf(target) !== 'objectNode')) throw new Error('Edge nodes require an object-node domain and object-node ranges');
  const sourceCenter = center(source); const targetCenter = target ? center(target) : (terminalPosition ?? { x: sourceCenter.x + 180, y: sourceCenter.y });
  const control = elementFactory.createShape({ id: edgeNodeId, type: 'dg:edgeNode', width: COMPACT_SIZE, height: COMPACT_SIZE, businessObject: businessObjectFor('edgeNode', { label }) });
  const controlPosition = target ? { x: (sourceCenter.x + targetCenter.x) / 2, y: (sourceCenter.y + targetCenter.y) / 2 } : targetCenter;
  modeling.createShape(control, controlPosition, parent);
  const first = modeling.connect(source, control, { id: `${edgeNodeId}_domain`, type: 'dg:domain', businessObject: { type: 'domain' } });
  const segments = [first];
  if (target) segments.push(modeling.connect(control, target, { id: `${edgeNodeId}_range_${target.id}`, type: 'dg:range', businessObject: { type: 'range' } }));
  const labelShape = createEdgeLabel(modeling, control, defaultControlLabelPosition(control), edgeNodeId, label, parent);
  return { id: edgeNodeId, control, label: labelShape, segments };
};

const addMembership = (modeling, control, target) => {
  if (typeOf(control) !== 'edgeNode' || typeOf(target) !== 'objectNode') throw new Error('Collection targets must be object nodes');
  const targets = (control.outgoing ?? []).filter(connection => typeOf(connection) === 'range');
  if (targets.some(connection => connection.target === target)) throw new Error('Relationship already targets this object');
  if (control.businessObject.collectionType === 'scalar' && targets.length) throw new Error('Scalar relationships allow at most one target');
  return modeling.connect(control, target, { id: `${control.id}_range_${target.id}`, type: 'dg:range', businessObject: { type: 'range' } });
};

function DataGraphLayouter() { BaseLayouter.call(this); }
DataGraphLayouter.prototype = Object.create(BaseLayouter.prototype);
DataGraphLayouter.prototype.layoutConnection = connection => [dock(connection.source, connection.target), dock(connection.target, connection.source)];

const appendTextMark = (parent, shape, text, className = 'data-graph-type-mark') => {
  const mark = svg('text', { x: shape.width / 2, y: shape.height / 2, class: className, 'text-anchor': 'middle', dy: '.35em', 'aria-hidden': 'true' });
  mark.textContent = text; parent.append(mark); return mark;
};
const appendCollectionMark = (parent, shape, collectionType) => {
  if (collectionType === 'scalar') return null;
  if (collectionType === 'stack' || collectionType === 'queue') {
    const group = svg('g', { class: `data-graph-collection-mark data-graph-${collectionType}-mark`, 'aria-hidden': 'true' });
    for (let index = -1; index <= 1; index += 1) {
      const horizontal = collectionType === 'stack';
      group.append(svg('line', horizontal
        ? { x1: 15, y1: shape.height / 2 + index * 7, x2: 35, y2: shape.height / 2 + index * 7 }
        : { x1: shape.width / 2 + index * 7, y1: 15, x2: shape.width / 2 + index * 7, y2: 35 }));
    }
    parent.append(group); return group;
  }
  return appendTextMark(parent, shape, typeDefinition('edgeNode', collectionType)?.mark ?? '{}', 'data-graph-type-mark data-graph-collection-text');
};

function DataGraphRenderer(eventBus) { BaseRenderer.call(this, eventBus, 1500); }
DataGraphRenderer.$inject = ['eventBus'];
DataGraphRenderer.prototype = Object.create(BaseRenderer.prototype);
DataGraphRenderer.prototype.canRender = () => true;
DataGraphRenderer.prototype.drawShape = function drawShape(parent, shape) {
  const type = typeOf(shape);
  if (type === 'edgeLabel') {
    const text = svg('text', { x: shape.width / 2, y: shape.height / 2, class: 'data-graph-edge-label', 'text-anchor': 'middle', dy: '.35em' });
    text.textContent = shape.businessObject?.label ?? ''; parent.append(text); return text;
  }
  if (type === 'edgeNode') {
    const outer = svg('circle', { cx: shape.width / 2, cy: shape.height / 2, r: shape.width / 2 - 1, class: 'data-graph-control-ring' });
    const inner = svg('circle', { cx: shape.width / 2, cy: shape.height / 2, r: shape.width / 2 - 7, class: 'data-graph-control-ring' });
    parent.setAttribute('aria-label', typeDefinition('edgeNode', shape.businessObject.collectionType)?.ariaLabel ?? 'Intermediate type');
    parent.append(outer, inner); appendCollectionMark(parent, shape, shape.businessObject.collectionType); return outer;
  }
  const visualType = type === 'objectNode' ? 'object' : type === 'mutationNode' ? 'mutation' : type;
  const circle = svg('circle', { cx: shape.width / 2, cy: shape.height / 2, r: shape.width / 2 - 2, class: `data-graph-shape data-graph-${visualType}` });
  parent.setAttribute('aria-label', type === 'objectNode' ? `Object ${shape.businessObject.label}` : `${typeDefinition(type, shape.businessObject[subtypeField(type)])?.ariaLabel}, ${shape.businessObject.label}`);
  parent.append(circle);
  if (type === 'objectNode') appendTextMark(parent, shape, shape.businessObject.label.length > 18 ? `${shape.businessObject.label.slice(0, 17)}…` : shape.businessObject.label, 'data-graph-node-label');
  else {
    appendTextMark(parent, shape, typeDefinition(type, shape.businessObject[subtypeField(type)])?.mark ?? '?', type === 'mutationNode' ? 'data-graph-type-mark data-graph-mutation-mark' : 'data-graph-type-mark');
    const label = svg('text', { x: shape.width / 2, y: shape.height + 15, class: 'data-graph-compact-label', 'text-anchor': 'middle' }); label.textContent = shape.businessObject.label; parent.append(label);
  }
  return circle;
};
DataGraphRenderer.prototype.drawConnection = function drawConnection(parent, connection) {
  const notation = connectionNotation(connection);
  const marker = notation === 'property' ? null : notation === 'mutation' ? 'url(#data-graph-mutation-target)' : 'url(#data-graph-arrow)';
  const sourceMarker = notation === 'object-link' && connection.businessObject.linkType === 'subclassOf' ? 'url(#data-graph-object-link-subclassOf)' : null;
  const path = svg('polyline', {
    points: connection.waypoints.map(point => `${point.x},${point.y}`).join(' '),
    class: `data-graph-edge data-graph-edge--${notation}${typeOf(connection) === 'attachment' ? ' data-graph-attachment' : ''}`,
    ...(sourceMarker ? { 'marker-start': sourceMarker } : {}),
    ...(marker ? { 'marker-end': marker } : {})
  });
  if (notation === 'object-link') parent.setAttribute('aria-label', typeDefinition('objectLink', connection.businessObject.linkType)?.ariaLabel ?? 'Object link');
  parent.append(path); return path;
};
DataGraphRenderer.prototype.getShapePath = shape => `M ${shape.x + shape.width / 2} ${shape.y} A ${shape.width / 2} ${shape.height / 2} 0 1 1 ${shape.x + shape.width / 2 - .1} ${shape.y}`;
DataGraphRenderer.prototype.getConnectionPath = connection => connection.waypoints.map((point, index) => `${index ? 'L' : 'M'} ${point.x} ${point.y}`).join(' ');

function DataGraphRules(eventBus) { RuleProvider.call(this, eventBus); }
DataGraphRules.$inject = ['eventBus'];
DataGraphRules.prototype = Object.create(RuleProvider.prototype);
DataGraphRules.prototype.init = function init() {
  this.addRule('connection.create', context => {
    if (typeOf(context.source) === 'edgeNode') {
      if (typeOf(context.target) !== 'objectNode') return false;
      const targets = (context.source.outgoing ?? []).filter(connection => typeOf(connection) === 'range');
      if (targets.some(connection => connection.target === context.target) || (context.source.businessObject.collectionType === 'scalar' && targets.length)) return false;
      return { id: `${context.source.id}_range_${context.target.id}`, type: 'dg:range', businessObject: { type: 'range' } };
    }
    if (typeOf(context.source) === 'mutationNode' && ['objectNode', 'edgeNode'].includes(typeOf(context.target))) return { id: `${context.source.id}_modifier_${context.target.id}`, type: 'dg:modifier', businessObject: { type: 'modifier' } };
    if (typeOf(context.source) === 'objectNode' && typeOf(context.target) === 'objectNode') {
      const intent = context.source.businessObject._connectionIntent ?? 'relationship';
      if (intent === 'objectLink') {
        if (context.source === context.target) return false;
        const duplicate = (context.source.outgoing ?? []).some(connection => typeOf(connection) === 'objectLink' && connection.target === context.target && connection.businessObject.linkType === 'sameAs');
        if (duplicate) return false;
        return { id: idSequence('object-link'), type: 'dg:objectLink', businessObject: businessObjectFor('objectLink') };
      }
      return { id: idSequence('relationship-gesture'), type: 'dg:relationshipGesture', businessObject: { type: 'relationshipGesture' } };
    }
    return false;
  });
  this.addRule('connection.reconnect', context => typeOf(context.connection) !== 'objectLink' && !['mutationNode', 'edgeNode'].includes(typeOf(context.hover ?? context.target)));
  this.addRule('shape.create', () => true);
  this.addRule('shape.move', () => true);
};

function DataGraphPalette(palette, create, elementFactory, canvas) {
  this.getPaletteEntries = () => {
    const start = type => event => {
      const size = sizeFor(type); const shape = elementFactory.createShape({ id: idSequence(type), type: `dg:${type}`, width: size, height: size, businessObject: businessObjectFor(type) });
      create.start(event, shape, { source: canvas.getRootElement() });
    };
    return {
      'create-object': { group: 'model', className: 'data-graph-tool data-graph-tool-object', title: 'Create object', action: { dragstart: start('objectNode'), click: start('objectNode') } },
      'create-property': { group: 'model', className: 'data-graph-tool data-graph-tool-property', title: 'Create property', action: { dragstart: start('property'), click: start('property') } },
      'create-mutation': { group: 'model', className: 'data-graph-tool data-graph-tool-mutation', title: 'Create mutation', action: { dragstart: start('mutationNode'), click: start('mutationNode') } }
    };
  };
  palette.registerProvider(this);
}
DataGraphPalette.$inject = ['palette', 'create', 'elementFactory', 'canvas'];

function UpdateSubtypeHandler(graphicsFactory, elementRegistry) { this.graphicsFactory = graphicsFactory; this.elementRegistry = elementRegistry; }
UpdateSubtypeHandler.$inject = ['graphicsFactory', 'elementRegistry'];
UpdateSubtypeHandler.prototype.execute = function execute(context) {
  const type = typeOf(context.element); const field = subtypeField(type);
  if (!field || !typeDefinition(type, context.value)) throw new Error(`Unsupported ${type} subtype ${context.value}`);
  if (type === 'edgeNode' && context.value === 'scalar' && (context.element.outgoing ?? []).filter(connection => typeOf(connection) === 'range').length > 1) throw new Error('Cannot set a multi-target edge node to scalar');
  if (type === 'objectLink' && (context.element.source.outgoing ?? []).some(connection => connection !== context.element && typeOf(connection) === 'objectLink' && connection.target === context.element.target && connection.businessObject.linkType === context.value)) throw new Error('Object link already exists');
  context.field = field; context.oldValue = context.element.businessObject[field]; context.element.businessObject[field] = context.value;
  this.graphicsFactory.update(type === 'objectLink' ? 'connection' : 'shape', context.element, this.elementRegistry.getGraphics(context.element)); return context.element;
};
UpdateSubtypeHandler.prototype.revert = function revert(context) {
  context.element.businessObject[context.field] = context.oldValue;
  this.graphicsFactory.update(typeOf(context.element) === 'objectLink' ? 'connection' : 'shape', context.element, this.elementRegistry.getGraphics(context.element)); return context.element;
};

function DataGraphTypeMenu(popupMenu, commandStack) {
  this.getPopupMenuEntries = target => Object.fromEntries(TYPE_REGISTRY[typeOf(target)].entries.map(entry => {
    const selected = target.businessObject[subtypeField(typeOf(target))] === entry.key;
    return [entry.key, {
    group: { id: 'types', name: 'Types' }, label: `${selected ? '✓ ' : ''}${entry.label}`, title: `${entry.ariaLabel}${selected ? ', selected' : ''}`, description: selected ? 'Currently selected' : `Set ${entry.label}`,
    className: `data-graph-type-entry data-graph-type-${entry.key}${selected ? ' selected' : ''}`,
    imageHtml: `<span class="data-graph-menu-mark" aria-hidden="true">${entry.mark}</span>`,
    action: () => { commandStack.execute('dg.updateSubtype', { element: target, value: entry.key }); popupMenu.close(); }
  }]; }));
  popupMenu.registerProvider(TYPE_MENU_ID, this);
}
DataGraphTypeMenu.$inject = ['popupMenu', 'commandStack'];

function DataGraphContextPad(contextPad, modeling, elementFactory, connect, elementRegistry, popupMenu, commandStack) {
  const createRelationship = (source, target = null, terminalPosition = null) => { const context = { source, target, parent: source.parent, terminalPosition }; commandStack.execute('dg.createRelationship', context); return context.relationship; };
  const append = (source, type) => () => {
    const isControl = typeOf(source) === 'edgeNode'; const size = sizeFor(type);
    const shape = elementFactory.createShape({ id: idSequence(type), type: `dg:${type}`, width: size, height: size, businessObject: businessObjectFor(type, { ownerId: type === 'property' ? source.id : null }) });
    modeling.createShape(shape, radialPlacement(source, elementRegistry, size), source.parent);
    if (type === 'objectNode') { if (isControl) addMembership(modeling, source, shape); else createRelationship(source, shape); }
    else if (type === 'mutationNode') modeling.connect(shape, source, { id: `${shape.id}_modifier_${source.id}`, type: 'dg:modifier', businessObject: { type: 'modifier' } });
    else modeling.connect(source, shape, { id: `${shape.id}_ownership`, type: 'dg:attachment', businessObject: { type: 'attachment', ownerId: source.id } });
  };
  const startConnection = (source, intent) => event => { source.businessObject._connectionIntent = intent; connect.start(event, source); };
  const wrench = element => { const position = center(element); return { group: 'edit', className: 'data-graph-tool data-graph-tool-wrench', title: `Set ${typeOf(element) === 'edgeNode' ? 'intermediate' : typeOf(element) === 'objectLink' ? 'object link modifier' : typeOf(element)} type`, action: { click: () => popupMenu.open(element, TYPE_MENU_ID, { x: position.x + 12, y: position.y }) } }; };
  this.getContextPadEntries = element => {
    if (typeOf(element) === 'edgeNode') return {
      'set-type': wrench(element),
      'connect-object': { group: 'connect', className: 'data-graph-tool data-graph-tool-connect', title: 'Connect object', action: { click: event => connect.start(event, element), dragstart: event => connect.start(event, element) } },
      'append-object': { group: 'model', className: 'data-graph-tool data-graph-tool-object', title: 'Create object', action: { click: append(element, 'objectNode') } },
      'append-edge-property': { group: 'model', className: 'data-graph-tool data-graph-tool-property', title: 'Add edge property', action: { click: append(element, 'property') } },
      'append-mutation': { group: 'model', className: 'data-graph-tool data-graph-tool-mutation', title: 'Add mutation', action: { click: append(element, 'mutationNode') } }
    };
    if (typeOf(element) === 'objectLink') return { 'set-type': wrench(element) };
    if (!['objectNode', 'property', 'mutationNode'].includes(typeOf(element))) return {};
    if (typeOf(element) === 'property') return { 'set-type': wrench(element) };
    if (typeOf(element) === 'mutationNode') return { 'set-type': wrench(element), 'append-property': { group: 'model', className: 'data-graph-tool data-graph-tool-property', title: 'Add parameter', action: { click: append(element, 'property') } }, connect: { group: 'connect', className: 'data-graph-tool data-graph-tool-connect', title: 'Connect modifier', action: { click: event => connect.start(event, element), dragstart: event => connect.start(event, element) } } };
    return {
      ...(typeOf(element) !== 'objectNode' ? { 'set-type': wrench(element) } : {}),
      'append-object': { group: 'model', className: 'data-graph-tool data-graph-tool-object', title: 'Append object', action: { click: append(element, 'objectNode') } },
      'append-property': { group: 'model', className: 'data-graph-tool data-graph-tool-property', title: 'Append property', action: { click: append(element, 'property') } },
      ...(typeOf(element) === 'objectNode' ? { 'append-mutation': { group: 'model', className: 'data-graph-tool data-graph-tool-mutation', title: 'Add function', action: { click: append(element, 'mutationNode') } } } : {}),
      'relationship': { group: 'connect', className: 'data-graph-tool data-graph-tool-connect', title: 'Relationship', action: { click: () => createRelationship(element), dragstart: startConnection(element, 'relationship') } },
      'object-link': { group: 'connect', className: 'data-graph-tool data-graph-tool-object-link', title: 'Object link', action: { click: startConnection(element, 'objectLink'), dragstart: startConnection(element, 'objectLink') } }
    };
  };
  contextPad.registerProvider(this);
}
DataGraphContextPad.$inject = ['contextPad', 'modeling', 'elementFactory', 'connect', 'elementRegistry', 'popupMenu', 'commandStack'];

function CreateRelationshipHandler(modeling, elementFactory) { this.modeling = modeling; this.elementFactory = elementFactory; }
CreateRelationshipHandler.$inject = ['modeling', 'elementFactory'];
CreateRelationshipHandler.prototype.preExecute = function preExecute(context) { context.relationship = createLogicalRelationship(this.modeling, this.elementFactory, context.source, context.target, context.parent, context.edgeNodeId, context.label, context.terminalPosition); };
CreateRelationshipHandler.prototype.execute = context => context.relationship?.control;
CreateRelationshipHandler.prototype.revert = () => {};

function DataGraphConnectionBehavior(eventBus, modeling, elementFactory, commandStack) {
  CommandInterceptor.call(this, eventBus);
  commandStack.registerHandler('dg.createRelationship', CreateRelationshipHandler);
  this.postExecute('connection.create', context => {
    const connection = context.connection;
    if (typeOf(connection) !== 'relationshipGesture') return;
    const { source, target } = connection;
    modeling.removeConnection(connection);
    context.relationship = createLogicalRelationship(modeling, elementFactory, source, target, source.parent);
  }, true);
  eventBus.on('connect.end', 500, event => {
    const source = event.context.start;
    if (typeOf(source) !== 'objectNode' || source.businessObject._connectionIntent !== 'relationship' || event.context.target) return;
    commandStack.execute('dg.createRelationship', { source, target: null, parent: source.parent, terminalPosition: { x: event.x, y: event.y } });
  });
  eventBus.on('connect.cleanup', 500, event => { delete event.context.start?.businessObject?._connectionIntent; });
}
DataGraphConnectionBehavior.$inject = ['eventBus', 'modeling', 'elementFactory', 'commandStack'];
DataGraphConnectionBehavior.prototype = Object.create(CommandInterceptor.prototype);

function UpdateLabelHandler(graphicsFactory, elementRegistry) { this.graphicsFactory = graphicsFactory; this.elementRegistry = elementRegistry; }
UpdateLabelHandler.$inject = ['graphicsFactory', 'elementRegistry'];
UpdateLabelHandler.prototype.execute = function execute(context) {
  context.oldLabel = context.element.businessObject.label; context.element.businessObject.label = context.newLabel;
  this.graphicsFactory.update(['edgeSegment', 'attachment'].includes(typeOf(context.element)) ? 'connection' : 'shape', context.element, this.elementRegistry.getGraphics(context.element)); return context.element;
};
UpdateLabelHandler.prototype.revert = function revert(context) {
  context.element.businessObject.label = context.oldLabel;
  this.graphicsFactory.update(['edgeSegment', 'attachment'].includes(typeOf(context.element)) ? 'connection' : 'shape', context.element, this.elementRegistry.getGraphics(context.element)); return context.element;
};

function DataGraphDirectEditing(directEditing, eventBus, commandStack) {
  commandStack.registerHandler('dg.updateLabel', UpdateLabelHandler); commandStack.registerHandler('dg.updateSubtype', UpdateSubtypeHandler);
  this.activate = element => {
    const middle = ['edgeSegment', 'attachment'].includes(typeOf(element)) ? midpointOnPath(element.waypoints) : center(element);
    return { bounds: { x: middle.x - 70, y: middle.y - 18, width: 140, height: 36 }, text: element.businessObject?.label ?? '', options: { resizable: true } };
  };
  this.update = (element, value) => commandStack.execute('dg.updateLabel', { element, newLabel: value.trim() || element.businessObject.label });
  directEditing.registerProvider(this);
  eventBus.on('element.dblclick', event => { if (['objectNode', 'property', 'mutationNode', 'edgeLabel'].includes(typeOf(event.element))) directEditing.activate(event.element); });
}
DataGraphDirectEditing.$inject = ['directEditing', 'eventBus', 'commandStack'];

function DataGraphMoveBehavior(eventBus, modeling, elementRegistry) {
  CommandInterceptor.call(this, eventBus);
  this.preExecute('shape.move', context => {
    if (typeOf(context.shape) !== 'mutationNode' || !context.delta) return;
    const origin = center(context.shape); const requested = { x: origin.x + context.delta.x, y: origin.y + context.delta.y };
    const resolved = collisionFreePosition(requested, context.shape.width, elementRegistry, context.shape.id);
    context.delta = { x: resolved.x - origin.x, y: resolved.y - origin.y };
  }, true);
  this.postExecute('shape.move', context => {
    if (typeOf(context.shape) !== 'edgeNode') return;
    const label = elementRegistry.get(`${context.shape.id}-label`);
    if (label && context.delta) modeling.moveShape(label, context.delta, label.parent);
  }, true);
}
DataGraphMoveBehavior.$inject = ['eventBus', 'modeling', 'elementRegistry'];
DataGraphMoveBehavior.prototype = Object.create(CommandInterceptor.prototype);

function DataGraphDeleteBehavior(eventBus, modeling, elementRegistry) {
  CommandInterceptor.call(this, eventBus);
  const deletingEdges = new Set();
  this.preExecute('connection.delete', context => {
    if (typeOf(context.connection) !== 'domain') return;
    const edgeNode = context.connection.target; context.edgeNode = edgeNode; context.ownerIds = new Set([edgeNode.id, ...(edgeNode.incoming ?? []).map(edge => edge.id), ...(edgeNode.outgoing ?? []).map(edge => edge.id)]);
  }, true);
  this.postExecute('connection.delete', context => {
    if (typeOf(context.connection) !== 'domain') return;
    const edgeNode = context.edgeNode; const edgeId = edgeNode.id;
    if (deletingEdges.has(edgeId)) return;
    const related = elementRegistry.getAll().filter(element => element === edgeNode || context.ownerIds.has(element.businessObject?.ownerId) || element.businessObject?.edgeId === edgeId);
    if (related.length) {
      deletingEdges.add(edgeId);
      try { modeling.removeElements(related); } finally { deletingEdges.delete(edgeId); }
    }
  }, true);
  this.postExecute('shape.delete', context => {
    if (!['edgeNode', 'mutationNode'].includes(typeOf(context.shape))) return;
    const ownerIds = new Set([context.shape.id, ...(context.shape.incoming ?? []).map(edge => edge.id), ...(context.shape.outgoing ?? []).map(edge => edge.id)]);
    const related = elementRegistry.getAll().filter(element => ownerIds.has(element.businessObject?.ownerId) || element.businessObject?.edgeId === context.shape.id);
    if (related.length) modeling.removeElements(related);
  }, true);
}
DataGraphDeleteBehavior.$inject = ['eventBus', 'modeling', 'elementRegistry'];
DataGraphDeleteBehavior.prototype = Object.create(CommandInterceptor.prototype);

function UpdateContextHandler() {}
UpdateContextHandler.prototype.execute = function execute(context) { context.oldDraft = context.drafts.get(context.nodeId); context.drafts.apply(context.nodeId, context.newDraft); return context.newDraft; };
UpdateContextHandler.prototype.revert = function revert(context) { context.drafts.apply(context.nodeId, context.oldDraft); return context.oldDraft; };

function DataGraphContextDrafts(eventBus, commandStack) {
  this.values = new Map(); this.tombstones = new Set();
  commandStack.registerHandler('dg.updateContext', UpdateContextHandler);
  eventBus.on('commandStack.shape.delete.executed', event => { const id = event.context?.shape?.id; if (id && this.values.has(id)) { event.context.deletedDraft = this.values.get(id); this.values.delete(id); this.tombstones.add(id); } });
  eventBus.on('commandStack.shape.delete.reverted', event => { const id = event.context?.shape?.id; if (id && event.context.deletedDraft) { this.values.set(id, event.context.deletedDraft); this.tombstones.delete(id); } });
}
DataGraphContextDrafts.$inject = ['eventBus', 'commandStack'];
DataGraphContextDrafts.prototype.get = function get(nodeId) { return this.values.get(nodeId) ?? null; };
DataGraphContextDrafts.prototype.apply = function apply(nodeId, draft) { if (draft) this.values.set(nodeId, draft); else this.values.delete(nodeId); this.tombstones.delete(nodeId); };
DataGraphContextDrafts.prototype.update = function update(nodeId, markdown, commandStack) { const prior = this.get(nodeId); commandStack.execute('dg.updateContext', { drafts: this, nodeId, newDraft: contextDraft(nodeId, markdown, prior?.sections) }); };
DataGraphContextDrafts.prototype.clear = function clear() { this.values.clear(); this.tombstones.clear(); };
DataGraphContextDrafts.prototype.snapshot = function snapshot() { return { drafts: [...this.values.values()], tombstones: [...this.tombstones] }; };

const DataGraphModule = {
  __init__: ['dataGraphRenderer', 'dataGraphRules', 'dataGraphPalette', 'dataGraphTypeMenu', 'dataGraphContextPad', 'dataGraphConnectionBehavior', 'dataGraphDirectEditing', 'dataGraphMoveBehavior', 'dataGraphDeleteBehavior', 'dataGraphContextDrafts'],
  dataGraphRenderer: ['type', DataGraphRenderer], dataGraphRules: ['type', DataGraphRules], dataGraphPalette: ['type', DataGraphPalette], dataGraphTypeMenu: ['type', DataGraphTypeMenu],
  dataGraphContextPad: ['type', DataGraphContextPad], dataGraphConnectionBehavior: ['type', DataGraphConnectionBehavior], dataGraphDirectEditing: ['type', DataGraphDirectEditing], dataGraphMoveBehavior: ['type', DataGraphMoveBehavior], dataGraphDeleteBehavior: ['type', DataGraphDeleteBehavior], dataGraphContextDrafts: ['type', DataGraphContextDrafts], layouter: ['type', DataGraphLayouter]
};
const modules = [ModelingModule, MoveModule, CreateModule, ConnectModule, BendpointsModule, PaletteModule, ContextPadModule, PopupMenuModule, KeyboardModule, KeyboardMoveModule, KeyboardMoveSelectionModule, MoveCanvasModule, ZoomScrollModule, EditorActionsModule, HandToolModule, LassoToolModule, GlobalConnectModule, AutoScrollModule, ConnectionPreviewModule, GridSnappingModule, ClipboardModule, CopyPasteModule, SpaceToolModule, AlignElementsModule, DistributeElementsModule, HoverFixModule, LabelSupportModule, DirectEditingModule, DataGraphModule];

export class DataGraphModeler extends Diagram {
  constructor({ container }) { container.classList.add('data-graph-editor'); super({ canvas: { container }, modules }); this.container = container; this.sequence = 1; this.contextStore = starterDocument().contextStore; this.installMarker(); }
  on(name, listener) { this.get('eventBus').on(name, listener); }
  installMarker() {
    const svgRoot = this.get('canvas')._svg; let defs = svgRoot.querySelector('defs'); if (!defs) { defs = svg('defs'); svgRoot.prepend(defs); }
    const marker = svg('marker', { id: 'data-graph-arrow', viewBox: '0 0 10 10', refX: 9, refY: 5, markerWidth: 7, markerHeight: 7, orient: 'auto-start-reverse' }); marker.append(svg('path', { d: 'M 0 0 L 10 5 L 0 10 z', class: 'data-graph-arrow' })); defs.append(marker);
    const mutationTarget = svg('marker', { id: 'data-graph-mutation-target', viewBox: '0 0 10 10', refX: 5, refY: 5, markerWidth: 7, markerHeight: 7, orient: 'auto' }); mutationTarget.append(svg('circle', { cx: 5, cy: 5, r: 4, class: 'data-graph-mutation-target' })); defs.append(mutationTarget);
    const subclassMarker = svg('marker', { id: 'data-graph-object-link-subclassOf', viewBox: '0 0 14 14', refX: 7, refY: 7, markerWidth: 8, markerHeight: 8, orient: 'auto' }); subclassMarker.append(svg('circle', { cx: 7, cy: 7, r: 4.5, class: 'data-graph-object-link-marker' })); defs.append(subclassMarker);
  }
  async importJSON(text) {
    const documentValue = parseDocument(text); this.contextStore = documentValue.contextStore; this.get('dataGraphContextDrafts').clear(); this.clear(); const canvas = this.get('canvas'); const factory = this.get('elementFactory'); const root = canvas.getRootElement(); const elements = new Map();
    documentValue.nodes.forEach(node => {
      const size = sizeFor(node.type); const shape = factory.createShape({ id: node.id, type: `dg:${node.type}`, x: node.x - size / 2, y: node.y - size / 2, width: size, height: size, businessObject: { ...node } }); canvas.addShape(shape, root); elements.set(node.id, shape);
      if (node.type === 'edgeNode') { const labelCenter = defaultControlLabelPosition(shape); const labelShape = factory.createLabel({ id: `${node.id}-label`, type: 'label', x: labelCenter.x - LABEL_WIDTH / 2, y: labelCenter.y - LABEL_HEIGHT / 2, width: LABEL_WIDTH, height: LABEL_HEIGHT, labelTarget: shape, businessObject: { type: 'edgeLabel', edgeId: node.id, label: node.label } }); canvas.addShape(labelShape, root); }
    });
    documentValue.edges.forEach(edge => {
      const source = elements.get(edge.source); const target = elements.get(edge.target); const connection = factory.createConnection({ id: edge.id, type: `dg:${edge.type}`, source, target, waypoints: edge.waypoints.length >= 2 ? edge.waypoints : [dock(source, target), dock(target, source)], businessObject: { type: edge.type, ...(edge.type === 'objectLink' ? { linkType: edge.linkType } : {}) } }); canvas.addConnection(connection, root); elements.set(edge.id, connection);
    });
    documentValue.properties.forEach(property => {
      const size = sizeFor(property.type); const node = factory.createShape({ id: property.id, type: `dg:${property.type}`, x: property.x - size / 2, y: property.y - size / 2, width: size, height: size, businessObject: { ...property } }); canvas.addShape(node, root); elements.set(property.id, node); const owner = elements.get(property.ownerId);
      const connection = factory.createConnection({ id: `${property.id}_ownership`, type: 'dg:attachment', source: owner, target: node, waypoints: [dock(owner, node), dock(node, owner)], businessObject: { type: 'attachment', ownerId: property.ownerId } });
      canvas.addConnection(connection, root);
    });
    this.get('commandStack').clear(); canvas.zoom('fit-viewport'); return { warnings: [] };
  }
  async saveJSON() {
    const all = this.get('elementRegistry').getAll();
    const labels = new Map(all.filter(element => typeOf(element) === 'edgeLabel').map(element => [element.businessObject.edgeId, element]));
    const nodes = all.filter(element => ['objectNode', 'edgeNode', 'mutationNode'].includes(typeOf(element))).map(element => { const type = typeOf(element); const label = labels.get(element.id); return { id: element.id, type, label: label?.businessObject.label ?? element.businessObject.label, x: center(element).x, y: center(element).y, ...(type === 'edgeNode' ? { collectionType: element.businessObject.collectionType } : {}), ...(type === 'mutationNode' ? { mutationType: element.businessObject.mutationType } : {}) }; });
    const edges = all.filter(element => ['domain', 'range', 'modifier', 'objectLink'].includes(typeOf(element))).map(connection => ({ id: connection.id, type: typeOf(connection), source: connection.source.id, target: connection.target.id, waypoints: connection.waypoints.map(({ x, y }) => ({ x, y })), ...(typeOf(connection) === 'objectLink' ? { linkType: connection.businessObject.linkType } : {}) }));
    const properties = all.filter(element => typeOf(element) === 'property').map(element => ({ id: element.id, type: 'property', ownerId: element.businessObject.ownerId, label: element.businessObject.label, x: center(element).x, y: center(element).y, dataType: element.businessObject.dataType }));
    return { json: serializeDocument({ ...starterDocument(), nodes, edges, properties, contextStore: this.contextStore }) };
  }
  addNode(type, ownerId = null) {
    if (!['objectNode', 'property', 'mutationNode'].includes(type)) throw new Error(`Unsupported node type ${type}`);
    const canvas = this.get('canvas'); const modeling = this.get('modeling'); const factory = this.get('elementFactory'); const registry = this.get('elementRegistry'); const id = `${type}-${this.sequence++}`; const size = sizeFor(type);
    const shape = factory.createShape({ id, type: `dg:${type}`, width: size, height: size, businessObject: businessObjectFor(type, { ownerId }) });
    const existing = registry.getAll().find(element => ['objectNode', 'property', 'mutationNode'].includes(typeOf(element))); modeling.createShape(shape, existing ? radialPlacement(existing, registry, size) : { x: 180, y: 180 }, canvas.getRootElement()); return shape;
  }
  createRelationship(source, target = null, terminalPosition = null) { const context = { source, target, parent: source.parent, terminalPosition }; this.get('commandStack').execute('dg.createRelationship', context); return context.relationship; }
  connect(sourceId, targetId) { const registry = this.get('elementRegistry'); return this.createRelationship(registry.get(sourceId), registry.get(targetId)); }
  addRelationship(sourceId, terminalPosition = null) { return this.createRelationship(this.get('elementRegistry').get(sourceId), null, terminalPosition); }
  addTarget(edgeNodeId, targetId) { const registry = this.get('elementRegistry'); return addMembership(this.get('modeling'), registry.get(edgeNodeId), registry.get(targetId)); }
  addObjectLink(sourceId, targetId, linkType = 'sameAs') {
    const registry = this.get('elementRegistry'); const source = registry.get(sourceId); const target = registry.get(targetId);
    if (typeOf(source) !== 'objectNode' || typeOf(target) !== 'objectNode' || source === target) throw new Error('Object links require distinct object-node endpoints');
    if (!typeDefinition('objectLink', linkType)) throw new Error(`Unsupported objectLink subtype ${linkType}`);
    if ((source.outgoing ?? []).some(connection => typeOf(connection) === 'objectLink' && connection.target === target && connection.businessObject.linkType === linkType)) throw new Error('Object link already exists');
    return this.get('modeling').connect(source, target, { id: idSequence('object-link'), type: 'dg:objectLink', businessObject: businessObjectFor('objectLink', { linkType }) });
  }
  effectiveConnections(objectId) { return effectiveConnections(JSON.parse(this.saveJSONSync()), objectId); }
  saveJSONSync() {
    const all = this.get('elementRegistry').getAll(); const labels = new Map(all.filter(element => typeOf(element) === 'edgeLabel').map(element => [element.businessObject.edgeId, element]));
    const nodes = all.filter(element => ['objectNode', 'edgeNode', 'mutationNode'].includes(typeOf(element))).map(element => { const type = typeOf(element); return { id: element.id, type, label: labels.get(element.id)?.businessObject.label ?? element.businessObject.label, x: center(element).x, y: center(element).y, ...(type === 'edgeNode' ? { collectionType: element.businessObject.collectionType } : {}), ...(type === 'mutationNode' ? { mutationType: element.businessObject.mutationType } : {}) }; });
    const edges = all.filter(element => ['domain', 'range', 'modifier', 'objectLink'].includes(typeOf(element))).map(connection => ({ id: connection.id, type: typeOf(connection), source: connection.source.id, target: connection.target.id, waypoints: connection.waypoints.map(({ x, y }) => ({ x, y })), ...(typeOf(connection) === 'objectLink' ? { linkType: connection.businessObject.linkType } : {}) }));
    const properties = all.filter(element => typeOf(element) === 'property').map(element => ({ id: element.id, type: 'property', ownerId: element.businessObject.ownerId, label: element.businessObject.label, x: center(element).x, y: center(element).y, dataType: element.businessObject.dataType }));
    return serializeDocument({ ...starterDocument(), nodes, edges, properties, contextStore: this.contextStore });
  }
  moveNode(id, x, y) { const element = this.get('elementRegistry').get(id); const current = center(element); this.get('modeling').moveShape(element, { x: x - current.x, y: y - current.y }, element.parent); }
  nodeMarkdown(id) { return this.get('dataGraphContextDrafts').get(id)?.markdown ?? ''; }
  setNodeMarkdown(id, markdown) { if (!this.get('elementRegistry').get(id)) throw new Error(`Unknown node ${id}`); this.get('dataGraphContextDrafts').update(id, markdown, this.get('commandStack')); }
  contextSnapshot() { const snapshot = this.get('dataGraphContextDrafts').snapshot(); const registry = this.get('elementRegistry'); return { ...snapshot, drafts: snapshot.drafts.map(draft => { const node = registry.get(draft.nodeId); return { ...draft, nodeType: typeOf(node), nodeLabel: node?.businessObject?.label ?? draft.nodeId }; }) }; }
  importContextDrafts(drafts = []) { const service = this.get('dataGraphContextDrafts'); drafts.forEach(draft => { if (this.get('elementRegistry').get(draft.nodeId)) service.apply(draft.nodeId, { contextId: draft.contextId, nodeId: draft.nodeId, markdown: draft.markdown, sections: draft.sections }); }); }
  setSubtype(id, value) { const element = this.get('elementRegistry').get(id); this.get('commandStack').execute('dg.updateSubtype', { element, value }); }
}
