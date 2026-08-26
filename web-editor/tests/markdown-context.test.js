import { describe, expect, it } from 'vitest';
import { contentHash, contextDraft, normalizeMarkdown, sectionMarkdown } from '../src/data-graph/MarkdownContext.js';

describe('Markdown context sectioning', () => {
  it('normalizes newlines and builds preamble and nested heading paths', () => {
    const sections = sectionMarkdown('intro\r\n# One\nbody  \n## Two\nchild\n');
    expect(sections.map(section => section.headingPath)).toEqual([[], ['One'], ['One', 'Two']]);
    expect(sections.every(section => /^[0-9a-f]{64}$/.test(section.contentHash))).toBe(true);
  });
  it('reuses unchanged and uniquely corresponding section IDs', () => {
    const first = sectionMarkdown('# One\nbody\n# Two\nother\n');
    const second = sectionMarkdown('# One\nchanged\n# Two\nother\n', first);
    expect(second[0].sectionId).toBe(first[0].sectionId);
    expect(second[0].contentHash).not.toBe(first[0].contentHash);
    expect(second[1].sectionId).toBe(first[1].sectionId);
  });
  it('does not attach an ambiguous duplicate heading to arbitrary prior content', () => {
    const first = sectionMarkdown('# Same\na\n# Same\nb\n');
    const reordered = sectionMarkdown('# Same\nb changed\n# Same\na changed\n', first);
    expect(new Set(reordered.map(section => section.sectionId)).size).toBe(2);
  });
  it('removes empty drafts and hashes canonical content', () => {
    expect(contextDraft('person', '  \n')).toBeNull();
    expect(contentHash('hello\r\n')).toBe(contentHash(normalizeMarkdown('hello\n')));
  });
});
