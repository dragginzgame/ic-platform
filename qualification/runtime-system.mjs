// Inspect exact IC0 memory/call ordering independently of PocketIC semantics.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
const wasm = readFileSync(new URL('../target/runtime-rollback/wasm32-unknown-unknown/release/runtime_rollback.wasm', import.meta.url));
const module = new WebAssembly.Module(wasm);
assert(!WebAssembly.Module.exports(module).some(({name}) => name === 'canister_query registry_size'));
function probe({caller = [4], own = [1, 2], certificate = [8, 9], certificateSize = certificate?.length, method = 'canister_query system_observe', traps = false} = {}) {
  let instance;
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
    time() { return 0x123456789abcdef0n; },
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
  instance = new WebAssembly.Instance(module, {ic0});
  assert.equal(instance.exports[method].length, 0);
  if (traps) assert.throws(() => instance.exports[method](), WebAssembly.RuntimeError);
  else assert.equal(instance.exports[method](), undefined);
  return {events, reply};
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
console.log(JSON.stringify({admission:'passed',cases:10,sha256:createHash('sha256').update(wasm).digest('hex')}));
