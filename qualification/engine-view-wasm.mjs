// Local probe evidence only: no optimizer, deployment adapter or IC cost model.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';

assert.equal(process.argv.length, 2, 'this fixed comparison accepts no arguments');

function measure(selection) {
  const url = new URL(`../target/engine-view-wasm/${selection}/wasm32-unknown-unknown/release/engine-view-wasm.wasm`, import.meta.url);
  const bytes = readFileSync(url);
  // Validate the entire module before scanning its outer section lengths.
  const module = new WebAssembly.Module(bytes);
  assert.deepEqual(WebAssembly.Module.imports(module), []);
  const instance = new WebAssembly.Instance(module);
  assert.equal(instance.exports.main(0, 0), 0, `${selection}: fixture failed`);

  let offset = 8;
  let codeBytes;
  while (offset < bytes.length) {
    const section = bytes[offset++];
    let length = 0;
    let multiplier = 1;
    let byte;
    do {
      byte = bytes[offset++];
      length += (byte & 0x7f) * multiplier;
      multiplier *= 128;
    } while (byte & 0x80);
    if (section === 10) {
      assert.equal(codeBytes, undefined);
      codeBytes = length;
    }
    offset += length;
  }
  assert.equal(offset, bytes.length);
  assert.ok(codeBytes > 0);
  return {
    module_bytes: bytes.length,
    code_section_payload_bytes: codeBytes,
    sha256: createHash('sha256').update(bytes).digest('hex'),
  };
}

const published = measure('published');
const candidate = measure('candidate');
const delta = {
  module_bytes: candidate.module_bytes - published.module_bytes,
  code_section_payload_bytes: candidate.code_section_payload_bytes - published.code_section_payload_bytes,
};
// No launch-derived size budget has been selected. Report evidence without
// converting a relative size change into an acceptance decision.
const size_acceptance = 'undecided';
console.log(JSON.stringify({ node: process.version, v8: process.versions.v8, published, candidate, delta, size_acceptance }, null, 2));
