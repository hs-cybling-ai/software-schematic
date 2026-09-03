import assert from 'node:assert/strict';
import { chmodSync, copyFileSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import test from 'node:test';

const root = resolve(import.meta.dirname, '..');
const helper = join(root, 'scripts', 'release.mjs');
const version = readFileSync(join(root, 'software-schematic-cli', 'Cargo.toml'), 'utf8').match(/^version\s*=\s*"([^"]+)"/m)[1];
const ref = `v${version}`;

function run(args, expected = 0) {
  const result = spawnSync(process.execPath, [helper, ...args], { cwd: root, encoding: 'utf8' });
  assert.equal(result.status, expected, result.stderr || result.stdout);
  return result;
}

test('accepts matching stable and recognized prerelease refs', () => {
  assert.equal(JSON.parse(run(['metadata', '--ref', ref]).stdout).version, version);
  assert.equal(JSON.parse(run(['metadata', '--ref', `${ref}-rc.1`]).stdout).prerelease, true);
});

test('rejects malformed and mismatched refs', () => {
  assert.match(run(['metadata', '--ref', 'main'], 1).stderr, /must match/);
  assert.match(run(['metadata', '--ref', 'v999.0.0'], 1).stderr, /does not match/);
  assert.match(run(['metadata', '--ref', `${ref}-preview.1`], 1).stderr, /must match/);
});

test('packages every target, finalizes, validates, and refuses overwrite', () => {
  const temporary = mkdtempSync(join(tmpdir(), 'software-schematic-release-test-'));
  try {
    for (const target of ['aarch64-apple-darwin', 'x86_64-apple-darwin', 'x86_64-pc-windows-msvc']) {
      const binary = join(temporary, target.endsWith('windows-msvc') ? 'ss.exe' : `ss-${target}`);
      writeFileSync(binary, '#!/bin/sh\nexit 0\n');
      chmodSync(binary, 0o755);
      const common = ['--ref', ref, '--target', target, '--binary', binary, '--output-dir', temporary];
      run(['package', ...common]);
      assert.match(run(['package', ...common], 1).stderr, /Refusing to overwrite/);
    }
    run(['finalize', '--ref', ref, '--input-dir', temporary]);
    run(['validate', '--ref', ref, '--input-dir', temporary]);
    const manifest = JSON.parse(readFileSync(join(temporary, 'release-manifest.json'), 'utf8'));
    assert.equal(manifest.artifacts.length, 3);
    assert.match(readFileSync(join(temporary, 'SHA256SUMS'), 'utf8'), /^[a-f0-9]{64}  software-schematic-/);
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});

test('workflow encodes the supported matrix and publication guardrails', () => {
  const workflow = readFileSync(join(root, '.github', 'workflows', 'release.yml'), 'utf8');
  for (const value of ['macos-15', 'macos-15-intel', 'windows-2022', 'aarch64-apple-darwin', 'x86_64-apple-darwin', 'x86_64-pc-windows-msvc']) {
    assert.match(workflow, new RegExp(value));
  }
  assert.match(workflow, /if: github\.event_name == 'push'/);
  assert.match(workflow, /contents: write/);
  assert.match(workflow, /attestations: write/);
  assert.match(workflow, /gh release view/);
  assert.match(workflow, /needs: assemble/);
  const actionRefs = [...workflow.matchAll(/uses:\s+[^@\s]+@([^\s]+)/g)].map((match) => match[1]);
  assert.ok(actionRefs.length > 0);
  assert.ok(actionRefs.every((revision) => /^[a-f0-9]{40}$/.test(revision)));
});

test('rejects incomplete, unexpected, and corrupted artifact sets', () => {
  const temporary = mkdtempSync(join(tmpdir(), 'software-schematic-release-failure-'));
  try {
    assert.match(run(['finalize', '--ref', ref, '--input-dir', temporary], 1).stderr, /Missing release archive/);
    const binary = join(temporary, 'ss');
    writeFileSync(binary, '#!/bin/sh\nexit 0\n');
    chmodSync(binary, 0o755);
    const target = 'aarch64-apple-darwin';
    run(['package', '--ref', ref, '--target', target, '--binary', binary, '--output-dir', temporary]);
    copyFileSync(join(temporary, `software-schematic-${ref}-${target}.tar.gz`), join(temporary, 'unexpected.tar.gz'));
    assert.match(run(['finalize', '--ref', ref, '--input-dir', temporary, '--targets', target], 1).stderr, /Unexpected release archives/);
    rmSync(join(temporary, 'unexpected.tar.gz'));
    run(['finalize', '--ref', ref, '--input-dir', temporary, '--targets', target]);
    writeFileSync(join(temporary, `software-schematic-${ref}-${target}.tar.gz`), 'corrupt');
    assert.match(run(['validate', '--ref', ref, '--input-dir', temporary, '--targets', target], 1).stderr, /Checksum mismatch/);
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});
