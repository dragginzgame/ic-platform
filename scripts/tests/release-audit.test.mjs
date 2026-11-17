import assert from 'node:assert/strict';
import {test} from 'node:test';
import {mkdtempSync, mkdirSync, readFileSync, writeFileSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {qualifyAudit, validateReleaseAudit, advisorySource} from '../release-audit.mjs';
const revision = 'a'.repeat(40);
function fixture(t, fault) {
  const root = mkdtempSync(join(tmpdir(), 'platform-release-audit-'));
  t.after(() => rmSync(root, {recursive: true, force: true}));
  mkdirSync(join(root, 'qualification'));
  writeFileSync(join(root, 'Cargo.lock'), 'product');
  writeFileSync(join(root, 'qualification/Cargo.lock'), 'qualification');
  const output = join(root, 'output'); mkdirSync(output);
  const calls = [];
  const execute = (cmd, args) => {
    calls.push([cmd, args]);
    if (cmd === 'bash') {
      assert.equal(args[1], 'online'); assert.equal(args[2], advisorySource);
      if (fault === 'acquire') return {status: 1, stdout: '', stderr: 'offline'};
      mkdirSync(args[3]); writeFileSync(join(args[3], 'revision'), revision);
      writeFileSync(join(args[3], 'prepare.log'), `online ${advisorySource} ${revision}`);
      return {status: 0, stdout: `${revision}\n`};
    }
    if (cmd === 'git') {
      return {status: 0, stdout: args.includes('rev-parse') ? revision : fault === 'dirty' ? ' M advisory.md' : ''};
    }
    assert.equal(cmd, 'cargo'); assert(args.includes('--no-fetch'));
    assert.equal(args[args.indexOf('--db') + 1], join(output, 'rustsec/db'));
    assert(!args.includes('--ignore'));
    const database = {'last-commit': fault === 'identity' ? 'b'.repeat(40) : null, 'advisory-count': 1527};
    const vulnerabilities = {found: fault === 'vulnerable', count: fault === 'vulnerable' ? 1 : 0, list: fault === 'vulnerable' ? [{}] : []};
    if (fault === 'lock') writeFileSync(join(root, 'Cargo.lock'), 'changed');
    return {status: fault === 'exit' ? 1 : 0, stdout: JSON.stringify({database, vulnerabilities})};
  };
  return {root, output, calls, execute};
}
test('fresh isolated audits bind both locks, reports and revision', t => {
  const f = fixture(t); const audit = qualifyAudit(f.output, f);
  validateReleaseAudit(audit, f.output, f.root);
  assert.equal(f.calls.filter(([cmd]) => cmd === 'cargo').length, 2);
  assert.equal(audit.reports.length, 2);
});
for (const fault of ['acquire', 'dirty', 'identity', 'vulnerable', 'lock', 'exit']) {
  test(`refuses ${fault} and retains no passing audit`, t => {
    const f = fixture(t, fault);
    assert.throws(() => qualifyAudit(f.output, f));
    if (['acquire', 'dirty'].includes(fault)) assert(!f.calls.some(([cmd]) => cmd === 'cargo'));
    if (fault === 'exit') assert(readFileSync(join(f.output, 'audit.json')).length > 0);
  });
}
for (const fault of ['legacy', 'report', 'lock', 'missing', 'expired', 'future', 'roster', 'preparation', 'revision']) {
  test(`publication refuses ${fault} evidence`, t => {
    const f = fixture(t); let audit = qualifyAudit(f.output, f);
    if (fault === 'legacy') audit = undefined;
    if (fault === 'report') writeFileSync(join(f.output, 'audit.json'), '{}');
    if (fault === 'lock') writeFileSync(join(f.root, 'qualification/Cargo.lock'), 'new');
    if (fault === 'missing') rmSync(join(f.output, 'qualification-audit.json'));
    if (fault === 'expired') audit.observed_at = new Date(Date.now() - 86400001).toISOString();
    if (fault === 'future') audit.observed_at = new Date(Date.now() + 60000).toISOString();
    if (fault === 'roster') audit.reports.pop();
    if (fault === 'preparation') writeFileSync(join(f.output, 'rustsec/prepare.log'), 'changed');
    if (fault === 'revision') writeFileSync(join(f.output, 'rustsec/revision'), 'b'.repeat(40));
    assert.throws(() => validateReleaseAudit(audit, f.output, f.root));
  });
}
