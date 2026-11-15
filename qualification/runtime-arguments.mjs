// No replica simulation: verify actual IC0 import order and bounded copies.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';

const bytes = readFileSync(new URL('../target/types-wasm/runtime-ic0/wasm32-unknown-unknown/release/runtime-arguments.wasm', import.meta.url));
const module = new WebAssembly.Module(bytes);
assert.deepEqual(WebAssembly.Module.imports(module).map(x => `${x.module}/${x.kind}/${x.name}`).sort(), ['ic0/function/msg_arg_data_copy','ic0/function/msg_arg_data_size']);
const sizes = [0, 3, 3, 1, 0xffffffff, 2, 3, 1, 2];
const data = new Map([[1, [0, 128, 255]], [5, [7, 8]], [7, [2]], [8, [0, 255]]]);
let index = -1, instance;
const calls = [];
instance = new WebAssembly.Instance(module, {ic0: {
  msg_arg_data_size() {
    index++;
    assert.ok(index < sizes.length);
    calls.push(`size:${index}`);
    return sizes[index] | 0;
  },
  msg_arg_data_copy(dst, offset, size) {
    assert.ok(data.has(index), 'empty or refused input must not be copied');
    assert.equal(offset, 0);
    assert.equal(size >>> 0, sizes[index]);
    const output = new Uint8Array(instance.exports.memory.buffer, dst >>> 0, size >>> 0);
    assert.deepEqual([...output], Array(size).fill(0));
    output.set(data.get(index));
    calls.push(`copy:${index}`);
  },
}});
assert.equal(instance.exports.main(0,0),0);
assert.deepEqual(calls, ['size:0','size:1','copy:1','size:2','size:3','size:4','size:5','copy:5','size:6','size:7','copy:7','size:8','copy:8']);
console.log(JSON.stringify({admission:'passed',cases:10,copies:4,sha256:createHash('sha256').update(bytes).digest('hex'),node:process.version}));
