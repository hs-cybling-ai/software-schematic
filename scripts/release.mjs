#!/usr/bin/env node

import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync, copyFileSync, chmodSync, readdirSync, utimesSync } from 'node:fs';
import { basename, dirname, join, resolve } from 'node:path';
import { tmpdir } from 'node:os';
import { spawnSync } from 'node:child_process';

const root = resolve(import.meta.dirname, '..');
const cargoManifest = join(root, 'software-schematic-cli', 'Cargo.toml');
const legalFiles = ['LICENSE', 'NOTICE', 'THIRD_PARTY_NOTICES.md'];
const targets = {
  'aarch64-apple-darwin': { archive: 'tar.gz', executable: 'ss' },
  'x86_64-apple-darwin': { archive: 'tar.gz', executable: 'ss' },
  'x86_64-pc-windows-msvc': { archive: 'zip', executable: 'ss.exe' },
};

function fail(message) {
  console.error(message);
  process.exit(1);
}

function argsOf(values) {
  const result = { _: [] };
  for (let index = 0; index < values.length; index += 1) {
    const value = values[index];
    if (!value.startsWith('--')) result._.push(value);
    else if (value === '--github-output') result.githubOutput = true;
    else result[value.slice(2)] = values[++index];
  }
  return result;
}

function cargoVersion() {
  const text = readFileSync(cargoManifest, 'utf8');
  const packageSection = text.match(/\[package\]([\s\S]*?)(?:\n\[|$)/)?.[1];
  const version = packageSection?.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
  if (!version) fail('Unable to read package.version from software-schematic-cli/Cargo.toml');
  return version;
}

export function releaseMetadata(ref) {
  const packageVersion = cargoVersion();
  const releaseRef = ref || `v${packageVersion}`;
  const match = releaseRef.match(/^v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-(alpha|beta|rc)\.(0|[1-9]\d*))?$/);
  if (!match) fail(`Release ref must match vMAJOR.MINOR.PATCH or vMAJOR.MINOR.PATCH-(alpha|beta|rc).N: ${releaseRef}`);
  const version = `${match[1]}.${match[2]}.${match[3]}`;
  if (version !== packageVersion) fail(`Release ref ${releaseRef} does not match Cargo package version ${packageVersion}`);
  return { ref: releaseRef, version, prerelease: Boolean(match[4]), packageVersion };
}

function sha256(path) {
  return createHash('sha256').update(readFileSync(path)).digest('hex');
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { stdio: 'inherit', ...options });
  if (result.status !== 0) fail(`Command failed: ${command} ${args.join(' ')}`);
}

function archiveName(version, target) {
  const config = targets[target] || fail(`Unsupported release target: ${target}`);
  return `software-schematic-v${version}-${target}.${config.archive}`;
}

