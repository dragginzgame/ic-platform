import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,readFileSync,writeFileSync,symlinkSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {spawnSync} from 'node:child_process';
const script=resolve('scripts/ci/setup-step.sh');
const cwd=mkdtempSync(join(tmpdir(),'platform-setup-test-'));
const run=(phase,command)=>spawnSync('bash',[script,phase,'bash','-c',command],{cwd,encoding:'utf8'});
try {
 mkdirSync(join(cwd,'target/.attempt'),{recursive:true});
 writeFileSync(join(cwd,'target/.attempt/download.log'),'download failed: HTTP 503\ntoken=excluded\n-----BEGIN PRIVATE KEY-----\nopaque-key-bytes\n-----END PRIVATE KEY-----\n');
 const failed=run('download','cat target/.attempt/download.log; exit 27');
 assert.equal(failed.status,27);
 const log=readFileSync(join(cwd,'target/ci-setup/download.log'),'utf8');
 assert.match(log,/HTTP 503/); assert.match(log,/status: 27/);
 assert(!log.includes('excluded'));assert(!log.includes('opaque-key-bytes'));
 assert.equal(run('empty','exit 9').status,9);
 assert.match(readFileSync(join(cwd,'target/ci-setup/empty.log'),'utf8'),/no diagnostics/);
 assert.equal(run('bounded',"yes 'failure' | head -n 20000; exit 8").status,8);
 assert(readFileSync(join(cwd,'target/ci-setup/bounded.log')).length<34000);
 symlinkSync(join(cwd,'outside'),join(cwd,'target/ci-setup/refused.log'));
 const refused=run('refused','echo failure; exit 19');assert.equal(refused.status,19);assert.match(refused.stderr,/refused symlink/);
 assert.equal(run('success','exit 0').status,0);
 console.log('Setup diagnostics: original status, hidden attempt output, redaction, bounds, missing diagnostics and collector refusal passed.');
} finally {rmSync(cwd,{recursive:true,force:true});}
