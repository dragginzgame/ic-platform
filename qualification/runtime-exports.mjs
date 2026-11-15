// Exercise actual IC exports, with fresh instances for every trapping case.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';

const bytes=readFileSync(new URL('../target/runtime-exports/wasm32-unknown-unknown/release/runtime_exports.wasm',import.meta.url));
const module=new WebAssembly.Module(bytes);
assert.deepEqual(WebAssembly.Module.exports(module).filter(x=>x.kind==='function').map(x=>x.name).sort(),['canister_update denied','canister_update echo']);
assert.deepEqual(WebAssembly.Module.imports(module).map(x=>`${x.module}/${x.kind}/${x.name}`).sort(),['in_replicated_execution','msg_arg_data_copy','msg_arg_data_size','msg_reply','msg_reply_data_append'].map(x=>`ic0/function/${x}`));
function run(method,data,replicated,trap,wanted) {
  let instance;
  const events=[];
  instance=new WebAssembly.Instance(module,{ic0:{
    in_replicated_execution() { events.push('context'); return replicated?1:0; },
    msg_arg_data_size() { events.push('size'); return data.length; },
    msg_arg_data_copy(p,offset,size) {
      assert.equal(offset,0); assert.equal(size,data.length);
      const output=new Uint8Array(instance.exports.memory.buffer,p>>>0,size);
      assert.deepEqual([...output],Array(size).fill(0)); output.set(data);
      events.push('copy');
    },
    msg_reply_data_append(p,size) {
      assert.deepEqual([...new Uint8Array(instance.exports.memory.buffer,p>>>0,size)],data);
      events.push('append');
    },
    msg_reply() { events.push('reply'); },
  }});
  const entry=instance.exports[`canister_update ${method}`];
  assert.equal(entry.length,0,'IC update must take no Wasm arguments');
  if(trap) assert.throws(()=>entry(),WebAssembly.RuntimeError);
  else assert.equal(entry(),undefined,'IC update must return no Wasm value');
  assert.deepEqual(events,wanted);
}
run('echo',[0,128,254],true,false,['context','size','copy','context','append','reply']);
run('echo',[],true,false,['context','size','context','reply']);
run('denied',[1],true,true,['context']);
run('echo',[1,2,3,4,5],true,true,['context','size']);
run('echo',[255],true,true,['context','size','copy']);
run('echo',[1],false,true,['context']);
console.log(JSON.stringify({admission:'passed',exports:2,cases:6,sha256:createHash('sha256').update(bytes).digest('hex'),node:process.version}));
