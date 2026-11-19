// Resolve isolated manifest probes; product sources are referenced, never copied.
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const root=fileURLToPath(new URL('..',import.meta.url));
const runtime=join(root,'crates/ic-platform-runtime');
const guard=join(root,'crates/ic-platform-runtime-guard');
const links='ic-cdk async executor, see https://github.com/dfinity/cdk-rs/blob/links-pin/TROUBLESHOOTING.md';
assert.ok(process.argv.length <= 3 && [undefined,'--prepare'].includes(process.argv[2]),'expected optional --prepare');
const prepare=process.argv[2]==='--prepare';
function cargo(manifest, args=[], offline=true) {
  const result=spawnSync('cargo',['metadata',...(offline?['--offline']:[]),'--format-version','1','--manifest-path',manifest,...args],{encoding:'utf8',timeout:60000,maxBuffer:8*1024*1024});
  assert.ifError(result.error);
  assert.equal(result.signal,null);
  return result;
}
const catalog=cargo(join(root,'Cargo.toml'),['--locked','--no-deps']);
assert.equal(catalog.status,0,catalog.stderr);
const packages=JSON.parse(catalog.stdout).packages;
const rt=packages.find(p=>p.name==='ic-platform-runtime');
const gd=packages.find(p=>p.name==='ic-platform-runtime-guard');
assert.equal(rt.version,gd.version);
assert.equal(gd.links,links);
const dependency=rt.dependencies.find(d=>d.name===gd.name);
assert.equal(dependency.req,`=${gd.version}`);
assert.equal(resolve(dependency.path),resolve(guard));
assert.equal(dependency.optional,true);
assert.ok(rt.features['call-ic0'].includes('dep:ic-platform-runtime-guard'));

const scratch=mkdtempSync(join(tmpdir(),'ic-platform-guard-'));
try {
  function probe(name,deps,offline=true) {
    const directory=join(scratch,name,'runtime-guard-probe');
    mkdirSync(directory,{recursive:true});
    const manifest=join(directory,'Cargo.toml');
    // Cargo metadata needs a target. Reference the guard's real source; no
    // Rust source generation or compilation is involved in these graph probes.
    writeFileSync(manifest,`[workspace]\n[package]\nname="runtime-guard-probe"\nversion="0.1.0"\nedition="2024"\npublish=false\n[lib]\npath=${JSON.stringify(join(guard,'src/lib.rs'))}\n[dependencies]\n${deps}\n`);
    return cargo(manifest,[],offline);
  }
  const candidate=features=>`candidate = { package="ic-platform-runtime", path=${JSON.stringify(runtime)}, default-features=false, features=${JSON.stringify(features)} }`;
  const old='old_guard = { package="ic-platform-runtime-guard", version="=0.22.5", default-features=false }';
  for(const features of [[],['call'],['call-ic0']]) {
    const result=probe(`candidate-${features.join('-')||'default'}`,candidate(features));
    assert.equal(result.status,0,result.stderr);
    const metadata=JSON.parse(result.stdout);
    const runtimePackage=metadata.packages.find(p=>p.name===rt.name);
    const node=metadata.resolve.nodes.find(n=>n.id===runtimePackage.id);
    const guards=node.deps.filter(d=>metadata.packages.find(p=>p.id===d.pkg)?.name===gd.name);
    assert.equal(guards.length,features.includes('call-ic0')?1:0);
  }
  // Explicit preparation may fetch only the exact published baseline. Normal
  // qualification is entirely offline and never refreshes workspace locks.
  const oldOnly=probe('published-control',old,!prepare);
  assert.equal(oldOnly.status,0,oldOnly.stderr);
  function conflict(result) {
    assert.notEqual(result.status,0,'mixed executors unexpectedly resolved');
    assert.ok(result.stderr.includes('links') && result.stderr.includes(links),result.stderr);
  }
  conflict(probe('mixed-published',`${candidate(['call-ic0'])}\n${old}`));

  const alternate=join(scratch,'alternate','ic-platform-runtime-guard');
  mkdirSync(alternate,{recursive:true});
  writeFileSync(join(alternate,'Cargo.toml'),`[workspace]\n[package]\nname="ic-platform-runtime-guard"\nversion=${JSON.stringify(gd.version)}\nedition="2024"\npublish=false\nlinks=${JSON.stringify(links)}\nbuild=${JSON.stringify(join(guard,'build.rs'))}\n[lib]\npath=${JSON.stringify(join(guard,'src/lib.rs'))}\n`);
  const alt=`alternate = { package="ic-platform-runtime-guard", path=${JSON.stringify(alternate)} }`;
  const altOnly=probe('alternate-control',alt);
  assert.equal(altOnly.status,0,altOnly.stderr);
  conflict(probe('mixed-source',`${candidate(['call-ic0'])}\n${alt}`));
  console.log(JSON.stringify({admission:'passed',version:gd.version,positive_graphs:5,conflicts:2}));
} finally {
  rmSync(scratch,{recursive:true,force:true});
}
