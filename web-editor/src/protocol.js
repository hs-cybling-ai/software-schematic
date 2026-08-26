export const BRIDGE_VERSION = 1;
export const SUPPORTED_FORMATS = new Set(['bpmn', 'dataGraph']);
export const NATIVE_COMMANDS = new Set(['load', 'requestExport', 'undo', 'redo', 'appearance', 'captureCompleted', 'captureFailed']);
export const WEB_EVENTS = new Set(['ready', 'changed', 'exported', 'failed', 'warnings', 'captureRequested', 'captureProgress', 'captureCompleted']);

export function envelope(type, { requestId = null, format = null, payload = {} } = {}) {
  return { version: BRIDGE_VERSION, type, requestId, format, payload };
}

export function validateCommand(message) {
  if (!message || message.version !== BRIDGE_VERSION) throw new Error('Unsupported bridge version');
  if (!NATIVE_COMMANDS.has(message.type)) throw new Error(`Unsupported command: ${message.type}`);
  if (message.format && !SUPPORTED_FORMATS.has(message.format)) throw new Error(`Unsupported format: ${message.format}`);
  if (message.type === 'load' && (!message.requestId || !message.format || typeof message.payload?.xml !== 'string')) {
    throw new Error('Invalid load command');
  }
  if (message.type === 'requestExport' && !message.requestId) throw new Error('Export requires a request identifier');
  return message;
}

export function errorPayload(error) {
  return { message: error instanceof Error ? error.message : String(error) };
}
