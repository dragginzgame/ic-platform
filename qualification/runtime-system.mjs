// Inspect exact IC0 memory/call ordering independently of PocketIC semantics.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
const wasm = readFileSync(new URL('../target/runtime-rollback/wasm32-unknown-unknown/release/runtime_rollback.wasm', import.meta.url));
const module = new WebAssembly.Module(wasm);
assert(!WebAssembly.Module.exports(module).some(({name}) => name === 'canister_query registry_size'));
function probe({clock = 0x123456789abcdef0n, caller = [4], own = [1, 2], certificate = [8, 9], certificateSize = certificate?.length, method = 'canister_query system_observe', traps = false} = {}) {
  let instance, clockReads = 0;
  const events = [], reply = [];
  const copy = (name, bytes) => (pointer, offset, size) => {
    events.push(name);
    assert.equal(offset, 0); assert.equal(size, bytes.length);
    const destination = new Uint8Array(instance.exports.memory.buffer, pointer >>> 0, size);
    assert.deepEqual([...destination], Array(size).fill(0));
    destination.set(bytes);
  };
  const ic0 = Object.fromEntries(WebAssembly.Module.imports(module).map(({module: owner, name, kind}) => {
    assert.equal(owner, 'ic0'); assert.equal(kind, 'function');
    return [name, () => { throw new Error(`unexpected import: ${name}`); }];
  }));
  Object.assign(ic0, {
    msg_arg_data_size() { events.push('args-size'); return 0; },
    msg_caller_size() { events.push('caller-size'); return caller.length; },
    msg_caller_copy: copy('caller-copy', caller),
    canister_self_size() { events.push('own-size'); return own.length; },
    canister_self_copy: copy('own-copy', own),
    stable64_size() { return 0n; }, // This probe installs no stable dispatch record.
    time() { clockReads++; return clock; },
    is_controller(pointer, size) {
      assert.deepEqual([...new Uint8Array(instance.exports.memory.buffer, pointer >>> 0, size)], caller);
      return 1;
    },
    data_certificate_present() { events.push('present'); return certificate === null ? 0 : 1; },
    data_certificate_size() { events.push('cert-size'); return certificateSize; },
    data_certificate_copy: copy('cert-copy', certificate),
    msg_reply_data_append(pointer, size) { reply.push(...new Uint8Array(instance.exports.memory.buffer, pointer >>> 0, size)); events.push('append'); },
    msg_reply() { events.push('reply'); },
    certified_data_set(pointer, size) {
      assert.equal(size, 32);
      const expected = method === 'canister_init' ? 7 : method === 'canister_pre_upgrade' ? 8 : 9;
      assert.deepEqual([...new Uint8Array(instance.exports.memory.buffer, pointer >>> 0, size)], Array(32).fill(expected));
      events.push('root');
    },
  });
  if (method === 'canister_update randomness') {
    // A zero cost and balance permit a zero-attachment submission. The default
    // import handler rejects cycle attachment; the event list also detects retries.
    Object.assign(ic0, {
      in_replicated_execution: () => 1,
      cost_call(methodSize, argsSize, dst) {
        assert.equal(methodSize, 8n);
        assert.equal(argsSize, 6n);
        new Uint8Array(instance.exports.memory.buffer, dst >>> 0, 16).fill(0);
      },
      canister_liquid_cycle_balance128(dst) {
        new Uint8Array(instance.exports.memory.buffer, dst >>> 0, 16).fill(0);
      },
      call_new(_target, targetSize, methodPtr, methodSize) {
        assert.equal(targetSize, 0);
        const bytes = new Uint8Array(instance.exports.memory.buffer, methodPtr >>> 0, methodSize);
        assert.equal(new TextDecoder().decode(bytes), 'raw_rand');
        events.push('new');
      },
      call_on_cleanup() {},
      call_with_best_effort_response(seconds) { assert.equal(seconds, 10); events.push('wait'); },
      call_data_append(ptr, size) {
        const bytes = new Uint8Array(instance.exports.memory.buffer, ptr >>> 0, size);
        assert.deepEqual([...bytes], [68, 73, 68, 76, 0, 0]);
        events.push('request');
      },
      call_perform() { events.push('perform'); return 17; },
    });
  }
  instance = new WebAssembly.Instance(module, {ic0});
  assert.equal(instance.exports[method].length, 0);
  if (traps) assert.throws(() => instance.exports[method](), WebAssembly.RuntimeError);
  else assert.equal(instance.exports[method](), undefined);
  return {events, reply, clockReads};
}
// system_observe reads now, time, now_millis and now_secs once each. Its
// assertions compare all four views before it replies with the original nanos.
const clocks = [0n, 999_999n, 1_000_000n, 1_000_001n, 999_999_999n,
  1_000_000_000n, 1_000_000_001n, (1n << 64n) - 1n];
for (const clock of clocks) {
  const {reply, clockReads} = probe({clock, caller: [], own: []});
  assert.equal(clockReads, 4);
  assert.equal(Buffer.from(reply).readBigUInt64LE(2), clock);
}
const normal = probe();
assert.deepEqual(normal.events, ['args-size','caller-size','caller-copy','own-size','own-copy','present','cert-size','cert-copy','present','cert-size','cert-copy','present','cert-size','append','reply']);
assert.deepEqual(normal.reply, [2,1,2,1,4,240,222,188,154,120,86,52,18,1,1,8,9]);
assert.deepEqual(probe({certificate:null}).events.slice(-3), ['present','append','reply']);
assert(!probe({caller:[],own:[],certificate:[]}).events.some(e => e.endsWith('-copy')));
assert(probe({caller:Array(29).fill(1), own:Array(29).fill(2)}).events.includes('reply'));
assert.deepEqual(probe({caller:Array(30).fill(1),traps:true}).events, ['args-size','caller-size']);
assert.deepEqual(probe({own:Array(30).fill(1),traps:true}).events, ['args-size','caller-size','caller-copy','own-size']);
const oversized = probe({certificateSize:1048577,traps:true}).events;
assert.equal(oversized.at(-1), 'cert-size'); assert(!oversized.includes('cert-copy'));
for (const method of ['canister_init','canister_pre_upgrade','canister_post_upgrade']) {
  assert.deepEqual(probe({method}).events, method === 'canister_pre_upgrade' ? ['root'] : ['args-size','root']);
}
const refused = probe({method:'canister_update randomness'});
assert.deepEqual(refused.events, ['args-size','new','wait','request','perform','append','reply']);
assert.deepEqual(refused.reply, [17,0,0,0]);
console.log(JSON.stringify({admission:'passed',cases:11 + clocks.length,sha256:createHash('sha256').update(wasm).digest('hex')}));
