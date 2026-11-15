// Compile maintained Rust fixtures; never inject or rewrite Rust source.
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const root=fileURLToPath(new URL('..',import.meta.url));
const scratch=mkdtempSync(join(tmpdir(),'ic-platform-input-ui-'));
const cases=[['accepted',null],['invalid-decoder','E0631'],['borrowed-input','E0597'],['borrowed-handler','E0373']];
try {
  for(const [name,expected] of cases) {
    const directory=join(scratch,name,'runtime-input-probe');
    mkdirSync(directory,{recursive:true});
    const source=join(root,'qualification/ui/runtime-input',`${name}.rs`);
    const manifest=join(directory,'Cargo.toml');
    writeFileSync(manifest,`[workspace]\n[package]\nname="runtime-input-probe"\nversion="0.1.0"\nedition="2024"\npublish=false\n[lib]\npath=${JSON.stringify(source)}\n[dependencies]\nrenamed-runtime={ package="ic-platform-runtime", path=${JSON.stringify(join(root,'crates/ic-platform-runtime'))}, default-features=false, features=["call-ic0"] }\n`);
    const result=spawnSync('cargo',['check','--offline','--manifest-path',manifest,'--target','wasm32-unknown-unknown','--target-dir',join(root,'target/runtime-input-ui'),'--message-format=json'],{encoding:'utf8',timeout:60000,maxBuffer:8*1024*1024});
    assert.ifError(result.error);
    assert.equal(result.signal,null);
    const records=result.stdout.trim().split('\n').filter(Boolean).map(line=>JSON.parse(line));
    const errors=records.filter(r=>r.reason==='compiler-message' && r.message.level==='error');
    if(expected===null) {
      assert.equal(result.status,0,result.stderr+JSON.stringify(errors));
    } else {
      assert.notEqual(result.status,0,`${name} unexpectedly compiled`);
      assert.ok(errors.some(r=>r.message.code?.code===expected && r.message.spans.some(s=>s.is_primary && resolve(s.file_name)===resolve(source))),JSON.stringify({name,expected,errors,stderr:result.stderr}));
    }
    assert.ok(records.some(r=>r.reason==='build-finished' && r.success===(expected===null)),result.stderr);
  }
  console.log(JSON.stringify({admission:'passed',accepted:1,rejected:3}));
} finally {
  rmSync(scratch,{recursive:true,force:true});
}
