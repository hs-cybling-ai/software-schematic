import { describe, expect, it, vi } from 'vitest';
import { MarkdownInspector, renderMarkdown } from '../src/data-graph/MarkdownInspector.js';

describe('MarkdownInspector', () => {
  it('sanitizes executable HTML and unsafe links', () => { const html = renderMarkdown('# Safe\n<script>alert(1)</script>\n[x](javascript:alert(1))'); expect(html).toContain('<h1>Safe</h1>'); expect(html).not.toContain('<script'); expect(html).not.toMatch(/href=["']javascript:/); });
  it('opens, switches preview, flushes on close, and supports Escape', () => {
    const element = document.createElement('aside'); document.body.append(element); const commit = vi.fn(); const close = vi.fn();
    const inspector = new MarkdownInspector(element, { onCommit: commit, onCapture: vi.fn(), onClose: close });
    inspector.open({ id: 'person', businessObject: { label: 'Person' } }, '# Person');
    element.querySelector('[data-mode=preview]').click(); expect(element.querySelector('.markdown-preview').innerHTML).toContain('Person');
    element.dispatchEvent(new window.KeyboardEvent('keydown', { key: 'Escape', bubbles: true })); expect(commit).toHaveBeenCalledWith('person', '# Person'); expect(close).toHaveBeenCalled(); inspector.destroy();
  });
});
