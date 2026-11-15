// Deterministic ABI host, not a replica or proof of remote execution/rollback.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';

const bytes = readFileSync(new URL('../target/types-wasm/runtime-ic0/wasm32-unknown-unknown/release/types-consumer.wasm', import.meta.url));
const module = new WebAssembly.Module(bytes);
const names = ['cost_call', 'canister_liquid_cycle_balance128', 'call_new', 'call_data_append', 'call_cycles_add128', 'call_perform'];
assert.deepEqual(WebAssembly.Module.imports(module).map(x => `${x.module}/${x.kind}/${x.name}`).sort(), names.map(x => `ic0/function/${x}`).sort());
const maximum = (1n << 128n) - 1n;
const attachment = (1n << 100n) + 7n;
const steps = [];
const add = (name, fields = {}) => steps.push({ name, ...fields });
const cost = (value, method = 4n, args = 3n) => add('cost_call', { value, method, args });
const balance = value => add('canister_liquid_cycle_balance128', { value });
const start = (empty = false) => add('call_new', { target: empty ? [] : [4], method: empty ? [] : [...Buffer.from('mé\0')] });
const data = () => add('call_data_append', { bytes: [0, 128, 255] });
const perform = value => add('call_perform', { value });
for (const value of [0n, (1n << 64n) + 7n, maximum, maximum, 1n]) cost(value);
// One successful nonempty notification, then one empty notification.
cost(13n); balance(13n + attachment); start(); data(); add('call_cycles_add128', { value: attachment }); perform(0);
cost(0n, 0n, 0n); balance(0n); start(true); perform(0);
// Insufficient balance and arithmetic overflow must stop before call_new.
cost(13n); balance(17n);
cost(maximum);
// Preserve both a known refusal and an unknown raw code; never retry.
for (const value of [2, -1]) { cost(13n); balance(13n); start(); data(); perform(value); }
let instance;
let position = 0;
function next(name) {
  const step = steps[position++];
  assert.ok(step, `unexpected ${name}`);
  assert.equal(step.name, name, `unexpected import ordering at ${position}`);
  return step;
}
function read(pointer, length) {
  const offset = pointer >>> 0;
  const size = length >>> 0;
  const memory = instance.exports.memory.buffer;
  assert.ok(offset + size <= memory.byteLength);
  return [...new Uint8Array(memory, offset, size)];
}
function write128(destination, value) {
  const offset = destination >>> 0;
  assert.deepEqual(read(destination, 16), Array(16).fill(0));
  const view = new DataView(instance.exports.memory.buffer);
  view.setBigUint64(offset, value & ((1n << 64n) - 1n), true);
  view.setBigUint64(offset + 8, value >> 64n, true);
}
instance = new WebAssembly.Instance(module, { ic0: {
  cost_call(methodBytes, argumentBytes, destination) {
    const step = next('cost_call');
    assert.equal(methodBytes, step.method);
    assert.equal(argumentBytes, step.args);
    write128(destination, step.value);
  },
  canister_liquid_cycle_balance128(destination) {
    write128(destination, next('canister_liquid_cycle_balance128').value);
  },
  call_new(target, targetLength, method, methodLength, reply, replyEnv, reject, rejectEnv) {
    const step = next('call_new');
    assert.deepEqual(read(target, targetLength), step.target);
    assert.deepEqual(read(method, methodLength), step.method);
    assert.deepEqual([reply, replyEnv, reject, rejectEnv], [-1, -1, -1, -1]);
  },
  call_data_append(source, length) {
    assert.deepEqual(read(source, length), next('call_data_append').bytes);
  },
  call_cycles_add128(high, low) {
    assert.equal((BigInt.asUintN(64, high) << 64n) | BigInt.asUintN(64, low), next('call_cycles_add128').value);
  },
  call_perform() { return next('call_perform').value; },
} });
assert.equal(instance.exports.main(0, 0), 0);
assert.equal(position, steps.length);
console.log(JSON.stringify({ admission: 'passed', imports: names, calls: position, sha256: createHash('sha256').update(bytes).digest('hex'), node: process.version }));