function listArchive(path) {
  const result = spawnSync('tar', ['-tf', basename(path)], { cwd: dirname(path), encoding: 'utf8' });
  if (result.status !== 0) fail(`Unable to inspect archive: ${path}`);
  return result.stdout.split(/\r?\n/).filter((entry) => entry && !entry.endsWith('/')).map((entry) => entry.replace(/^\.\//, ''));
}

function packageTarget(options) {
  const metadata = releaseMetadata(options.ref);
  const target = options.target || fail('--target is required');
  const config = targets[target] || fail(`Unsupported release target: ${target}`);
  const binary = resolve(options.binary || fail('--binary is required'));
  if (!existsSync(binary)) fail(`Release executable does not exist: ${binary}`);
  const outputDir = resolve(options['output-dir'] || 'release-dist');
  mkdirSync(outputDir, { recursive: true });
  const name = archiveName(metadata.version, target);
  const archive = join(outputDir, name);
  const targetMetadata = join(outputDir, `${name}.json`);
  if (existsSync(archive) || existsSync(targetMetadata)) fail(`Refusing to overwrite release output: ${name}`);

  const temporary = mkdtempSync(join(tmpdir(), 'software-schematic-package-'));
  const directoryName = name.replace(/\.(tar\.gz|zip)$/, '');
  const stage = join(temporary, directoryName);
  mkdirSync(stage);
  try {
    copyFileSync(binary, join(stage, config.executable));
    if (config.executable === 'ss') chmodSync(join(stage, config.executable), 0o755);
    for (const legal of legalFiles) copyFileSync(join(root, legal), join(stage, legal));
    const archiveTime = new Date('1980-01-01T00:00:00Z');
    for (const entry of [config.executable, ...legalFiles]) utimesSync(join(stage, entry), archiveTime, archiveTime);
    utimesSync(stage, archiveTime, archiveTime);
    const temporaryArchive = join(temporary, name);
    if (config.archive === 'zip') run('tar', ['-a', '-cf', name, directoryName], { cwd: temporary });
    else run('tar', ['-czf', name, directoryName], { cwd: temporary });
    copyFileSync(temporaryArchive, archive);
    const record = { ...metadata, target, executable: config.executable, archive: name, sha256: sha256(archive) };
    writeFileSync(targetMetadata, `${JSON.stringify(record, null, 2)}\n`);
    process.stdout.write(`${JSON.stringify(record)}\n`);
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
}

function expectedTargets(options) {
  const selected = options.targets ? options.targets.split(',') : Object.keys(targets);
  for (const target of selected) if (!targets[target]) fail(`Unsupported release target: ${target}`);
  return selected;
}

function finalize(options) {
  const metadata = releaseMetadata(options.ref);
  const inputDir = resolve(options['input-dir'] || fail('--input-dir is required'));
  const outputDir = resolve(options['output-dir'] || inputDir);
  mkdirSync(outputDir, { recursive: true });
  const manifestPath = join(outputDir, 'release-manifest.json');
  const checksumsPath = join(outputDir, 'SHA256SUMS');
  if (existsSync(manifestPath) || existsSync(checksumsPath)) fail('Refusing to overwrite release manifest or checksums');
  const artifacts = expectedTargets(options).map((target) => {
    const name = archiveName(metadata.version, target);
    const path = join(inputDir, name);
    if (!existsSync(path)) fail(`Missing release archive: ${name}`);
    return { target, file: name, sha256: sha256(path), size: readFileSync(path).byteLength };
  });
  const expectedArchives = new Set(artifacts.map(({ file }) => file));
  const unexpected = readdirSync(inputDir).filter((name) => /\.(zip|tar\.gz)$/.test(name) && !expectedArchives.has(name));
  if (unexpected.length) fail(`Unexpected release archives: ${unexpected.join(', ')}`);
  writeFileSync(manifestPath, `${JSON.stringify({ ref: metadata.ref, version: metadata.version, prerelease: metadata.prerelease, artifacts }, null, 2)}\n`);
  writeFileSync(checksumsPath, `${artifacts.map(({ sha256: digest, file }) => `${digest}  ${file}`).join('\n')}\n`);
}

function validate(options) {
  const metadata = releaseMetadata(options.ref);
  const inputDir = resolve(options['input-dir'] || fail('--input-dir is required'));
  const manifest = JSON.parse(readFileSync(join(inputDir, 'release-manifest.json'), 'utf8'));
  if (manifest.ref !== metadata.ref || manifest.version !== metadata.version) fail('Release manifest version does not match release ref');
  const checksumLines = readFileSync(join(inputDir, 'SHA256SUMS'), 'utf8').trim().split(/\r?\n/);
  const checksumMap = new Map(checksumLines.map((line) => {
    const match = line.match(/^([a-f0-9]{64})  (.+)$/);
    if (!match) fail(`Malformed checksum line: ${line}`);
    return [match[2], match[1]];
  }));
  const selectedTargets = expectedTargets(options);
  if (manifest.artifacts.length !== selectedTargets.length) fail('Release manifest has an unexpected artifact count');
  for (const target of selectedTargets) {
    const name = archiveName(metadata.version, target);
    const path = join(inputDir, name);
    const digest = sha256(path);
    if (checksumMap.get(name) !== digest) fail(`Checksum mismatch: ${name}`);
    const directoryName = name.replace(/\.(tar\.gz|zip)$/, '');
    const entries = new Set(listArchive(path));
    const expected = [targets[target].executable, ...legalFiles].map((entry) => `${directoryName}/${entry}`);
    if (entries.size !== expected.length || expected.some((entry) => !entries.has(entry))) fail(`Unexpected archive contents: ${name}`);
  }
}

function emitMetadata(options) {
  const metadata = releaseMetadata(options.ref);
  process.stdout.write(`${JSON.stringify(metadata)}\n`);
  const output = process.env.GITHUB_OUTPUT;
  if (options.githubOutput) {
    if (!output) fail('GITHUB_OUTPUT is required with --github-output');
    writeFileSync(output, Object.entries(metadata).map(([key, value]) => `${key}=${value}`).join('\n') + '\n', { flag: 'a' });
  }
}

const options = argsOf(process.argv.slice(2));
const command = options._[0];
if (command === 'metadata') emitMetadata(options);
else if (command === 'package') packageTarget(options);
else if (command === 'finalize') finalize(options);
else if (command === 'validate') validate(options);
else fail('Usage: release.mjs <metadata|package|finalize|validate> [options]');
