import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';

assert.ok(process.argv.length <= 3 && [undefined, 'runtime-call'].includes(process.argv[2]), 'expected optional runtime-call selection');
const selections = process.argv[2] === 'runtime-call' ? ['runtime-call'] : ['default', 'self-authentication'];
for (const selection of selections) {
  const bytes = readFileSync(new URL(`../target/types-wasm/${selection}/wasm32-unknown-unknown/release/types-consumer.wasm`, import.meta.url));
  const module = new WebAssembly.Module(bytes);
  assert.deepEqual(WebAssembly.Module.imports(module), []);
  assert.equal(new WebAssembly.Instance(module).exports.main(0, 0), 0);
  console.log(JSON.stringify({ selection, admission: 'passed', sha256: createHash('sha256').update(bytes).digest('hex'), node: process.version }));
}
