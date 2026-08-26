import { describe, expect, it } from 'vitest';
import { BRIDGE_VERSION, envelope, validateCommand } from '../src/protocol.js';

describe('bridge protocol', () => {
  it('creates versioned correlated envelopes', () => {
    expect(envelope('exported', { requestId: '42', format: 'bpmn', payload: { xml: '<x/>' } })).toEqual({ version: BRIDGE_VERSION, type: 'exported', requestId: '42', format: 'bpmn', payload: { xml: '<x/>' } });
  });

  it('rejects invalid versions, formats, and load payloads', () => {
    expect(() => validateCommand({ version: 2, type: 'load' })).toThrow('version');
    expect(() => validateCommand({ version: 1, type: 'load', requestId: '1', format: 'uml', payload: { xml: '' } })).toThrow('format');
    expect(() => validateCommand({ version: 1, type: 'load', requestId: '1', format: 'bpmn', payload: {} })).toThrow('load');
  });

  it('accepts correlated context capture completion and failure commands', () => {
    expect(validateCommand({ version: 1, type: 'captureCompleted', requestId: 'c1', format: 'dataGraph', payload: { revision: 2 } }).payload.revision).toBe(2);
    expect(validateCommand({ version: 1, type: 'captureFailed', requestId: 'c1', format: 'dataGraph', payload: { message: 'failed' } }).payload.message).toBe('failed');
  });
});
