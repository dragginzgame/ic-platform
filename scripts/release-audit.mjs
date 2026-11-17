// Platform audit policy; Toko Tooling owns isolated database acquisition.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync, writeFileSync} from 'node:fs';
import {join, resolve} from 'node:path';
import {spawnSync} from 'node:child_process';

export const advisorySource = 'https://github.com/RustSec/advisory-db.git';
const maximumAgeMs = 24 * 60 * 60 * 1000;
const selections = [
  {lock: 'Cargo.lock', report: 'audit.json'},
  {lock: 'qualification/Cargo.lock', report: 'qualification-audit.json'},
];
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
function admitReport(bytes, revision) {
  const report = JSON.parse(bytes);
  // cargo-audit --no-fetch reports null Git metadata even for a fresh clone.
  // Identity comes from the prepared database receipt, never from that null.
  assert(report.database?.['last-commit'] === null || report.database?.['last-commit'] === revision, 'audit database identity mismatched');
  assert(Number.isSafeInteger(report.database['advisory-count']) && report.database['advisory-count'] > 0, 'empty advisory database');
  assert.equal(report.vulnerabilities?.found, false, 'vulnerabilities found');
  assert.equal(report.vulnerabilities.count, 0, 'inconsistent vulnerability count');
  assert.deepEqual(report.vulnerabilities.list, [], 'inconsistent vulnerability details');
}

// Called once by the release collector before product checks. Failure preserves
// acquisition diagnostics and reports, and cannot create a passing receipt.
export function qualifyAudit(output, {root = process.cwd(), execute = spawnSync} = {}) {
  output = resolve(output);
  const run = (command, args, report) => {
    const result = execute(command, args, {cwd: root, encoding: 'utf8', timeout: 180_000, maxBuffer: 32 * 1024 * 1024});
    if (report) {
      writeFileSync(join(output, report), result.stdout ?? '', {flag: 'wx'});
      writeFileSync(join(output, `${report}.stderr`), result.stderr ?? '', {flag: 'wx'});
    }
    if (result.error) throw result.error;
    assert.equal(result.status, 0, `${command} failed: ${result.stderr ?? ''}`);
    return result.stdout;
  };
  const locks = selections.map(({lock}) => digest(readFileSync(join(root, lock))));
  const preparation = join(output, 'rustsec');
  const revision = run('bash', [join(root, 'vendor/toko-tooling/scripts/ci/prepare-rustsec-db.sh'), 'online', advisorySource, preparation]).trim();
  assert.match(revision, /^[a-f0-9]{40}$/);
  assert.equal(readFileSync(join(preparation, 'revision'), 'utf8').trim(), revision);
  const database = join(preparation, 'db');
  const checkDatabase = () => {
    assert.equal(run('git', ['-C', database, 'rev-parse', 'HEAD']).trim(), revision, 'database revision changed');
    assert.equal(run('git', ['-C', database, 'status', '--porcelain', '--untracked-files=all']).trim(), '', 'database contains uncommitted input');
  };
  checkDatabase();
  const audit = {schema: 1, source: advisorySource, revision, observed_at: new Date().toISOString(), preparation_sha256: digest(readFileSync(join(preparation, 'prepare.log'))), reports: []};
  for (const [index, {lock, report}] of selections.entries()) {
    const bytes = run('cargo', ['audit', '--no-fetch', '--db', database, '--file', join(root, lock), '--json'], report);
    admitReport(bytes, revision);
    assert.equal(digest(readFileSync(join(root, lock))), locks[index], 'lock changed during audit');
    audit.reports.push({lock, report, lock_sha256: locks[index], sha256: digest(bytes)});
  }
  checkDatabase();
  validateReleaseAudit(audit, output, root);
  return audit;
}

// Publication refuses old receipts with an unbound cache audit. This checks
// evidence, not the shared cache or an inferred claim of perpetual freshness.
export function validateReleaseAudit(audit, output, root = process.cwd(), now = Date.now()) {
  assert.equal(audit?.schema, 1, 'fresh bound release audit required');
  assert.equal(audit.source, advisorySource);
  assert.match(audit.revision, /^[a-f0-9]{40}$/);
  assert.equal(readFileSync(join(output, 'rustsec/revision'), 'utf8').trim(), audit.revision, 'prepared revision differs');
  assert.equal(digest(readFileSync(join(output, 'rustsec/prepare.log'))), audit.preparation_sha256, 'preparation evidence differs');
  const observed = Date.parse(audit.observed_at);
  assert(Number.isFinite(observed) && observed <= now && now - observed <= maximumAgeMs, 'audit observation expired or invalid');
  assert(Array.isArray(audit.reports) && audit.reports.length === selections.length, 'complete audit roster required');
  for (const [index, selected] of selections.entries()) {
    const evidence = audit.reports[index];
    assert.equal(evidence.lock, selected.lock);
    assert.equal(evidence.report, selected.report);
    assert.equal(evidence.lock_sha256, digest(readFileSync(join(root, selected.lock))), 'audit lock differs');
    const bytes = readFileSync(join(output, selected.report));
    assert.equal(evidence.sha256, digest(bytes), 'audit report differs');
    admitReport(bytes, audit.revision);
  }
}
