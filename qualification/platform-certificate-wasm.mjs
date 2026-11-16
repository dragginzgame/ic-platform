// Standalone verifier execution; this does not model IC instruction costs.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';

for (const binary of ['certificate-admission', 'mainnet-admission']) {
  const bytes = readFileSync(new URL(`../target/platform-certificate-wasm/wasm32-unknown-unknown/release/${binary}.wasm`, import.meta.url));
  const module = new WebAssembly.Module(bytes);
  assert.deepEqual(WebAssembly.Module.imports(module), []);
  const instance = new WebAssembly.Instance(module);
  assert.equal(instance.exports.main(0, 0), 0);
  console.log(JSON.stringify({
    binary,
    admission: 'passed',
    node: process.version,
    v8: process.versions.v8,
    sha256: createHash('sha256').update(bytes).digest('hex'),
  }, null, 2));
}
