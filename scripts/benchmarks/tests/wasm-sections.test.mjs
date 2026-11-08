import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

const header = [0, 97, 115, 109, 1, 0, 0, 0];
const moduleBytes = Uint8Array.from([...header, 1, 4, 1, 96, 0, 0, 3, 2, 1, 0, 10, 4, 1, 2, 0, 11, 0, 3, 1, 120, 102]);

function fixture(run) {
  const root = mkdtempSync('target/wasm-sections-test.');
  try { run(root); } finally { rmSync(root, { recursive: true, force: true }); }
}
function inspect(file, json = false) {
  return spawnSync('perl', ['scripts/benchmarks/wasm-sections.pl', ...(json ? ['--json'] : []), file], { encoding: 'utf8' });
}

test('one reader retains total, payload, framing and multi-byte lengths', () => fixture(root => {
  const file = join(root, 'module.wasm');
  writeFileSync(file, moduleBytes);
  assert.doesNotThrow(() => new WebAssembly.Module(moduleBytes));
  const tsv = inspect(file);
  assert.equal(tsv.status, 0, tsv.stderr);
  assert.equal(tsv.stdout, '29\t4\t3');
  const result = inspect(file, true);
  assert.equal(result.status, 0, result.stderr);
  const report = JSON.parse(result.stdout);
  assert.equal(report.totalBytes, 29);
  assert.equal(report.codePayloadBytes, 4);
  assert.equal(report.codeFramedBytes, 6);
  const stdin = spawnSync('perl', ['scripts/benchmarks/wasm-sections.pl', '--json', '-'], { input: moduleBytes, encoding: 'utf8' });
  assert.equal(stdin.status, 0, stdin.stderr);
  assert.deepEqual(JSON.parse(stdin.stdout), report);
  assert.deepEqual(report.sections.map(s => [s.id, s.payloadBytes, s.framedBytes]), [[1, 4, 6], [3, 2, 4], [10, 4, 6], [0, 3, 5]]);
  writeFileSync(file, Uint8Array.from([...header, 0, 128, 1, ...new Array(128).fill(0)]));
  const large = JSON.parse(inspect(file, true).stdout);
  assert.equal(large.sections[0].framedBytes, 131);
}));

test('malformed framing refuses instead of returning plausible size measurements', () => fixture(root => {
  const file = join(root, 'invalid.wasm');
  for (const bytes of [[0], [...header.slice(0, 4), 2, 0, 0, 0], [...header, 10],
    [...header, 10, 1], [...header, 10, 128, 128, 128, 128, 16], [...header, 10, 128, 128, 128, 128, 128]]) {
    writeFileSync(file, Uint8Array.from(bytes));
    const result = inspect(file, true);
    assert.notEqual(result.status, 0);
    assert.equal(result.stdout, '');
  }
}));

test('Canic report preserves contract equality and uses the same code-size meaning', () => fixture(root => {
  for (const arm of ['before', 'after']) {
    mkdirSync(join(root, arm));
    writeFileSync(join(root, arm, 'Cargo.lock'), 'same');
    for (const role of ['issuer', 'receiver']) {
      writeFileSync(join(root, arm, role + '.wasm'), moduleBytes);
      for (const ext of ['did', 'wire.json', 'wire.ts']) writeFileSync(join(root, arm, role + '.' + ext), 'same');
    }
  }
  const run = (env = process.env) => spawnSync(process.execPath, ['scripts/benchmarks/canic-wasm/report.cjs', root, 'before', 'after'], { encoding: 'utf8', env });
  let result = run();
  assert.equal(result.status, 0, result.stderr);
  const reports = JSON.parse(readFileSync(join(root, 'measurements.json'), 'utf8'));
  assert.equal(reports.length, 4);
  assert(reports.every(r => r.totalBytes === 29 && r.codePayloadBytes === 4 && r.codeFramedBytes === 6));
  assert.equal(reports[1].deltaCodePayloadBytes, 0);
  // Replace a path after JS read it, at the Perl boundary. Both consumers must
  // still observe the captured bytes, rather than mixing two file generations.
  const bin = join(root, 'bin');
  mkdirSync(bin);
  writeFileSync(join(bin, 'perl'), '#!/bin/sh\nprintf broken > "$WASM_REPLACE_PATH"\nexec "$WASM_REAL_PERL" "$@"\n', { mode: 0o755 });
  const replaced = join(root, 'before', 'issuer.wasm');
  result = run({ ...process.env, PATH: `${bin}:${process.env.PATH}`,
    WASM_REPLACE_PATH: replaced,
    WASM_REAL_PERL: spawnSync('perl', ['-e', 'print $^X'], { encoding: 'utf8' }).stdout });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(readFileSync(replaced, 'utf8'), 'broken');
  writeFileSync(replaced, moduleBytes);
  writeFileSync(join(root, 'after', 'issuer.did'), 'changed');
  result = run();
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /changed contractHashes/);
}));
