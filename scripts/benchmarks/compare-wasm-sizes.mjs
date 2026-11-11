import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const [before, after, allowance = '0', mode, ...extra] = process.argv.slice(2);
if (!before || !after || !/^\d+$/.test(allowance) || !Number.isSafeInteger(Number(allowance)) ||
    extra.length || (mode !== undefined && mode !== '--allow-dependency-changes')) {
  throw Error('usage: compare-wasm-sizes.mjs BEFORE AFTER [MAX_GROWTH_BYTES] [--allow-dependency-changes]');
}
for (const name of ['rustc-version.txt', 'cargo-version.txt', 'optimizer-version.txt', 'cpubits.txt', 'build-settings.txt']) {
  if (!readFileSync(join(before, name)).equals(readFileSync(join(after, name)))) {
    throw Error(`Incomparable inputs: ${name}`);
  }
}
function sources(directory, prefix = 'src') {
  return readdirSync(join(directory, prefix), { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))
    .flatMap(entry => {
      const path = join(prefix, entry.name);
      if (entry.isDirectory()) return sources(directory, path);
      if (!entry.isFile()) throw Error(`Unsupported probe source: ${path}`);
      return [[path, readFileSync(join(directory, path)).toString('hex')]];
    });
}
if (JSON.stringify(sources(before)) !== JSON.stringify(sources(after))) throw Error('Incomparable inputs: probe sources');

function rows(directory) {
  const lines = readFileSync(join(directory, 'sizes.tsv'), 'utf8').trim().split('\n');
  const expected = 'recipe\tcapability\traw_bytes\tcode_bytes\tcustom_bytes\toptimized_bytes\toptimized_code_bytes\toptimized_custom_bytes';
  if (lines.shift() !== expected || !lines.length) throw Error('Invalid size table');
  const result = new Map();
  for (const line of lines) {
    const [recipe, profile, ...sizes] = line.split('\t');
    const key = `${recipe}/${profile}`;
    if (!/^(3-none-16|3-thin-1|3-fat-1|s-fat-1|z-fat-1)$/.test(recipe) || !/^[a-z0-9-]+$/.test(profile) ||
        result.has(key) || sizes.length !== 6 || sizes.some(size => !/^\d+$/.test(size) || !Number.isSafeInteger(Number(size)))) {
      throw Error('Invalid or duplicate size row');
    }
    result.set(key, sizes.map(Number));
  }
  return result;
}
const old = rows(before), current = rows(after);
if (old.size !== current.size || [...old.keys()].some(key => !current.has(key))) throw Error('Profile/recipe selections differ');

// Keep complete Cargo-generated external entries, including checksum and edge
// changes. Do not equate a matching package name/version with identical inputs.
function externalLock(directory) {
  const text = readFileSync(join(directory, 'Cargo.lock'), 'utf8');
  if (!/^version = \d+$/m.test(text) || !text.includes('[[package]]')) throw Error('Invalid Cargo lock evidence');
  return text.split(/^\[\[package\]\]\s*$/m).slice(1)
    .filter(block => /^source = /m.test(block)).map(block => block.trim()).sort();
}
const difference = (left, right) => left.filter(value => !right.includes(value));
function changes(left, right) {
  return { removed: difference(left, right), added: difference(right, left) };
}
function graph(directory, key) {
  const metadata = JSON.parse(readFileSync(join(directory, `${key.replace('/', '-')}.metadata.json`), 'utf8'));
  const packages = new Map(metadata.packages.map(pkg => [pkg.id, pkg]));
  const nodes = new Map(metadata.resolve.nodes.map(node => [node.id, node]));
  const pending = [metadata.resolve.root], seen = new Set(), external = [];
  while (pending.length) {
    const id = pending.pop();
    if (seen.has(id)) continue;
    seen.add(id);
    const pkg = packages.get(id), node = nodes.get(id);
    if (!pkg || !node) throw Error(`Incomplete dependency graph: ${key}`);
    if (pkg.source) external.push(JSON.stringify({
      name: pkg.name, version: pkg.version, source: pkg.source, features: [...node.features].sort(),
    }));
    for (const dep of node.deps) {
      if (dep.dep_kinds.some(kind => kind.kind === null || kind.kind === 'build')) pending.push(dep.pkg);
    }
  }
  return external.sort();
}
const lock = changes(externalLock(before), externalLock(after));
const profiles = [...old.keys()].map(profile => {
  const delta = changes(graph(before, profile), graph(after, profile));
  return { profile, removed: delta.removed.map(JSON.parse), added: delta.added.map(JSON.parse) };
});
const changed = lock.removed.length + lock.added.length + profiles.reduce((n, row) => n + row.removed.length + row.added.length, 0) > 0;
const report = join(after, 'dependency-changes.json');
writeFileSync(report, JSON.stringify({
  mode: mode ? 'deployment' : 'strict',
  attribution: changed ? 'Implementation and dependency changes; not implementation-only' : 'Unchanged external dependency inputs',
  lock, profiles,
}, null, 2) + '\n');
console.error(`Dependency evidence: ${report}`);
console.error(`External lock entries: ${lock.removed.length} removed/changed, ${lock.added.length} added/changed`);
for (const row of profiles) console.error(`${row.profile}: ${row.removed.length} removed/changed, ${row.added.length} added/changed external graph selections`);
if (changed && !mode) throw Error('External dependency locks or feature graphs differ; use --allow-dependency-changes for a deployment comparison');
if (changed) console.error('Size deltas include dependency changes; the same growth threshold still applies.');

console.log('profile\traw_before\traw_after\traw_delta\traw_code_before\traw_code_after\traw_code_delta\toptimized_before\toptimized_after\toptimized_delta\toptimized_code_before\toptimized_code_after\toptimized_code_delta');
let failed = false;
for (const [key, sizes] of current) {
  const previous = old.get(key);
  const deltas = [0, 1, 3, 4].map(index => sizes[index] - previous[index]);
  console.log([key, ...[0, 1, 3, 4].flatMap(index => [previous[index], sizes[index], sizes[index] - previous[index]])].join('\t'));
  if (deltas.some(delta => delta > Number(allowance))) failed = true;
}
if (failed) process.exitCode = 1;
