import assert from 'node:assert/strict';
import * as fs from 'node:fs';
import { createRequire } from 'node:module';
import { join, resolve, dirname } from 'node:path';
import { runInNewContext } from 'node:vm';
import test from 'node:test';

const require = createRequire(import.meta.url);
const capture = resolve('scripts/benchmarks/canic-wasm/capture.cjs');

// Control the copy boundary deterministically; no sleeps or real concurrent Git.
function runCapture(change) {
  const root = fs.mkdtempSync('target/capture-consistency.');
  const input = resolve(root, 'input'), output = resolve(root, 'output');
  fs.mkdirSync(input);
  fs.writeFileSync(join(input, 'source.txt'), 'original');
  try {
    let headReads = 0;
    const git = { execFileSync(command, args) {
      assert.equal(command, 'git');
      if (args[0] === 'ls-files') return Buffer.from(fs.readdirSync(input).sort().join('\0') + '\0');
      if (args[0] === 'rev-parse') return Buffer.from((change === 'head' && headReads++ > 0 ? 'b' : 'a').repeat(40) + '\n');
      if (args[0] === 'status') return Buffer.from('');
      throw Error('unexpected Git observation');
    } };
    const controlledFs = { ...fs, copyFileSync(from, to) {
      if (change === 'copy') fs.writeFileSync(from, 'transient change');
      fs.copyFileSync(from, to);
      if (change === 'copy') fs.writeFileSync(from, 'original');
      if (change === 'addition') fs.writeFileSync(join(input, 'added.txt'), 'new input');
    } };
    const run = () => runInNewContext(fs.readFileSync(capture, 'utf8'), {
      require(name) { return name === 'fs' ? controlledFs : name === 'child_process' ? git : require(name); },
      __dirname: dirname(capture),
      process: { argv: ['node', capture, input, output], cwd: () => process.cwd() },
      console: { log() {} },
    });
    if (change === 'copy') assert.throws(run, /Copied source differs/);
    else if (change === 'addition') assert.throws(run, /Canic changed during capture/);
    else if (change === 'head') assert.throws(run, /Git identity changed/);
    else {
      assert.doesNotThrow(run);
      assert.equal(fs.readFileSync(join(output, 'source/source.txt'), 'utf8'), 'original');
      assert(fs.existsSync(join(output, 'source-inventory.json')));
    }
    if (change) assert(!fs.existsSync(join(output, 'source-inventory.json')));
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
}

test('capture records unchanged copied bytes', () => runCapture());
test('capture refuses a changed copy even when the original is restored', () => runCapture('copy'));
test('capture refuses additions outside its initial roster', () => runCapture('addition'));
test('capture refuses a changed Git identity with unchanged file bytes', () => runCapture('head'));
