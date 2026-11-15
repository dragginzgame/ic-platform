import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';

for (const selection of ['default', 'self-authentication']) {
  const bytes = readFileSync(new URL(`../target/types-wasm/${selection}/wasm32-unknown-unknown/release/types-consumer.wasm`, import.meta.url));
  const module = new WebAssembly.Module(bytes);
  assert.deepEqual(WebAssembly.Module.imports(module), []);
  assert.equal(new WebAssembly.Instance(module).exports.main(0, 0), 0);
  console.log(JSON.stringify({ selection, admission: 'passed', sha256: createHash('sha256').update(bytes).digest('hex'), node: process.version }));
}
