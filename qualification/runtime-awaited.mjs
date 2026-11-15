// Executes real Wasm callback functions, but does not simulate replica rollback.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
const bytes = readFileSync(new URL('../target/runtime-awaited/wasm32-unknown-unknown/release/runtime-awaited.wasm', import.meta.url));
const module = new WebAssembly.Module(bytes);
const expected = ['cost_call','canister_liquid_cycle_balance128','call_new','call_on_cleanup','call_with_best_effort_response','call_data_append','call_cycles_add128','call_perform','in_replicated_execution','msg_cycles_refunded128','msg_reject_code','msg_reject_msg_size','msg_reject_msg_copy','msg_arg_data_size','msg_arg_data_copy'];
assert.deepEqual(WebAssembly.Module.imports(module).map(x => `${x.module}/${x.kind}/${x.name}`).sort(), expected.map(x => `ic0/function/${x}`).sort());
const plans = [ ['reply',7], ['reject',null], ['drop',0], ['cleanup',null], ['refuse',null], ['after',0xffffffff], ['exec-reply',null], ['exec-cleanup',null], ['exec-after',null], ['a-first',null], ['b',null], ['a-second',null], ['pair-left',null], ['pair-right',null], ['survivor',null] ];
function harness() {
  let instance, assembly, response;
  let entryReplicated = true, entryObservations = 0;
  const submitted = [];
  const read = (p,n) => [...new Uint8Array(instance.exports.memory.buffer,p>>>0,n>>>0)];
  function write(p, data) {
    assert.deepEqual(read(p,data.length),Array(data.length).fill(0));
    new Uint8Array(instance.exports.memory.buffer,p>>>0,data.length).set(data);
  }
  function u128(p,value) {
    const data = new Uint8Array(16), view = new DataView(data.buffer);
    view.setBigUint64(0,value & ((1n<<64n)-1n),true); view.setBigUint64(8,value>>64n,true); write(p,data);
  }
  function event(name) { assert.ok(assembly); assembly.events.push(name); }
  function capture(name) { assert.ok(response,'capture outside callback'); response.events.push(name); }
  instance = new WebAssembly.Instance(module,{ic0:{
    cost_call(m,a,p) {
      assert.equal(assembly,undefined);
      const [method,timeout] = plans[submitted.length] ?? [];
      assert.ok(method); assert.equal(m,BigInt(method.length)); assert.equal(a,method==='refuse'?0n:3n);
      assembly={method,timeout,events:['cost']}; u128(p,13n);
    },
    canister_liquid_cycle_balance128(p) { event('balance'); u128(p,100n); },
    call_new(p,n,m,l,reply,id,reject,rejectId) {
      event('new'); assert.deepEqual(read(p,n),[4]); assert.deepEqual(read(m,l),[...Buffer.from(assembly.method)]);
      assert.equal(reply,reject); assert.equal(id,rejectId); assert.equal(id,submitted.length+1);
      assert.ok(reply>=0); Object.assign(assembly,{reply,id});
    },
    call_on_cleanup(fun,id) { event('cleanup'); assert.equal(id,assembly.id); assert.notEqual(fun,assembly.reply); assembly.cleanup=fun; },
    call_with_best_effort_response(seconds) { event('wait'); assert.equal(seconds>>>0,assembly.timeout); },
    call_data_append(p,n) { event('data'); assert.deepEqual(read(p,n),[0,128,255]); },
    call_cycles_add128(high,low) { event('cycles'); assert.equal(high,0n); assert.equal(low,9n); },
    call_perform() {
      event('perform'); const wanted=['cost','balance','new','cleanup'];
      if(assembly.timeout!==null) wanted.push('wait');
      if(assembly.method!=='refuse') wanted.push('data','cycles'); wanted.push('perform');
      assert.deepEqual(assembly.events,wanted); const refused=assembly.method==='refuse';
      submitted.push(assembly); assembly=undefined; return refused?-1:0;
    },
    in_replicated_execution() {
      // Context admission and destructor reentry also observe this flag.
      if (response && response.events.length === 0) { capture('replicated'); return response.replicated?1:0; }
      assert.ok(submitted.length >= 6, 'unexpected entry context observation');
      entryObservations++; return entryReplicated?1:0;
    },
    msg_cycles_refunded128(p) { capture('refund'); assert.equal(response.replicated,true); u128(p,response.refund); },
    msg_reject_code() { capture('code'); return response.code|0; },
    msg_reject_msg_size() { capture('reject-size'); assert.notEqual(response.code,0); return response.data.length; },
    msg_reject_msg_copy(p,offset,size) { capture('reject-copy'); assert.equal(offset,0); assert.equal(size,response.data.length); write(p,response.data); },
    msg_arg_data_size() { capture('reply-size'); assert.equal(response.code,0); return response.data.length; },
    msg_arg_data_copy(p,offset,size) { capture('reply-copy'); assert.equal(offset,0); assert.equal(size,response.data.length); write(p,response.data); },
  }});
  function deliver(index,code,data,refund=0n,replicated=true) {
    response={code,data,refund,replicated,events:[]}; const registration=submitted[index];
    instance.exports.__indirect_function_table.get(registration.reply)(registration.id);
    const wanted=['replicated']; if(replicated) wanted.push('refund'); wanted.push('code',code===0?'reply-size':'reject-size',code===0?'reply-copy':'reject-copy');
    assert.deepEqual(response.events,wanted); response=undefined;
  }
  return {step:()=>assert.equal(instance.exports.main(0,0),0), deliver, submitted,
    nonReplicated() { entryReplicated=false; },
    replicated() { entryReplicated=true; },
    observations:()=>entryObservations,
    cleanup(index) { const r=submitted[index]; instance.exports.__indirect_function_table.get(r.cleanup)(r.id); },
  };
}
const run=harness();
run.step(); run.deliver(0,0,[0,128,255],(1n<<100n)+9n);
run.step(); run.deliver(1,6,[...Buffer.from('拒否\0')],0n,false);
run.step(); run.deliver(2,0,[],5n);
run.step(); run.cleanup(3);
run.step(); run.deliver(5,0xffffffff,[...Buffer.from('raw')]);
run.step(); assert.equal(run.submitted.length,6);
run.step(); run.deliver(6,0,[42],17n);
run.step(); run.cleanup(7);
run.step(); run.deliver(8,0,[42],17n);
run.nonReplicated(); run.step();
assert.equal(run.submitted.length,9);
assert.equal(run.observations(),10); // Three admissions, three nested polls, three drops, one refusal.
run.replicated();
run.step(); run.deliver(10,0,[20],201n); // B completes before A.
run.step(); run.deliver(9,0,[10],101n); // A submits its second call from a callback.
assert.equal(run.submitted.length,12);
run.step(); run.deliver(11,0,[11],102n);
run.step(); run.cleanup(12); // One cleanup terminates the root with two physical calls.
run.step(); run.deliver(13,0,[99],999n); // Orphan callback retains custody, but no polling authority.
run.step(); run.deliver(14,0,[30],301n);
run.step(); assert.equal(run.submitted.length,15);
assert.equal(run.observations(),14);
assert.throws(()=>run.deliver(13,0,[]),WebAssembly.RuntimeError,'orphan registration must retire exactly once');
assert.throws(()=>run.cleanup(3),WebAssembly.RuntimeError,'duplicate cleanup must refuse');
const stale=harness(); stale.step(); stale.deliver(0,0,[0,128,255],(1n<<100n)+9n);
assert.throws(()=>stale.deliver(0,0,[]),WebAssembly.RuntimeError,'stale callback must refuse');
const malformed=harness(); malformed.step();
assert.throws(()=>malformed.deliver(0,4,[255]),WebAssembly.RuntimeError,'malformed rejection UTF-8 must trap');
console.log(JSON.stringify({admission:'passed',submissions:15,executor_tasks:7,negative_cases:4,sha256:createHash('sha256').update(bytes).digest('hex'),node:process.version}));
