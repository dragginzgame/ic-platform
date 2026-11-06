import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { spawnSync } from 'node:child_process';

test('size comparison rejects growth and incomparable inputs', () => {
  mkdirSync('target', { recursive: true });
  const root = mkdtempSync(resolve('target/wasm-comparison-test.'));
  try {
    for (const arm of ['before', 'after']) {
      mkdirSync(join(root, arm, 'src'), { recursive: true });
      for (const file of ['rustc-version.txt', 'cargo-version.txt', 'optimizer-version.txt', 'cpubits.txt', 'src/lib.rs', 'Cargo.lock']) {
        writeFileSync(join(root, arm, file), 'same');
      }
    }
    const header = 'recipe\tcapability\traw_bytes\tcode_bytes\tcustom_bytes\toptimized_bytes\toptimized_code_bytes\toptimized_custom_bytes\n';
    const table = (arm, row) => writeFileSync(join(root, arm, 'sizes.tsv'), header + row + '\n');
    const run = (allowance = '0') => spawnSync(process.execPath,
      ['scripts/benchmarks/compare-wasm-sizes.mjs', join(root, 'before'), join(root, 'after'), allowance], { encoding: 'utf8' });
    table('before', '3-fat-1\tverification\t100\t90\t0\t80\t70\t0');
    table('after', '3-fat-1\tverification\t100\t90\t0\t80\t70\t0');
    assert.equal(run().status, 0);
    table('after', '3-fat-1\tverification\t101\t91\t0\t81\t71\t0');
    assert.equal(run().status, 1);
    assert.equal(run('1').status, 0);
    table('after', '3-fat-1\truntime\t100\t90\t0\t80\t70\t0');
    assert.match(run().stderr, /selections differ/);
    table('after', '3-fat-1\tverification\tNaN\t90\t0\t80\t70\t0');
    assert.match(run().stderr, /Invalid/);
    writeFileSync(join(root, 'after', 'Cargo.lock'), '\n[[package]]\nname = "supplier"\nsource = "registry+x"\n');
    assert.match(run().stderr, /dependency locks differ/);
    writeFileSync(join(root, 'after', 'rustc-version.txt'), 'different');
    assert.match(run().stderr, /Incomparable inputs/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('regression command rejects invalid allowances before resolving or building a baseline', () => {
  for (const value of ['-1', 'NaN', '1.5', '9007199254740992']) {
    const result = spawnSync('bash', ['scripts/benchmarks/check-wasm-size-regression.sh', 'missing-ref', value], { encoding: 'utf8' });
    assert.equal(result.error, undefined);
    assert.equal(result.status, 2);
    assert.match(result.stderr, /MAX_GROWTH_BYTES/);
    assert.doesNotMatch(result.stderr, /Wasm regression evidence/);
  }
});
