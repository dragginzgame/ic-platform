// Build real Wasm artifacts so duplicate export names reach codegen/linking.
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
const root=fileURLToPath(new URL('..',import.meta.url));
const cases=[['accepted',null],['invalid-option','unsupported update option'],['invalid-handler','update requires a safe async nongeneric function'],['invalid-decoder','E0631'],['returning-error','E0308'],['duplicate-export','already defined']];
const scratch=mkdtempSync(join(tmpdir(),'ic-platform-export-ui-'));
try {
  for(const [name,expected] of cases) {
    const directory=join(scratch,name,'runtime-export-probe');
    mkdirSync(directory,{recursive:true});
    const source=name==='accepted'?join(root,'qualification/runtime-exports/src/lib.rs'):join(root,'qualification/ui/runtime-exports',`${name}.rs`);
    const manifest=join(directory,'Cargo.toml');
    writeFileSync(manifest,`[workspace]\n[package]\nname="runtime-export-probe"\nversion="0.1.0"\nedition="2024"\npublish=false\n[lib]\ncrate-type=["cdylib"]\npath=${JSON.stringify(source)}\n[dependencies]\ncandidate-runtime={package="ic-platform-runtime",path=${JSON.stringify(join(root,'crates/ic-platform-runtime'))},default-features=false,features=["entrypoint-macros"]}\n[lints.rust]\nunsafe_code="deny"\nunfulfilled_lint_expectations="deny"\n`);
    const result=spawnSync('cargo',['build','--offline','--manifest-path',manifest,'--target','wasm32-unknown-unknown','--target-dir',join(root,'target/runtime-exports-ui'),'--message-format=json'],{encoding:'utf8',timeout:60000,maxBuffer:8*1024*1024});
    assert.ifError(result.error); assert.equal(result.signal,null);
    const records=result.stdout.trim().split('\n').filter(Boolean).map(line=>JSON.parse(line));
    const errors=records.filter(r=>r.reason==='compiler-message' && r.message.level==='error');
    if(expected===null) {
      assert.equal(result.status,0,result.stderr+JSON.stringify(errors));
      assert.ok(records.some(r=>r.reason==='compiler-artifact' && r.filenames.some(p=>p.endsWith('.wasm'))));
    } else {
      assert.notEqual(result.status,0,`${name} unexpectedly built`);
      // Custom macro/codegen errors have no rustc code; match their specific
      // message only in those cases, not arbitrary Cargo stderr failures.
      assert.ok(errors.some(r=>expected.startsWith('E')?r.message.code?.code===expected:r.message.message.includes(expected)),JSON.stringify({name,errors,stderr:result.stderr}));
    }
    assert.ok(records.some(r=>r.reason==='build-finished' && r.success===(expected===null)),result.stderr);
  }
  console.log(JSON.stringify({admission:'passed',accepted:1,rejected:5}));
} finally { rmSync(scratch,{recursive:true,force:true}); }
