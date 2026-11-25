// Explicit receipt reuse; the shared runner still owns clean-source dispatch.
import assert from 'node:assert/strict';
import {validateQualificationReceipt} from './qualify-release.mjs';
import { spawnSync } from 'node:child_process';
import { readFileSync, mkdirSync, appendFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
const run = (cmd,args) => {
  const r=spawnSync(cmd,args,{encoding:'utf8',maxBuffer:32*1024*1024});
  if(r.error)throw r.error;
  assert.equal(r.status,0,`${cmd} ${args.join(' ')}\n${r.stderr}`);
  return r.stdout;
};
run('bash',['vendor/toko-tooling/scripts/ci/check-release-source.sh']);
assert(process.env.RELEASE_RECEIPT,'Select an explicit complete local RELEASE_RECEIPT');
const receipt=JSON.parse(readFileSync(process.env.RELEASE_RECEIPT));
const commit=run('git',['rev-parse','HEAD']).trim();
validateQualificationReceipt(receipt, dirname(process.env.RELEASE_RECEIPT), {
  commit, rust: run('rustc',['-V']).trim(),
});
const selection=process.env.CARGO_PUBLISH_SELECTION?.trim().split(/\s+/);
assert.equal(selection?.length,1,'Publish one admitted package at a time in dependency order');
const name=selection[0];
const policy=JSON.parse(readFileSync('ci/cargo-release-groups.json'));
const groups=policy.groups.filter(group=>group.members.includes(name));
assert.equal(groups.length,1,'package must have exactly one release owner');
assert.equal(process.env.RELEASE_GROUP,groups[0].name,'select the package release group explicitly');
const metadata=JSON.parse(run('cargo',['metadata','--locked','--format-version','1']));
const product=metadata.packages.find(p=>p.name===name && metadata.workspace_members.includes(p.id));
assert(product);assert.deepEqual(product.publish,['crates-io']);assert.deepEqual(product.features.default,[]);
assert.equal(product.repository,'https://github.com/dragginzgame/ic-platform');
assert.equal(run('git',['remote','get-url','origin']).trim(),`${product.repository}.git`);
const tag=`${groups[0].name}-v${product.version}`;
assert.equal(run('git',['cat-file','-t',tag]).trim(),'tag');
assert.equal(run('git',['rev-parse',`${tag}^{commit}`]).trim(),commit);
assert.equal(run('git',['ls-remote','--exit-code','origin',`refs/tags/${tag}^{}`]).trim().split(/\s+/)[0],commit);
const output=resolve(`target/release-admission/${Date.now()}-${process.pid}`);mkdirSync(output,{recursive:true});
const status=run('curl',['--silent','--show-error','--output',`${output}/index.jsonl`,'--write-out','%{http_code}',`https://index.crates.io/ic/-p/${name}`]);
assert(status==='200'||(status==='404'&&name==='ic-platform-protocol'),'Registry observation unavailable');
if(status==='200')assert(!readFileSync(`${output}/index.jsonl`,'utf8').trim().split('\n').map(JSON.parse).some(p=>p.vers===product.version),'Already published; reconcile, do not retry');
run('cargo',['package','--locked','--all-features','-p',name]);
run('tar',['-xf',`target/package/${name}-${product.version}.crate`,'-C',output]);
const manifest=`${output}/${name}-${product.version}/Cargo.toml`;
// Isolate Cargo workspace discovery only; leave archive and dependencies intact.
appendFileSync(manifest,'\n[workspace]\n');
for(const target of [[],['--target','wasm32-unknown-unknown']])run('cargo',['clippy','--locked','--manifest-path',manifest,'--all-features','--all-targets',...target,'--','-D','warnings']);
run('cargo',['test','--locked','--manifest-path',manifest,'--all-features']);
run('bash',['vendor/toko-tooling/scripts/ci/check-cargo-package-provenance.sh',`${output}/provenance`,'target/package',product.repository,commit,'--package',`${name}@${product.version}`,`crates/${name}`]);
console.log(`Admitted ${name}@${product.version}; evidence ${output}`);
