// Compile the pinned consumer through a reviewable migration patch; sibling is read-only.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdirSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {adoptTooling} from '../tooling-selection.mjs';
const root=process.cwd(), source=resolve(process.env.STORAGE_SOURCE??'../ic-storage');
const revision='3ad8beda21eeb9451c7755791648b2a5cbffb0c1';
const destination=resolve(`target/storage-runtime-${Date.now()}`);
mkdirSync(destination,{recursive:true});
function run(command,args,cwd=destination) {
  const result=spawnSync(command,args,{cwd,encoding:'utf8',maxBuffer:32*1024*1024});
  assert.ifError(result.error); assert.equal(result.status,0,result.stdout+result.stderr);
  return result.stdout;
}
run('git',['archive',`--output=${destination}/source.tar`,revision,'Cargo.toml','Cargo.lock','LICENSE.md','crates','qualification'],source);
run('tar',['-xf','source.tar']);
run('patch',['--batch','-p1','-i',resolve(root,'qualification/storage-runtime/migration.patch')]);
for (const name of ['Cargo.toml','qualification/Cargo.toml']) adoptTooling(`${destination}/${name}`);
const base=['--offline','--manifest-path','qualification/Cargo.toml','--target','wasm32-unknown-unknown'];
run('cargo',['check',...base,'-p','ic-storage-rust-startup-probe','--features','managed-bench']);
for(const [name,features,extra] of [
  ['ic-storage-rust-startup-probe','managed-bench',[]],
  ['ic-storage-checkpoint-provider-probe','native-job',[]],
  ['ic-storage-simple-benchmarks','storage',['--example','storage-wasm']],
])run('cargo',['clippy',...base,'--locked','-p',name,'--features',features,...extra,'--','-D','warnings']);
writeFileSync(`${destination}/source.json`,JSON.stringify({revision,scope:'actual pinned consumer Wasm compilation; no installed Storage or production migration claim'},null,2)+'\n');
console.log(`Storage synchronous consumer compilation passed: ${destination}`);
