// Exercise actual IC exports, with fresh instances for every trapping case.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';

const bytes=readFileSync(new URL('../target/runtime-exports/wasm32-unknown-unknown/release/runtime_exports.wasm',import.meta.url));
const module=new WebAssembly.Module(bytes);
assert.deepEqual(WebAssembly.Module.exports(module).filter(x=>x.kind==='function').map(x=>x.name).sort(),
  ['canister_inspect_message','canister_update denied','canister_update echo','canister_update probe']);
assert.deepEqual(WebAssembly.Module.imports(module).map(x=>`${x.module}/${x.kind}/${x.name}`).sort(),
  ['accept_message','in_replicated_execution','is_controller','msg_arg_data_copy','msg_arg_data_size','msg_caller_copy','msg_caller_size',
    'msg_method_name_copy','msg_method_name_size','msg_reply','msg_reply_data_append'].map(x=>`ic0/function/${x}`));
function run(exportName,data,replicated,trap,wanted,{method=[],caller=[42,2]}={}) {
  let instance;
  const events=[];
  function copy(p,offset,size,bytes) {
    assert.equal(offset,0); assert.equal(size,bytes.length);
    const output=new Uint8Array(instance.exports.memory.buffer,p>>>0,size);
    assert.deepEqual([...output],Array(size).fill(0)); output.set(bytes);
  }
  instance=new WebAssembly.Instance(module,{ic0:{
    accept_message() { events.push('accept'); },
    in_replicated_execution() { events.push('context'); return replicated?1:0; },
    is_controller(p,size) {
      const principal = new Uint8Array(instance.exports.memory.buffer,p>>>0,size);
      assert.deepEqual([...principal],caller);
      events.push('controller');
      return caller[0] === 42 && caller[1] === 2 ? 1 : 0;
    },
    msg_arg_data_size() { events.push('size'); return data.length; },
    msg_arg_data_copy(p,offset,size) {
      copy(p,offset,size,data);
      events.push('copy');
    },
    msg_caller_size() { events.push('caller size'); return caller.length; },
    msg_caller_copy(p,offset,size) { copy(p,offset,size,caller); events.push('caller copy'); },
    msg_method_name_size() { events.push('method size'); return method.length; },
    msg_method_name_copy(p,offset,size) { copy(p,offset,size,method); events.push('method copy'); },
    msg_reply_data_append(p,size) {
      assert.deepEqual([...new Uint8Array(instance.exports.memory.buffer,p>>>0,size)],data);
      events.push('append');
    },
    msg_reply() { events.push('reply'); },
  }});
  const entry=instance.exports[exportName];
  assert.equal(entry.length,0,'IC export must take no Wasm arguments');
  if(trap) assert.throws(()=>entry(),WebAssembly.RuntimeError);
  else assert.equal(entry(),undefined,'IC update must return no Wasm value');
  assert.deepEqual(events,wanted);
}
run('canister_update echo',[0,128,254],true,false,['context','size','copy','context','append','reply']);
run('canister_update echo',[],true,false,['context','size','context','reply']);
run('canister_update denied',[1],true,true,['context']);
run('canister_update echo',[1,2,3,4,5],true,true,['context','size']);
run('canister_update echo',[255],true,true,['context','size','copy']);
run('canister_update echo',[1],false,true,['context']);

run('canister_update probe',[0,128,254],true,false,['context','size','copy','context','append','reply']);
run('canister_update probe',[255],true,true,['context','size','copy']);
run('canister_update probe',[1],false,true,['context']);

const inspect='canister_inspect_message';
const metadata=['method size','method copy','caller size','caller copy','size'];
run(inspect,[0,1,2,3],false,false,[...metadata,'controller','accept'],{method:Buffer.from('echo')});
run(inspect,[0],false,false,[...metadata,'controller'],{method:Buffer.from('echo'),caller:[43,2]});
run(inspect,[0,1,2,3,4],false,false,metadata,{method:Buffer.from('echo')});
run(inspect,[0],false,false,metadata,{method:Buffer.from('drop')});
run(inspect,[0],false,false,['method size'],{method:Buffer.from('probe')});
run(inspect,[0],false,false,['method size','method copy'],{method:[255]});
console.log(JSON.stringify({admission:'passed',exports:4,cases:15,sha256:createHash('sha256').update(bytes).digest('hex'),node:process.version}));
