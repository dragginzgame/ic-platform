// Freeze and migrate the actual Query consumer without changing its checkout.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {lstatSync, mkdirSync, readFileSync, writeFileSync} from 'node:fs';
import {dirname, relative, resolve} from 'node:path';
import {adoptTooling} from './tooling-selection.mjs';
const root = process.cwd();
const cohort = process.env.QUERY_COHORT ?? 'registry';
const versions = cohort === 'candidate' ? {platform:'0.34.8',protocol:'0.2.7',types:'0.21.2'} : {platform:'0.34.6',protocol:'0.2.5',types:'0.21.2'};
assert(['registry', 'candidate'].includes(cohort), 'unknown Query qualification cohort');
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
const portable = {};
for (const [from, name] of [['fixtures.rs', 'mod.rs'], ['corpus-v1.json', 'corpus-v1.json']]) {
  const bytes = readFileSync(`qualification/query-hard-cut/portable/${from}`);
  portable[from] = hash(bytes);
  const destination = `${output}/crates/ic-topology/src/tests/${name}`;
  mkdirSync(dirname(destination), {recursive:true});
  writeFileSync(destination, bytes);
}
adoptTooling(`${output}/Cargo.toml`);
if (cohort === 'candidate') {
  let manifest = readFileSync(`${output}/Cargo.toml`, 'utf8');
  for (const name of ['ic-platform', 'ic-platform-protocol', 'ic-platform-types']) {
    const prefix = `${name} = { version = `;
    assert(manifest.includes(prefix));
    const version = versions[name === "ic-platform" ? "platform" : name === "ic-platform-protocol" ? "protocol" : "types"];
    manifest = manifest.replace(new RegExp(`${name} = \{ version = "[^"\n]+"`), `${name} = { path = "${root}/crates/${name}", version = "=${version}"`);
  }
  writeFileSync(`${output}/Cargo.toml`, manifest);
}
run('cargo',['update']);
const metadata = JSON.parse(run('cargo',['metadata','--locked','--format-version=1']));
for (const [name,version] of [['ic-platform',versions.platform],['ic-platform-protocol',versions.protocol],['ic-platform-types',versions.types]]) {
  const packages = metadata.packages.filter(p => p.name === name);
  assert.equal(packages.length,1,`one canonical ${name}`);
  assert.equal(packages[0].version,version);
  if (cohort === 'registry') {
    assert.equal(packages[0].source,'registry+https://github.com/rust-lang/crates.io-index');
  } else {
    assert.equal(packages[0].source, null);
    assert(packages[0].manifest_path.startsWith(`${root}/crates/`));
  }
}
assert(!metadata.packages.some(p => p.name === 'ic-agent'), 'Agent must not reenter this consumer graph');
assert(!metadata.packages.some(p => p.name === 'engine-view-consumer'), 'portable consumer must not select the private schema generator');
for (const name of ['ic-topology', 'ic-query-cli']) {
  assert(!metadata.packages.find(p => p.name === name).dependencies.some(d => d.name === 'candid'), 'Query must not directly select Candid');
}
const selected = ['--locked','-p','ic-topology','-p','ic-query-cli'];
const tests = run('cargo',['test',...selected]);
run('cargo',['clippy',...selected,'--all-targets','--','-D','warnings']);
const archives = {};
if (cohort === 'registry') {
run('cargo', ['package', '--locked', '--allow-dirty', '--no-verify', '-p', 'ic-topology', '-p', 'ic-query-cli']);
for (const name of ['ic-topology', 'ic-query-cli']) {
  const pkg = metadata.packages.find(p => p.name === name);
  const archive = resolve(root, `target/query-hard-cut-build/package/${name}-${pkg.version}.crate`);
  const listing = run('tar', ['-tzf', archive]);
  assert(!listing.includes('engine-view-consumer'));
  if (name === 'ic-topology') {
    assert(listing.includes('/src/tests/mod.rs'));
    assert(listing.includes('/src/tests/corpus-v1.json'));
  }
  const extracted = `${output}/archives`;
  mkdirSync(extracted, {recursive:true});
  run('tar', ['-xzf', archive, '-C', extracted]);
  const directory = `${extracted}/${name}-${pkg.version}`;
  const manifest = readFileSync(`${directory}/Cargo.toml`, 'utf8');
  assert(!/^path\s*=\s*"(?:\/|\.\.)/m.test(manifest), 'archive must not need outside paths');
  // The archive is independent; prevent Cargo from discovering this validation
  // directory's enclosing source workspace. No dependency selection is changed.
  writeFileSync(`${directory}/Cargo.toml`, `${manifest}\n[workspace]\n`);
  archives[name] = {sha256:hash(readFileSync(archive)),version:pkg.version};
  if (name === 'ic-topology') {
    run('cargo', ['test', '--locked'], directory);
    run('cargo', ['clippy', '--locked', '--all-targets', '--', '-D', 'warnings'], directory);
  }
}
}
for (const name of files) assert.equal(hash(readFileSync(resolve(source,name))),inventory[name],`source changed during qualification: ${name}`);
const evidence = {revision,dirty,cohort,inventory,portable,archives,inventorySha256:hash(JSON.stringify(inventory)),patchSha256:hash(patch),
  lockSha256:hash(readFileSync(`${output}/Cargo.lock`)),packages:versions,tests,status:'passed'};
writeFileSync(`${output}/verified.json`,JSON.stringify(evidence,null,2)+'\n');
console.log(JSON.stringify({output,...evidence},null,2));
