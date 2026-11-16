// Controlled external observations exercise admission wiring without an upload.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { tmpdir } from 'node:os';

const root = process.cwd();
const fixture = mkdtempSync(resolve(tmpdir(), 'types-admission-'));
mkdirSync(`${fixture}/bin`);
mkdirSync(`${fixture}/ci`);
writeFileSync(`${fixture}/ci/types-release-profiles.json`, readFileSync('ci/types-release-profiles.json'));
const stub = `#!/usr/bin/env node
const command = process.argv[1].split('/').at(-1);
const args = process.argv.slice(2).join(' ');
const mode = process.env.ADMISSION_CASE;
const commit = 'a'.repeat(40);
const answer = value => console.log(typeof value === 'string' ? value : JSON.stringify(value));
if (command === 'bash') {
  if (args.includes('check-release-source') && mode === 'dirty') process.exit(1);
} else if (command === 'git') {
  if (args.startsWith('remote')) answer(mode === 'repository' ? 'https://example.invalid/repo' : 'https://github.com/dragginzgame/ic-platform.git');
  else if (args.startsWith('cat-file')) answer(mode === 'lightweight' ? 'commit' : 'tag');
  else if (args.startsWith('ls-remote')) answer((mode === 'remote-tag' ? 'b'.repeat(40) : commit) + '\\trefs/tags/types-v0.21.1^{}');
  else answer(commit);
} else if (command === 'cargo') {
  if (args.startsWith('metadata')) answer({workspace_members:['types','runtime'], packages:[
    {id:'types',name:'ic-platform-types',version:'0.21.1',publish:['crates-io'],repository:'https://github.com/dragginzgame/ic-platform',features:{default:[]}},
    {id:'runtime',name:'ic-platform-runtime',publish:mode === 'roster' ? ['crates-io'] : []}
  ]});
  else if (args.startsWith('audit')) { if (mode === 'audit') process.exit(1); answer({vulnerabilities:{found:false}}); }
  else process.exit(99);
} else if (command === 'gh') answer([{databaseId:1,headSha:commit,status:'completed',conclusion:mode === 'hosted' ? 'failure' : 'success'}]);
else if (command === 'curl') {
  if (mode === 'unavailable') process.exit(22);
  answer({vers:mode === 'existing' ? '0.21.1' : '0.20.4'});
} else if (command === 'make') { if (mode === 'profile') process.exit(1); }
else process.exit(98);
`;
for (const tool of ['bash', 'git', 'cargo', 'gh', 'curl', 'make']) {
  writeFileSync(`${fixture}/bin/${tool}`, stub, { mode: 0o755 });
}
for (const mode of ['dirty', 'repository', 'lightweight', 'remote-tag', 'roster',
  'unavailable', 'existing', 'audit', 'profile', 'success']) {
  const result = spawnSync(process.execPath, [resolve(root, 'scripts/types-publication-admission.mjs')], {
    cwd: fixture, encoding: 'utf8', env: { ...process.env,
      PATH: `${fixture}/bin:${process.env.PATH}`, ADMISSION_CASE: mode } });
  if (result.error) throw result.error;
  writeFileSync(`${fixture}/${mode}.log`, `${result.stdout}\n${result.stderr}`);
  assert.equal(result.status === 0, mode === 'success', `${mode}: ${result.stderr}`);
}
console.log(`Types publication refusal wiring passed; evidence: ${fixture}`);
