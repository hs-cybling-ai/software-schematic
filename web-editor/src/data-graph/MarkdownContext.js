import { sha256 } from '@noble/hashes/sha2.js';
import { bytesToHex } from '@noble/hashes/utils.js';

export const normalizeMarkdown = value => String(value ?? '').replace(/\r\n?/g, '\n').replace(/[ \t]+$/gm, '').replace(/\n*$/, '') + (String(value ?? '').trim() ? '\n' : '');
export const contentHash = value => bytesToHex(sha256(new globalThis.TextEncoder().encode(normalizeMarkdown(value))));

const slug = value => value.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '').slice(0, 32) || 'section';

export function sectionMarkdown(markdown, previous = []) {
  const source = normalizeMarkdown(markdown);
  if (!source) return [];
  const lines = source.split('\n');
  const boundaries = [];
  lines.forEach((line, index) => { const match = /^(#{1,6})\s+(.+?)\s*#*\s*$/.exec(line); if (match) boundaries.push({ index, level: match[1].length, title: match[2].trim() }); });
  if (!boundaries.length) boundaries.push({ index: 0, level: 0, title: 'Preamble', synthetic: true });
  else if (boundaries[0].index > 0) boundaries.unshift({ index: 0, level: 0, title: 'Preamble', synthetic: true });
  const stack = [];
  const parsed = boundaries.map((boundary, ordinal) => {
    if (boundary.level > 0) { stack.length = boundary.level - 1; stack[boundary.level - 1] = boundary.title; }
    else stack.length = 0;
    const end = boundaries[ordinal + 1]?.index ?? lines.length;
    const text = normalizeMarkdown(lines.slice(boundary.index, end).join('\n'));
    return { headingPath: boundary.synthetic ? [] : stack.filter(Boolean), ordinal, markdown: text, contentHash: contentHash(text) };
  });
  const used = new Set();
  const exact = hash => previous.filter(item => item.contentHash === hash && !used.has(item.sectionId));
  return parsed.map(section => {
    let candidate = exact(section.contentHash);
    if (candidate.length !== 1) {
      const key = JSON.stringify(section.headingPath);
      candidate = previous.filter(item => JSON.stringify(item.headingPath) === key && item.ordinal === section.ordinal && !used.has(item.sectionId));
    }
    const sectionId = candidate.length === 1 ? candidate[0].sectionId : `${slug(section.headingPath.at(-1) ?? 'preamble')}-${section.contentHash.slice(0, 12)}`;
    used.add(sectionId); return { ...section, sectionId };
  });
}

export function contextDraft(nodeId, markdown, previousSections = []) {
  const normalized = normalizeMarkdown(markdown);
  if (!normalized) return null;
  return { contextId: `context-${nodeId}`, nodeId, markdown: normalized, sections: sectionMarkdown(normalized, previousSections) };
}
