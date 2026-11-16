// Freeze actual Query inputs before checking its registry-only settings cohort.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {lstatSync, mkdirSync, readFileSync, writeFileSync} from 'node:fs';
import {dirname, resolve} from 'node:path';
const source = resolve(process.env.QUERY_SOURCE ?? '../ic-query');
const output = resolve(`target/query-settings/${Date.now()}`);
mkdirSync(output, {recursive:true});
function run(command, args, cwd = output) {
  const result = spawnSync(command, args, {cwd, encoding:'utf8', maxBuffer:32*1024*1024});
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stdout + result.stderr);
  return result.stdout.trim();
}
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const revision = run('git', ['rev-parse', 'HEAD'], source);
const dirty = run('git', ['status', '--short'], source);
const files = [...new Set(run('git', ['ls-files', '-z', '--cached', '--others', '--exclude-standard'], source)
  .split('\0').filter(Boolean))].sort();
const inventory = {};
for (const name of files) {
  const from = resolve(source, name), to = resolve(output, name);
  assert(from.startsWith(`${source}/`) && to.startsWith(`${output}/`));
  assert(lstatSync(from).isFile(), `expected ordinary source file: ${name}`);
  const bytes = readFileSync(from);
  inventory[name] = hash(bytes);
  mkdirSync(dirname(to), {recursive:true});
  writeFileSync(to, bytes);
}
const metadata = JSON.parse(run('cargo', ['metadata', '--offline', '--locked', '--format-version=1']));
for (const [name, version] of [['ic-platform-protocol','0.2.3'], ['ic-platform-types','0.21.1']]) {
  assert(metadata.packages.some(p => p.name === name && p.version === version && p.source?.startsWith('registry+')));
}
const results = [];
for (const [pkg, filter] of [['ic-topology','subnet::settings::'], ['ic-topology','settings_comparison'], ['ic-query-cli','settings']]) {
  results.push(run('cargo', ['test', '--offline', '--locked', '-p', pkg, filter]));
}
run('cargo', ['clippy', '--offline', '--locked', '-p', 'ic-topology', '-p', 'ic-query-cli', '--all-targets', '--', '-D', 'warnings']);
for (const name of files) assert.equal(hash(readFileSync(resolve(source,name))), inventory[name], `source changed during qualification: ${name}`);
const evidence = {revision, dirty, inventory, inventorySha256:hash(JSON.stringify(inventory)),
  packages:{protocol:'0.2.3',types:'0.21.1'}, results, status:'passed'};
writeFileSync(`${output}/verified.json`, JSON.stringify(evidence,null,2)+'\n');
console.log(JSON.stringify({output,revision,inventorySha256:evidence.inventorySha256,results,status:evidence.status},null,2));
