// Freeze and migrate the actual Query consumer without changing its checkout.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {lstatSync, mkdirSync, readFileSync, writeFileSync} from 'node:fs';
import {dirname, relative, resolve} from 'node:path';
import {adoptTooling} from './tooling-selection.mjs';
const root = process.cwd();
const source = resolve(process.env.QUERY_SOURCE ?? '../ic-query');
const output = resolve(`target/query-hard-cut/${Date.now()}`);
mkdirSync(output, {recursive:true});
function run(command, args, cwd = output) {
  const result = spawnSync(command, args, {cwd, encoding:'utf8', maxBuffer:32*1024*1024,
    env:{...process.env,CARGO_TARGET_DIR:resolve(root,'target/query-hard-cut-build')}});
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stdout + result.stderr);
  return result.stdout.trim();
}
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const expected = JSON.parse(readFileSync('qualification/query-hard-cut/source.json'));
const revision = run('git', ['rev-parse', 'HEAD'], source);
assert.equal(revision, expected.revision, 'Query source revision changed; requalify the migration patch');
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
for (const [name,digest] of Object.entries(expected.files)) assert.equal(inventory[name],digest,`Query patch input changed: ${name}`);
const patch = readFileSync('qualification/query-hard-cut/consumer.patch','utf8');
writeFileSync(`${output}/consumer.patch`,patch.replaceAll('__PLATFORM_ROOT__',root));
run('git',['apply',`--directory=${relative(root,output)}`,`${output}/consumer.patch`],root);
adoptTooling(`${output}/Cargo.toml`);
run('cargo',['update']);
const metadata = JSON.parse(run('cargo',['metadata','--locked','--format-version=1']));
for (const [name,version] of [['ic-platform','0.34.6'],['ic-platform-protocol','0.2.5'],['ic-platform-types','0.21.2']]) {
  const packages = metadata.packages.filter(p => p.name === name);
  assert.equal(packages.length,1,`one canonical ${name}`);
  assert.equal(packages[0].version,version);
  assert.equal(packages[0].source,null);
  assert(packages[0].manifest_path.startsWith(`${root}/crates/`));
}
assert(!metadata.packages.some(p => p.name === 'ic-agent'), 'Agent must not reenter this consumer graph');
const selected = ['--locked','-p','ic-topology','-p','ic-query-cli'];
const tests = run('cargo',['test',...selected]);
run('cargo',['clippy',...selected,'--all-targets','--','-D','warnings']);
for (const name of files) assert.equal(hash(readFileSync(resolve(source,name))),inventory[name],`source changed during qualification: ${name}`);
const evidence = {revision,dirty,inventory,inventorySha256:hash(JSON.stringify(inventory)),patchSha256:hash(patch),
  lockSha256:hash(readFileSync(`${output}/Cargo.lock`)),packages:{platform:'0.34.6',protocol:'0.2.5',types:'0.21.2'},tests,status:'passed'};
writeFileSync(`${output}/verified.json`,JSON.stringify(evidence,null,2)+'\n');
console.log(JSON.stringify({output,...evidence},null,2));
