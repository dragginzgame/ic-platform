// Platform owns the complete obligation catalog; Make owns dependency dispatch.
import assert from 'node:assert/strict';
import {qualifyAudit, validateReleaseAudit} from './release-audit.mjs';
import {spawnSync} from 'node:child_process';
import {mkdirSync, readdirSync, readFileSync, writeFileSync} from 'node:fs';
import {dirname, join, relative, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
const hash = bytes => createHash('sha256').update(bytes).digest('hex');

export function releaseCatalog(root = process.cwd()) {
  const catalog = readFileSync(join(root, 'ci/release-profiles.json'));
  const typesCatalog = readFileSync(join(root, 'ci/types-release-profiles.json'));
  const types = JSON.parse(typesCatalog);
  assert(types.length > 0 && new Set(types.map(p => p.id)).size === types.length);
  for (const profile of types) {
    assert.equal(profile.owner, 'types');
    assert.equal(profile.command[0], 'make');
  }
  const profiles = {types: types.flatMap(p => p.command.slice(1)), ...JSON.parse(catalog)};
  assert.deepEqual(Object.keys(profiles).sort(), ['platform', 'protocol', 'repository', 'runtime', 'types']);
  for (const targets of Object.values(profiles)) {
    assert(Array.isArray(targets) && targets.length > 0);
    for (const target of targets) assert.match(target, /^[a-zA-Z0-9][a-zA-Z0-9_.-]*$/);
  }
  return {catalog: hash(catalog), types_catalog: hash(typesCatalog), profiles,
    targets: [...new Set(Object.values(profiles).flat())]};
}

// Compare the complete candidate tree with the preceding Platform release.
// Ignore the candidate tag itself: publication creates it after qualification.
export function stableSelection(root, commit) {
  const git = (...args) => spawnSync('git', args, {cwd: root, encoding: 'utf8'});
  const manifest = readFileSync(join(root, 'crates/ic-platform/Cargo.toml'), 'utf8');
  const version = manifest.match(/^version = "(\d+\.\d+\.\d+)"$/m)?.[1];
  if (!version) return null;
  const tags = git('tag', '--sort=-version:refname', '--list', 'platform-v*');
  if (tags.status !== 0) return null;
  const baseline = tags.stdout.trim().split('\n')
    .filter(tag => /^platform-v\d+\.\d+\.\d+$/.test(tag) && tag !== `platform-v${version}`)[0];
  if (!baseline || git('cat-file', '-t', baseline).stdout.trim() !== 'tag') return null;
  const base = git('rev-parse', '--verify', `${baseline}^{commit}`).stdout.trim();
  if (!base || git('merge-base', '--is-ancestor', base, commit).status !== 0) return null;
  const diff = git('diff', '--no-renames', '--name-only', '-z', base, commit);
  if (diff.status !== 0) return null;
  const paths = diff.stdout.split('\0').filter(Boolean);
  const fixture = 'crates/ic-platform/tests/stable_structures.rs';
  const lib = 'crates/ic-platform/src/lib.rs';
  if (!paths.length || !paths.every(path => path === fixture || path === lib)) return null;
  // Added/deleted fixtures, symlinks and mode changes have no qualified mapping.
  for (const path of paths) {
    for (const revision of [base, commit]) {
      const entry = git('ls-tree', revision, '--', path);
      if (entry.status !== 0 || !entry.stdout.startsWith('100644 blob ')) return null;
    }
  }
  if (paths.includes(lib)) {
    // Only a trailing documented, feature-gated upstream alias belongs to this slice.
    const strip = text => text.replace(/\n(?:\/\/\/[^\n]*\n)*#\[cfg\(feature = "stable-structures"\)\]\npub use ic_stable_structures as stable_structures;\n$/, '');
    const old = git('show', `${base}:${lib}`), next = git('show', `${commit}:${lib}`);
    if (old.status !== 0 || next.status !== 0 || strip(old.stdout) !== strip(next.stdout)) return null;
  }
  return {baseline, base, paths};
}

function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (!value || typeof value !== 'object') return value;
  return Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])]));
}

function scopedRunEvidence(output) {
  const runs = readdirSync(output).filter(name => /^validation\.[a-zA-Z0-9]+$/.test(name));
  assert.equal(runs.length, 1, 'expected one shared runner invocation');
  const dir = join(output, runs[0]);
  for (const name of ['plan', 'policy']) {
    assert.deepEqual(readFileSync(join(dir, `parsed-inputs.json.${name}.json`)),
      readFileSync(join(output, `${name}.json`)), `runner used a different ${name}`);
  }
  const timings = readFileSync(join(dir, 'timings.tsv'), 'utf8').trimEnd().split('\n');
  assert.equal(timings.length, 2, 'expected exactly one executed stage');
  const [target, result, , log] = timings[1].split('\t');
  assert.equal(target, 'qualify-stable-structures');
  assert.equal(result, 'PASS');
  assert.equal(dirname(resolve(log)), resolve(dir));
  return hash(readFileSync(log));
}

