import './styles.css';
import { EditorHost } from './EditorHost.js';

const editor = document.querySelector('#editor');
const status = document.querySelector('#status');
const nativeHandler = window.webkit?.messageHandlers?.diagramBridge;
const postMessage = message => {
  if (message.type === 'failed') status.textContent = message.payload.message;
  else if (message.type === 'warnings') status.textContent = message.payload.warnings.join('\n');
  nativeHandler?.postMessage(message);
};

const host = new EditorHost(editor, postMessage);
Object.defineProperty(window, 'diagramStudioReceive', {
  value: message => host.receive(message),
  configurable: false,
  writable: false
});
window.addEventListener('pagehide', () => host.destroy(), { once: true });
