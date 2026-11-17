import assert from 'node:assert/strict';
import {test} from 'node:test';
import {readFileSync, writeFileSync, mkdtempSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {spawnSync} from 'node:child_process';
import {releaseCatalog} from '../qualify-release.mjs';
// These are wiring contracts: the owning targets execute the actual native/Wasm
// fixtures; this guard prevents a green workflow silently omitting them again.
test('release flow regressions stay in CI and focused tooling checks, outside product releases', () => {
  const targets = releaseCatalog().targets;
  assert(!targets.includes('check-release-evidence'));
  assert(!targets.includes('check-types-publication'));
  assert.match(readFileSync('.github/workflows/ci.yml', 'utf8'), /run: make lint-workflows check-release-evidence/);
  assert.match(readFileSync('.github/workflows/types-release.yml', 'utf8'), /run: make check-types-publication/);
  assert.match(readFileSync('Makefile', 'utf8'), /^check-types-publication: check-release-evidence$/m);
});
test('regular native CI selects the new owning checks on every host', () => {
  const workflow = readFileSync('.github/workflows/ci.yml', 'utf8').split('  native:')[1];
  assert.match(workflow, /\[ubuntu-24.04, macos-15, macos-15-intel\]/);
  assert.match(workflow, /make install-tools tools-check/);
  assert.match(workflow, /run: make qualify-native-signed/);
  assert.match(workflow, /run: make qualify-caffeine qualify-management-ledger/);
});
test('normal Make ignores ambient ICP executable and admits the local pin', t => {
  const directory = mkdtempSync(join(tmpdir(), 'platform-make-wiring-'));
  t.after(() => rmSync(directory, {recursive: true, force: true}));
  const file = join(directory, 'probe.mk');
  writeFileSync(file, 'qualification-wiring-probe:\n\t@printf "%s\\n" "$(ICP_CLI)" "$(IC_TOOLS)" "$(LOCAL_TOOL_INSTALL_TARGETS)" "$(LOCAL_TOOL_CHECK_TARGETS)"\n');
  const result = spawnSync('make', ['--no-print-directory', '-f', 'Makefile', '-f', file, 'qualification-wiring-probe', 'ICP_CLI=/untrusted/icp'], {
    encoding: 'utf8', timeout: 10000,
  });
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(result.stdout.trim().split('\n'), [`${process.cwd()}/.tools/ic/bin/icp`, 'icp', 'install-ic-tools', 'ic-tools-check']);
  const make = readFileSync('Makefile', 'utf8');
  assert.match(make, /qualify-native-signed:.*ic-tools-check/);
  assert.match(make, /qualify-release:.*ic-tools-check/);
});
