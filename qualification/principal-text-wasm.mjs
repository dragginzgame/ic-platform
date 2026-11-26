import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';

function call(module, text) {
  const input = Buffer.from(text);
  const chunks = [];
  let replied = false;
  const instance = new WebAssembly.Instance(module, {
    ic0: {
      msg_arg_data_size: () => input.length,
      msg_arg_data_copy(destination, offset, length) {
        new Uint8Array(instance.exports.memory.buffer, destination, length)
          .set(input.subarray(offset, offset + length));
      },
      msg_reply_data_append(source, length) {
        chunks.push(Buffer.from(new Uint8Array(instance.exports.memory.buffer, source, length)));
      },
      msg_reply() {
        replied = true;
      },
    },
  });
  instance.exports['canister_query principal_text']();
  assert.ok(replied);
  return Buffer.concat(chunks).toString('utf8');
}

function codeSectionBytes(bytes) {
  // WebAssembly.Module has already validated the sections and their lengths.
  let offset = 8;
  while (offset < bytes.length) {
    const section = bytes[offset++];
    let length = 0;
    let shift = 0;
    let byte;
    do {
      byte = bytes[offset++];
      length += (byte & 0x7f) * 2 ** shift;
      shift += 7;
    } while (byte & 0x80);
    if (section === 10) return length;
    offset += length;
  }
  assert.fail('missing Wasm code section');
}

function measure(path) {
  const bytes = readFileSync(path);
  const module = new WebAssembly.Module(bytes);
  const maximalPrincipal = 'bnkmk-jqaaa-aaaaa-aaaaa-aaaaa-aaaaa-aaaaa-aaaaa-aaaaa-aaaaa-aaa';
  for (const [input, canonical] of [
    ['aaaaa-aa', 'aaaaa-aa'],
    ['2VXSX-FAE', '2vxsx-fae'],
    ['rrkah-fqaaa-aaaaa-aaaaq-cai', 'rrkah-fqaaa-aaaaa-aaaaq-cai'],
    [maximalPrincipal, maximalPrincipal],
  ]) {
    assert.equal(call(module, input), canonical);
  }
  for (const input of ['aaaaaaa', 'aaaaa-ab', 'baaaa-aa', 'a'.repeat(64)]) {
    assert.throws(() => call(module, input), WebAssembly.RuntimeError, input);
  }
  return {
    moduleBytes: bytes.length,
    codeSectionBytes: codeSectionBytes(bytes),
    sha256: createHash('sha256').update(bytes).digest('hex'),
  };
}

const [before, after] = process.argv.slice(2);
assert.equal(process.argv.length, 4, 'expected paths to the before and after Wasm modules');
console.log(JSON.stringify({ node: process.version, before: measure(before), after: measure(after) }, null, 2));
