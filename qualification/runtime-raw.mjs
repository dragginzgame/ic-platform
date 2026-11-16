// Independently check the raw ABI, copy ranges and forwarded diagnostic bytes.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
const wasm=readFileSync(new URL('../target/runtime-raw/wasm32-unknown-unknown/release/runtime_exports.wasm',import.meta.url));
const module=new WebAssembly.Module(wasm);
const names=['msg_arg_data_size','msg_arg_data_copy','msg_caller_size','msg_caller_copy','is_controller','msg_reply_data_append','msg_reply','trap','performance_counter','stable64_size','stable64_grow','stable64_read','stable64_write','canister_liquid_cycle_balance128','cycles_burn128','cost_http_request','in_replicated_execution'];
assert.deepEqual(WebAssembly.Module.imports(module).map(i=>`${i.module}/${i.kind}/${i.name}`).sort(), names.map(n=>`ic0/function/${n}`).sort());
function run(data,method='canister_update probe',caller=[4]) {
  let instance,done=false,counter=0n;
  const events=[],reply=[];
  const memory=(p,n)=>new Uint8Array(instance.exports.memory.buffer,p>>>0,n>>>0);
  function copy(name,source,p,offset,size) {
    offset>>>=0; size>>>=0;
    events.push([name,offset,size]);
    if(offset+size>source.length) throw new Error('source range');
    const target=memory(p,size);
    assert.deepEqual([...target],Array(size).fill(0));
    target.set(source.slice(offset,offset+size));
  }
  instance=new WebAssembly.Instance(module,{ic0:{
    in_replicated_execution(){return method.startsWith('canister_query') ? 0 : 1;},
    msg_arg_data_size(){return data.length;},
    msg_arg_data_copy(p,o,n){copy('args',data,p,o,n);},
    msg_caller_size(){return caller.length;},
    msg_caller_copy(p,o,n){copy('caller',caller,p,o,n);},
    is_controller(p,n){assert.deepEqual([...memory(p,n)],caller);return 1;},
    msg_reply_data_append(p,n){if(done)throw new Error('already replied');events.push(['append',n]);reply.push(...memory(p,n));},
    msg_reply(){if(done)throw new Error('already replied');done=true;events.push(['reply']);},
    trap(p,n){events.push(['trap',[...memory(p,n)]]);throw new Error('trap');},
    stable64_size(){return 2n;},
    stable64_grow(n){return n===-1n ? -1n : 2n;},
    stable64_read(){throw new Error('unselected stable read');},
    stable64_write(){throw new Error('unselected stable write');},
    canister_liquid_cycle_balance128(){throw new Error('unselected balance');},
    cycles_burn128(){throw new Error('unselected burn');},
    cost_http_request(){throw new Error('unselected cost');},
    performance_counter(kind){assert.equal(kind,0);return ++counter;},
  }});
  let failure;
  try {assert.equal(instance.exports[method](),undefined);}catch(e){failure=e.message;}
  return {events,reply,failure};
}
assert.deepEqual(run([]),{events:[['reply']],reply:[],failure:undefined});
assert.deepEqual(run([0,1,128,255,0]),{events:[['args',0,1],['args',1,4],['args',5,0],['append',2],['append',0],['append',2],['reply']],reply:[1,128,255,0],failure:undefined});
for(const [mode,expected] of [[3,['args',1,1]],[4,['args',4294967295,0]],[5,['caller',1,1]],[6,['caller',4294967295,0]]]){
  const result=run([mode]);assert.equal(result.failure,'source range');assert.deepEqual(result.events.at(-1),expected);
}
for(const mode of [7,8]) assert.equal(run([mode]).failure,'already replied');
const trapped=run([2]);assert.equal(trapped.failure,'trap');assert.deepEqual(trapped.events.at(-1),['trap',[...Buffer.from('rollback diagnostic\0bytes')]]);
for(const caller of [[],[4],Array(29).fill(255)]) {
  const result=run([],'canister_query observe',caller);assert.equal(result.failure,undefined);
  assert.deepEqual(result.reply.slice(0,2+caller.length),[0,1,...caller]);
  assert.equal(Buffer.from(result.reply).readBigUInt64LE(result.reply.length-16),1n);
  assert.equal(Buffer.from(result.reply).readBigUInt64LE(result.reply.length-8),2n);
}
assert.deepEqual(run([], 'canister_query context').reply, [0,0]);
assert.deepEqual(run([], 'canister_update context_update').reply, [1,1]);
console.log(JSON.stringify({admission:'passed',imports:names.length,cases:14,sha256:createHash('sha256').update(wasm).digest('hex')}));