// No cross-source cache: old receipts must be regenerated under this contract.
export function validateQualificationReceipt(receipt, output, {
  root = process.cwd(), commit, rust, validateAudit = validateReleaseAudit,
} = {}) {
  const catalog = releaseCatalog(root);
  assert([2, 3].includes(receipt.schema), 'regenerate release receipt');
  assert.equal(receipt.commit, commit);
  assert.equal(receipt.rust, rust);
  assert.equal(receipt.host, `${process.platform}/${process.arch}`);
  assert.equal(receipt.catalog, catalog.catalog);
  assert.equal(receipt.types_catalog, catalog.types_catalog);
  if (receipt.schema === 3) {
    assert.equal(receipt.group, 'platform');
    assert.equal(receipt.stage_log_sha256, scopedRunEvidence(output));
    assert.equal(receipt.plan_sha256, hash(readFileSync(join(output, 'plan.json'))));
    assert.equal(receipt.policy_sha256, hash(readFileSync(join(output, 'policy.json'))));
    const selection = stableSelection(root, commit);
    assert(selection, 'scoped receipt requires an isolated stable-structures change');
    assert.deepEqual(receipt.selection, selection);
    assert.deepEqual(receipt.targets, ['qualify-stable-structures']);
    assert(catalog.targets.includes(receipt.targets[0]));
  } else assert.deepEqual(receipt.targets, catalog.targets);
  assert.equal(receipt.log_sha256, hash(readFileSync(join(output, 'qualification.log'))));
  validateAudit(receipt.audit, output, root);
}

export function qualifyRelease({root = process.cwd(), output, execute = spawnSync,
  audit = qualifyAudit, validateAudit = validateReleaseAudit, group = process.env.RELEASE_GROUP} = {}) {
  const run = (command, args) => {
    const result = execute(command, args, {cwd: root, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024});
    if (result.error) throw result.error;
    assert.equal(result.status, 0, `${command}: ${result.stderr}`);
    return result.stdout;
  };
  run('bash', ['vendor/toko-tooling/scripts/ci/check-make-execution.sh']);
  const clean = () => run('bash', ['vendor/toko-tooling/scripts/ci/check-release-source.sh']);
  clean();
  const commit = run('git', ['rev-parse', 'HEAD']).trim();
  const catalog = releaseCatalog(root);
  output ??= join(root, `target/release-qualification/${Date.now()}-${process.pid}`);
  output = resolve(output);
  mkdirSync(dirname(output), {recursive: true});
  mkdirSync(output);
  const receiptPath = join(output, 'receipt.json');
  const receipt = {schema: 2, commit, catalog: catalog.catalog, types_catalog: catalog.types_catalog,
    rust: run('rustc', ['-V']).trim(), host: `${process.platform}/${process.arch}`, targets: catalog.targets};
  console.log(`Release qualification: ${output}`);
  const started = performance.now();
  receipt.audit = audit(output, {root, execute});
  const audited = performance.now();
  // One serial Make graph preserves prerequisites and runs shared phony targets
  // once. Parallel qualification can mutate shared fixture artifacts, so refuse
  // inherited parallel dispatch by explicitly selecting one job.
  let command = 'make', args = ['--jobs=1', ...catalog.targets];
  let env = process.env;
  const selection = group === 'platform' && stableSelection(root, commit);
  if (selection) {
    receipt.schema = 3;
    receipt.group = 'platform';
    receipt.selection = selection;
    receipt.targets = ['qualify-stable-structures'];
    const policy = relative(root, join(output, 'policy.json'));
    const plan = relative(root, join(output, 'plan.json'));
    const write = (path, value) => writeFileSync(join(root, path), JSON.stringify(canonical(value)) + '\n', {flag: 'wx'});
    write(policy, {schema: 1, groups: ['platform'], stages: catalog.targets});
    write(plan, {schema: 1, source: commit, group: 'platform', scope: 'group', policy,
      inputs: ['Makefile', 'Cargo.lock', 'qualification/Cargo.lock', 'ci/release-profiles.json',
        'ci/types-release-profiles.json', 'scripts/qualify-release.mjs', 'crates/ic-platform/Cargo.toml',
        'crates/ic-platform/src/lib.rs', 'crates/ic-platform/tests/stable_structures.rs'],
      stages: catalog.targets.map(target => ({target, required: receipt.targets.includes(target),
        reason: receipt.targets.includes(target) ? 'Stable structures contract' : 'Outside isolated stable structures change'}))});
    receipt.plan_sha256 = hash(readFileSync(join(root, plan)));
    receipt.policy_sha256 = hash(readFileSync(join(root, policy)));
    command = 'bash';
    args = ['vendor/toko-tooling/scripts/ci/run-validation-targets.sh', '--plan', plan];
    env = {...process.env, RELEASE_SOURCE: commit, RELEASE_GROUP: 'platform',
      VALIDATION_REPOSITORY_ROOT: root, VALIDATION_LOG_DIR: output};
  }
  const result = execute(command, args, {env,
    cwd: root, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024,
  });
  const log = `${result.stdout ?? ''}\n${result.stderr ?? ''}`;
  writeFileSync(join(output, 'qualification.log'), log, {flag: 'wx'});
  writeFileSync(join(output, 'timings.json'), JSON.stringify({
    audit_ms: audited - started, qualification_ms: performance.now() - audited,
    total_ms: performance.now() - started,
  }, null, 2) + '\n', {flag: 'wx'});
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `qualification failed; inspect ${output}/qualification.log`);
  if (selection) receipt.stage_log_sha256 = scopedRunEvidence(output);
  receipt.log_sha256 = hash(log);
  validateQualificationReceipt(receipt, output, {root, commit, rust: receipt.rust, validateAudit});
  clean();
  assert.equal(run('git', ['rev-parse', 'HEAD']).trim(), commit);
  writeFileSync(receiptPath, JSON.stringify(receipt, null, 2) + '\n', {flag: 'wx'});
  console.log(`RELEASE_RECEIPT=${receiptPath}`);
  return receipt;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) qualifyRelease();
