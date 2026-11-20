// Source-bound consumer qualification. No sibling writes or live-provider effects.
import {spawnSync} from 'node:child_process';
import {readFileSync,writeFileSync,readdirSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {resolve} from 'node:path';
import assert from 'node:assert/strict';
const root=process.cwd(),source=resolve('target/blob-supplier/source'),build=resolve('target/blob-supplier/build'),native=resolve('target/blob-supplier/native');
function run(command,args,env={}){const r=spawnSync(command,args,{encoding:'utf8',maxBuffer:32*1024*1024,env:{...process.env,...env}});assert.ifError(r.error);process.stdout.write(r.stdout??'');process.stderr.write(r.stderr??'');assert.equal(r.status,0,`${command} failed`);return r.stdout;}
function candidateSource(){
 const files=['Cargo.toml','Cargo.lock'];
 function visit(directory){for(const entry of readdirSync(directory,{withFileTypes:true})){const path=directory+'/'+entry.name;if(entry.isDirectory())visit(path);else if(entry.isFile()&&(path.endsWith('.rs')||entry.name==='Cargo.toml'))files.push(path);}}
 visit('crates');files.sort();
 return Object.fromEntries(files.map(path=>[path,createHash('sha256').update(readFileSync(path)).digest('hex')]));
}
const candidate=candidateSource();
run('node',['qualification/blob-supplier/prepare.mjs']);
run('cargo',['build','--manifest-path',`${source}/Cargo.toml`,'-p','ic-blob-storage-canister','--target','wasm32-unknown-unknown','--release','--target-dir',build]);
const manifest=['--manifest-path',`${source}/qualification/Cargo.toml`];
run('cargo',['build',...manifest,'-p','ic-blob-storage-callback-probe','-p','ic-blob-storage-enrollment-probe','--target','wasm32-unknown-unknown','--release','--target-dir',build]);
const selection=[...manifest,'-p','ic-blob-storage-tests','--test','signed','--test','callback_core','--test','enrollment_supplier','--target-dir',native];
run('cargo',['clippy','--locked',...selection,'--','-D','warnings']);
run('cargo',['clippy','--locked',...manifest,'-p','ic-blob-storage-callback-probe','-p','ic-blob-storage-enrollment-probe','--lib','--target','wasm32-unknown-unknown','--target-dir',build,'--','-D','warnings']);
const artifact=name=>`${build}/wasm32-unknown-unknown/release/${name}.wasm`;
const env={BLOB_WASM:artifact('ic_blob_storage_canister'),BLOB_CALLBACK_PROBE:artifact('ic_blob_storage_callback_probe'),BLOB_ENROLLMENT_PROBE:artifact('ic_blob_storage_enrollment_probe')};
run('node',['qualification/server.mjs','run','cargo','test','--locked',...selection,'--','--nocapture'],env);
const tree=run('cargo',['tree','--locked',...manifest,'-p','ic-blob-storage-tests','--target','x86_64-unknown-linux-gnu','--edges','normal,build,dev','--prefix','none']);
assert(!/^ic-agent /m.test(tree),'old Agent fallback entered selected caller graph');
assert(!/^ic-platform v0\.33\./m.test(tree),'old Platform fallback entered selected caller graph');
const hash=p=>createHash('sha256').update(readFileSync(p)).digest('hex');
assert.deepEqual(candidateSource(),candidate,'candidate sources changed during qualification');
const evidence={candidate,...JSON.parse(readFileSync('target/blob-supplier/source.json')),tests:['signed (1)','callback_core (2)','enrollment_supplier (4)'],artifacts:Object.fromEntries(Object.entries(env).map(([name,path])=>[name,hash(path)])),lock:hash(`${source}/qualification/Cargo.lock`),adapters:Object.fromEntries(['prepare.mjs','codecs.mjs','signed-client.rs'].map(name=>[name,hash(`qualification/blob-supplier/${name}`)])),limits:['Full certificate CLI/issuance and management CLI journeys are not selected by this target.','Storage 0.74.1 still imports legacy Runtime 0.22.5 raw IC APIs on Wasm; this is not a full consumer hard-cut claim.','Testkit held fixtures use their registry Types/Protocol behind the simulator-only boundary.']};
writeFileSync('target/blob-supplier/verified.json',JSON.stringify(evidence,null,2)+'\n');
console.log(JSON.stringify(evidence,null,2));
