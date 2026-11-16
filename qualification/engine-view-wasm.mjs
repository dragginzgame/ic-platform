// Local probe evidence only: no optimizer, deployment adapter or IC cost model.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';

assert.ok(process.argv.length <= 3, 'expected at most one probe selection');
const probe = process.argv[2] ?? 'size';
assert.ok(['size', 'admission', 'registry', 'marketplace', 'raw-rand', 'management-chunks'].includes(probe), 'unknown probe');
const binary = { size: 'engine-view-wasm', admission: 'engine-view-admission', registry: 'registry-admission', marketplace: 'marketplace-admission', 'raw-rand': 'raw-rand-admission', 'management-chunks': 'management-chunks-admission' }[probe];

function measure(selection) {
  const url = new URL(`../target/engine-view-wasm/${selection}/wasm32-unknown-unknown/release/${binary}.wasm`, import.meta.url);
  const bytes = readFileSync(url);
  // Validate the entire module before scanning its outer section lengths.
  const module = new WebAssembly.Module(bytes);
  assert.deepEqual(WebAssembly.Module.imports(module), []);
  const instance = new WebAssembly.Instance(module);
  assert.equal(instance.exports.main(0, 0), 0, `${selection}: fixture failed`);
  const sha256 = createHash('sha256').update(bytes).digest('hex');
  if (probe !== 'size') return { sha256, admission: 'passed' };

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
    sha256,
  };
}

const standalone = ['registry', 'marketplace', 'raw-rand', 'management-chunks'].includes(probe);
const candidate = measure(standalone ? probe : 'candidate');
// No launch-derived size budget has been selected. Report evidence without
// converting a relative size change into an acceptance decision.
const size_acceptance = probe === 'size' ? 'undecided' : undefined;
console.log(JSON.stringify({ probe, node: process.version, v8: process.versions.v8, candidate, size_acceptance }, null, 2));
