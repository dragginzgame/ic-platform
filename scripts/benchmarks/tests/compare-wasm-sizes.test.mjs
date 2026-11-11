import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, writeFileSync, readFileSync, rmSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { spawnSync } from 'node:child_process';

function fixture(t) {
  mkdirSync('target', { recursive: true });
  const root = mkdtempSync(resolve('target/wasm-comparison-test.'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const put = (arm, file, value) => writeFileSync(join(root, arm, file), value);
  const header = 'recipe\tcapability\traw_bytes\tcode_bytes\tcustom_bytes\toptimized_bytes\toptimized_code_bytes\toptimized_custom_bytes\n';
  const table = (arm, row) => put(arm, 'sizes.tsv', header + row + '\n');
  const metadata = {
    packages: [{ id: 'probe', name: 'probe', version: '0.0.0', source: null },
      { id: 'supplier', name: 'supplier', version: '1.0.0', source: 'registry+x' }],
    resolve: { root: 'probe', nodes: [
      { id: 'probe', features: [], deps: [{ pkg: 'supplier', dep_kinds: [{ kind: null }] }] },
      { id: 'supplier', features: ['alloc'], deps: [] },
    ] },
  };
  const lock = 'version = 4\n\n[[package]]\nname = "probe"\nversion = "0.0.0"\n\n[[package]]\nname = "supplier"\nversion = "1.0.0"\nsource = "registry+x"\nchecksum = "abc"\n';
  for (const arm of ['before', 'after']) {
    mkdirSync(join(root, arm, 'src/nested'), { recursive: true });
    for (const file of ['rustc-version.txt', 'cargo-version.txt', 'optimizer-version.txt', 'cpubits.txt', 'build-settings.txt', 'src/lib.rs', 'src/nested/mod.rs']) put(arm, file, 'same');
    put(arm, 'Cargo.lock', lock);
    put(arm, '3-fat-1-verification.metadata.json', JSON.stringify(metadata));
    table(arm, '3-fat-1\tverification\t100\t90\t0\t80\t70\t0');
  }
  const run = (allowance = '0', mode) => spawnSync(process.execPath,
    ['scripts/benchmarks/compare-wasm-sizes.mjs', join(root, 'before'), join(root, 'after'), allowance, ...(mode ? [mode] : [])], { encoding: 'utf8' });
  const report = () => JSON.parse(readFileSync(join(root, 'after/dependency-changes.json'), 'utf8'));
  return { root, put, table, metadata, lock, run, report };
}

test('size comparison reports absolute bytes and rejects growth, mismatched profiles and malformed tables', t => {
  const { run, table } = fixture(t);
  assert.equal(run().status, 0);
  assert.match(run().stdout, /verification\t100\t100\t0\t90\t90\t0\t80\t80\t0\t70\t70\t0/);
  table('after', '3-fat-1\tverification\t101\t91\t0\t81\t71\t0');
  assert.equal(run().status, 1);
  assert.equal(run('1').status, 0);
  table('after', '3-fat-1\truntime\t100\t90\t0\t80\t70\t0');
  assert.match(run().stderr, /selections differ/);
  table('after', '3-fat-1\tverification\tNaN\t90\t0\t80\t70\t0');
  assert.match(run().stderr, /Invalid/);
  table('after', '3-fat-1\t../../outside\t100\t90\t0\t80\t70\t0');
  assert.match(run().stderr, /Invalid/);
});

test('dependency changes require explicit mode and cannot waive size growth', t => {
  const { run, put, lock, report, table } = fixture(t);
  put('after', 'Cargo.lock', lock.replace('checksum = "abc"', 'checksum = "def"'));
  assert.match(run().stderr, /dependency locks or feature graphs differ/);
  assert.equal(report().lock.removed.length, 1);
  assert.match(report().lock.added[0], /checksum = "def"/);
  assert.equal(run('0', '--allow-dependency-changes').status, 0);
  assert.equal(report().mode, 'deployment');
  assert.match(report().attribution, /not implementation-only/);
  table('after', '3-fat-1\tverification\t100\t90\t0\t80\t71\t0');
  assert.equal(run('0', '--allow-dependency-changes').status, 1);
  assert.equal(run('1', '--allow-dependency-changes').status, 0);
  assert.match(run('0', '--ignore').stderr, /usage:/);
});

test('per-profile normal/build graph detects feature changes and excludes disconnected or dev-only packages', t => {
  const { run, put, metadata, report } = fixture(t);
  metadata.packages.push({ id: 'dev', name: 'unused-or-dev', version: '1', source: 'registry+x' });
  metadata.resolve.nodes.push({ id: 'dev', features: [], deps: [] });
  metadata.resolve.nodes[0].deps.push({ pkg: 'dev', dep_kinds: [{ kind: 'dev' }] });
  put('after', '3-fat-1-verification.metadata.json', JSON.stringify(metadata));
  assert.equal(run().status, 0);
  metadata.resolve.nodes[1].features.push('std');
  put('after', '3-fat-1-verification.metadata.json', JSON.stringify(metadata));
  assert.match(run().stderr, /feature graphs differ/);
  assert.equal(run('0', '--allow-dependency-changes').status, 0);
  assert.deepEqual(report().profiles[0].added[0].features, ['alloc', 'std']);
  metadata.resolve.nodes[0].deps[1].dep_kinds[0].kind = 'build';
  put('after', '3-fat-1-verification.metadata.json', JSON.stringify(metadata));
  assert.equal(run('0', '--allow-dependency-changes').status, 0);
  assert.equal(report().profiles[0].added.length, 2);
  metadata.resolve.nodes.pop();
  put('after', '3-fat-1-verification.metadata.json', JSON.stringify(metadata));
  assert.match(run('0', '--allow-dependency-changes').stderr, /Incomplete dependency graph/);
});

test('deployment mode still refuses compiler, recipe settings and nested probe source drift', t => {
  const { run, put } = fixture(t);
  for (const file of ['rustc-version.txt', 'cargo-version.txt', 'optimizer-version.txt', 'cpubits.txt', 'build-settings.txt', 'src/nested/mod.rs']) {
    put('after', file, 'different');
    assert.match(run('100', '--allow-dependency-changes').stderr, /Incomparable inputs/);
    put('after', file, 'same');
  }
  put('after', 'src/nested/extra.rs', 'new source');
  assert.match(run('100', '--allow-dependency-changes').stderr, /Incomparable inputs/);
});

test('regression command rejects invalid arguments before resolving or building a baseline', () => {
  for (const value of ['-1', 'NaN', '1.5', '9007199254740992']) {
    const result = spawnSync('bash', ['scripts/benchmarks/check-wasm-size-regression.sh', 'missing-ref', value], { encoding: 'utf8' });
    assert.equal(result.status, 2);
    assert.match(result.stderr, /MAX_GROWTH_BYTES/);
    assert.doesNotMatch(result.stderr, /Wasm regression evidence/);
  }
  const result = spawnSync('bash', ['scripts/benchmarks/check-wasm-size-regression.sh', 'missing-ref', '0', '--ignore'], { encoding: 'utf8' });
  assert.equal(result.status, 2);
  assert.match(result.stderr, /Unknown dependency comparison mode/);
});
