import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';

const bytes = readFileSync(new URL('../target/types-wasm/runtime-ic0/wasm32-unknown-unknown/release/runtime-entrypoints.wasm', import.meta.url));
const module = new WebAssembly.Module(bytes);
assert.deepEqual(WebAssembly.Module.imports(module).map(x=>`${x.module}/${x.kind}/${x.name}`).sort(), ['in_replicated_execution','msg_arg_data_copy','msg_arg_data_size','msg_reply','msg_reply_data_append'].map(x=>`ic0/function/${x}`));
let instance, contexts=0, sizes=0;
const events=[];
const data=[[1,2,3],[9],[0,255]];
instance=new WebAssembly.Instance(module,{ic0:{
  in_replicated_execution() { events.push('context'); return ++contexts===9?0:1; },
  msg_arg_data_size() { events.push(`size:${sizes}`); assert.ok(sizes<3); return data[sizes++].length; },
  msg_arg_data_copy(p,offset,size) {
    assert.notEqual(sizes,1,'oversized input must not copy');
    assert.equal(offset,0); assert.equal(size,data[sizes-1].length);
    const output=new Uint8Array(instance.exports.memory.buffer,p>>>0,size);
    assert.deepEqual([...output],Array(size).fill(0)); output.set(data[sizes-1]);
    events.push(`copy:${sizes-1}`);
  },
  msg_reply_data_append(p,size) {
    assert.deepEqual([...new Uint8Array(instance.exports.memory.buffer,p>>>0,size)],[0,255]);
    events.push('append');
  },
  msg_reply() { events.push('reply'); },
}});
assert.equal(instance.exports.main(0,0),0);
assert.equal(contexts,9);
assert.deepEqual(events,['context','context','size:0','context','size:1','copy:1','context','size:2','copy:2','context','context','append','reply','context','context','reply','context']);
console.log(JSON.stringify({admission:'passed',cases:6,handler_invocations:1,sha256:createHash('sha256').update(bytes).digest('hex'),node:process.version}));
