// Platform owns the complete obligation catalog; Make owns dependency dispatch.
import assert from 'node:assert/strict';
import {qualifyAudit, validateReleaseAudit} from './release-audit.mjs';
import {spawnSync} from 'node:child_process';
import {mkdirSync, readFileSync, writeFileSync} from 'node:fs';
import {dirname, join, resolve} from 'node:path';
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

// No cross-source cache: old receipts must be regenerated under this contract.
export function validateQualificationReceipt(receipt, output, {
  root = process.cwd(), commit, rust, validateAudit = validateReleaseAudit,
} = {}) {
  const catalog = releaseCatalog(root);
  assert.equal(receipt.schema, 2, 'regenerate the complete local release receipt');
  assert.equal(receipt.commit, commit);
  assert.equal(receipt.rust, rust);
  assert.equal(receipt.host, `${process.platform}/${process.arch}`);
  assert.equal(receipt.catalog, catalog.catalog);
  assert.equal(receipt.types_catalog, catalog.types_catalog);
  assert.deepEqual(receipt.targets, catalog.targets);
  assert.equal(receipt.log_sha256, hash(readFileSync(join(output, 'qualification.log'))));
  validateAudit(receipt.audit, output, root);
}

export function qualifyRelease({root = process.cwd(), output, execute = spawnSync,
  audit = qualifyAudit, validateAudit = validateReleaseAudit} = {}) {
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
  const result = execute('make', ['--jobs=1', ...catalog.targets], {
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
  receipt.log_sha256 = hash(log);
  validateQualificationReceipt(receipt, output, {root, commit, rust: receipt.rust, validateAudit});
  clean();
  assert.equal(run('git', ['rev-parse', 'HEAD']).trim(), commit);
  writeFileSync(receiptPath, JSON.stringify(receipt, null, 2) + '\n', {flag: 'wx'});
  console.log(`RELEASE_RECEIPT=${receiptPath}`);
  return receipt;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) qualifyRelease();
