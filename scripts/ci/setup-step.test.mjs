import assert from 'node:assert/strict';
import {mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join, resolve} from 'node:path';
import {spawnSync} from 'node:child_process';

const script = resolve('scripts/ci/setup-step.sh');
const cwd = mkdtempSync(join(tmpdir(), 'platform-setup-test-'));
const evidence = join(cwd, 'target/ci-setup');
const log = phase => readFileSync(join(evidence, phase + '.log'), 'utf8');
const run = (phase, command, env = {}) => spawnSync('bash', [script, phase, 'bash', '-c', command], {
  cwd, encoding: 'utf8', env: {...process.env, ...env},
});

try {
  mkdirSync(join(cwd, 'target/.attempt'), {recursive: true});
  writeFileSync(join(cwd, 'target/.attempt/download.log'), [
    'download failed: HTTP 503', 'token=excluded', '-----BEGIN PRIVATE KEY-----',
    'opaque-key-bytes', '-----END PRIVATE KEY-----', '',
  ].join('\n'));
  assert.equal(run('download', 'cat target/.attempt/download.log; exit 27').status, 27);
  const download = log('download');
  assert.match(download, /HTTP 503/);
  assert.match(download, /status: 27/);
  assert.doesNotMatch(download, /excluded|opaque-key-bytes/);

  const marker = 'synthetic-setup-fixture-value';
  const redacted = run('redacted', [
    'echo "$FIXTURE_ACCESS_KEY"', 'echo "prefix-${FIXTURE_ACCESS_KEY}-suffix"',
    'echo "Bearer opaque"', 'echo "/asset?signature=opaque"', 'echo "HTTP 503"', 'exit 23',
  ].join('\n'), {FIXTURE_ACCESS_KEY: marker});
  assert.equal(redacted.status, 23);
  for (const text of [redacted.stdout, log('redacted')]) {
    assert.match(text, /HTTP 503/);
    assert(!text.includes(marker));
    assert.doesNotMatch(text, /Bearer opaque|signature=opaque/);
  }

  assert.equal(run('empty', 'exit 9').status, 9);
  assert.match(log('empty'), /no usable diagnostic output/);
  assert.equal(run('bounded', "yes 'failure' | head -n 20000; exit 8").status, 8);
  assert(Buffer.byteLength(log('bounded')) < 34000);

  symlinkSync(join(cwd, 'outside'), join(evidence, 'refused.log'));
  const refused = run('refused', 'echo failure; exit 19');
  assert.equal(refused.status, 19);
  assert.match(refused.stderr, /refused symlink/);
  assert.equal(run('success', 'exit 0').status, 0);
  console.log('Setup diagnostics passed');
} finally {
  rmSync(cwd, {recursive: true, force: true});
}
