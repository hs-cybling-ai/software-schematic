import { EditorState } from '@codemirror/state';
import { EditorView, keymap } from '@codemirror/view';
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands';
import { markdown } from '@codemirror/lang-markdown';
import MarkdownIt from 'markdown-it';
import DOMPurify from 'dompurify';

const renderer = new MarkdownIt({ html: false, linkify: true, typographer: false });
export const renderMarkdown = source => DOMPurify.sanitize(renderer.render(source ?? ''), { USE_PROFILES: { html: true }, FORBID_TAGS: ['style', 'script', 'iframe', 'object', 'embed', 'form'] });

export class MarkdownInspector {
  constructor(container, { onCommit, onCapture, onClose }) {
    this.container = container; this.onCommit = onCommit; this.onCapture = onCapture; this.onClose = onClose; this.node = null; this.timer = null; this.mode = 'source';
    container.className = 'markdown-inspector'; container.hidden = true;
    container.innerHTML = `<header><div><small>Node context</small><h2 id="context-title">Markdown</h2></div><button type="button" data-action="close" aria-label="Close Markdown inspector">×</button></header><div class="markdown-tabs" role="tablist"><button type="button" role="tab" data-mode="source" aria-selected="true">Source</button><button type="button" role="tab" data-mode="preview" aria-selected="false">Preview</button></div><div class="markdown-source" aria-label="Markdown source editor"></div><article class="markdown-preview" aria-label="Markdown preview" hidden></article><footer><span class="capture-status" role="status">Not captured</span><button type="button" data-action="capture">Capture content + embeddings</button></footer>`;
    this.preview = container.querySelector('.markdown-preview'); this.status = container.querySelector('.capture-status');
    this.view = new EditorView({ parent: container.querySelector('.markdown-source'), state: EditorState.create({ doc: '', extensions: [history(), markdown(), keymap.of([...defaultKeymap, ...historyKeymap]), EditorView.updateListener.of(update => { if (update.docChanged && !this.loading) this.schedule(); })] }) });
    container.addEventListener('click', event => this.click(event));
    container.addEventListener('focusout', event => { if (!container.contains(event.relatedTarget)) this.flush(); });
    container.addEventListener('keydown', event => { if (event.key === 'Escape') { event.preventDefault(); this.close(); } });
  }
  open(node, source = '') { this.flush(); this.node = node; this.container.hidden = false; this.container.querySelector('#context-title').textContent = `${node.businessObject?.label ?? node.id} · ${node.id}`; this.replace(source); this.setMode('source'); }
  replace(source) { this.loading = true; this.view.dispatch({ changes: { from: 0, to: this.view.state.doc.length, insert: source } }); this.loading = false; }
  schedule() { globalThis.clearTimeout(this.timer); this.status.textContent = 'Draft changed'; this.timer = globalThis.setTimeout(() => this.flush(), 300); }
  flush() { globalThis.clearTimeout(this.timer); this.timer = null; if (!this.node) return; this.onCommit(this.node.id, this.view.state.doc.toString()); }
  setMode(mode) { this.mode = mode; const preview = mode === 'preview'; this.container.querySelector('.markdown-source').hidden = preview; this.preview.hidden = !preview; if (preview) this.preview.innerHTML = renderMarkdown(this.view.state.doc.toString()); this.container.querySelectorAll('[role=tab]').forEach(button => button.setAttribute('aria-selected', String(button.dataset.mode === mode))); }
  click(event) { const button = event.target.closest('button'); if (!button) return; if (button.dataset.mode) this.setMode(button.dataset.mode); else if (button.dataset.action === 'capture') { this.flush(); this.status.textContent = 'Capture requested…'; this.onCapture(); } else if (button.dataset.action === 'close') this.close(); }
  close() { this.flush(); this.node = null; this.container.hidden = true; this.onClose?.(); }
  setCaptureStatus(message) { this.status.textContent = message; }
  destroy() { this.flush(); this.view.destroy(); this.container.replaceChildren(); }
}
