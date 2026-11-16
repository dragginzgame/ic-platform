// Exercise caller wiring and cleanup on success, command failure and bad readiness.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdtempSync,writeFileSync,readFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
const dir=mkdtempSync(join(tmpdir(),'platform-server-test-'));
try {
  const binary=join(dir,'server');
  const pidfile=join(dir,'pid');
  for(const [port,status] of [['12345\n',0],['12345\n',23],['invalid\n',0]]) {
    writeFileSync(binary,`#!${process.execPath}\nimport {writeFileSync} from 'node:fs';\nwriteFileSync(${JSON.stringify(pidfile)},String(process.pid));\nwriteFileSync(process.argv.at(-1),${JSON.stringify(port)});\nsetInterval(()=>{},1000);\n`,{mode:0o755});
    const result=spawnSync(resolve('qualification/target/debug/platform-qualification-server'),[binary,process.execPath,'-e',`if(!process.env.POCKET_IC_SERVER_URL) process.exit(99); process.exit(${status});`],{encoding:'utf8',timeout:10_000});
    assert.ifError(result.error);
    assert.equal(result.status===0,port==='12345\n'&&status===0,result.stderr);
    const pid=Number(readFileSync(pidfile,'utf8'));
    assert.throws(()=>process.kill(pid,0),{code:'ESRCH'},'server must be reaped on every path');
  }
  console.log('qualification server: success, command failure and invalid readiness all reap the server');
} finally {rmSync(dir,{recursive:true,force:true});}
