class ResizeObserver { observe() {} unobserve() {} disconnect() {} }
globalThis.ResizeObserver = ResizeObserver;
globalThis.CSS ??= {};
globalThis.CSS.escape ??= value => String(value).replace(/[^a-zA-Z0-9_-]/g, match => `\\${match}`);
globalThis.SVGMatrix ??= class SVGMatrix {};
globalThis.HTMLCanvasElement.prototype.getContext = () => ({ measureText: text => ({ width: String(text).length * 7 }) });
if (!globalThis.SVGElement.prototype.getBBox) globalThis.SVGElement.prototype.getBBox = () => ({ x: 0, y: 0, width: 100, height: 100 });
if (!globalThis.SVGElement.prototype.getCTM) globalThis.SVGElement.prototype.getCTM = () => ({ a: 1, b: 0, c: 0, d: 1, e: 0, f: 0 });
if (!globalThis.SVGElement.prototype.createSVGTransform) globalThis.SVGElement.prototype.createSVGTransform = () => ({ setTranslate() {}, setScale() {} });
if (!Object.getOwnPropertyDescriptor(globalThis.SVGElement.prototype, 'transform')) {
  Object.defineProperty(globalThis.SVGElement.prototype, 'transform', {
    get: () => ({ baseVal: { clear() {}, appendItem() {}, consolidate: () => ({ matrix: Object.assign(new globalThis.SVGMatrix(), { inverse() { return this; }, multiply() { return this; }, translate() { return this; }, scale() { return this; } }) }), createSVGTransformFromMatrix: matrix => matrix } })
  });
}
if (!globalThis.SVGSVGElement.prototype.createSVGMatrix) {
  globalThis.SVGSVGElement.prototype.createSVGMatrix = () => {
    const matrix = Object.assign(new globalThis.SVGMatrix(), { a: 1, b: 0, c: 0, d: 1, e: 0, f: 0 });
    matrix.scale = () => matrix;
    matrix.translate = () => matrix;
    matrix.multiply = () => matrix;
    matrix.inverse = () => matrix;
    return matrix;
  };
}
if (!globalThis.SVGSVGElement.prototype.createSVGPoint) {
  globalThis.SVGSVGElement.prototype.createSVGPoint = () => ({ x: 0, y: 0, matrixTransform() { return { x: this.x, y: this.y }; } });
}
