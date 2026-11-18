import assert from 'node:assert/strict';
import {test} from 'node:test';
import {existsSync, readFileSync, writeFileSync, mkdtempSync, rmSync} from 'node:fs';
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
  const workflow = readFileSync('.github/workflows/ci.yml', 'utf8');
  assert.match(workflow.split('  native:')[1], /run: make check-release-evidence/);
  assert.equal((workflow.match(/run: make check-release-evidence/g) ?? []).length, 1);
  assert(!existsSync('.github/workflows/types-release.yml'));
  assert.doesNotMatch(readFileSync('Makefile', 'utf8'), /^check-types-publication:/m);
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

test('actual protocol graph commands admit Candid only in the native oracle fixture', t => {
  const directory = mkdtempSync(join(tmpdir(), 'platform-oracle-graph-'));
  t.after(() => rmSync(directory, {recursive: true, force: true}));
  const probe = join(directory, 'capture.cjs');
  const log = join(directory, 'commands.jsonl');
  const override = join(directory, 'override.mk');
  writeFileSync(probe, `require('node:fs').appendFileSync(${JSON.stringify(log)}, JSON.stringify(process.argv.slice(2))+'\\n');\n`);
  writeFileSync(override, 'qualification-tooling-check:\n\t@true\n');
  for (const target of ['wasm32-unknown-unknown', 'x86_64-unknown-linux-gnu', 'aarch64-apple-darwin', 'x86_64-apple-darwin']) {
    writeFileSync(log, '');
    const result = spawnSync('make', ['--no-print-directory', '-f', 'Makefile', '-f', override,
      'check-protocol-graphs', `GRAPH_TARGET=${target}`, `PROTOCOL_GRAPH_CHECK=${process.execPath} ${probe}`],
    {encoding: 'utf8', timeout: 10000});
    assert.equal(result.status, 0, result.stderr);
    const commands = readFileSync(log, 'utf8').trim().split('\n').map(JSON.parse);
    assert.equal(commands.length, 11);
    for (const args of commands) {
      const pkg = args[args.indexOf('--package') + 1];
      const forbidden = new RegExp(args[args.indexOf('--forbid-pattern') + 1]);
      const nativeOracle = pkg === 'engine-view-consumer' && !target.startsWith('wasm');
      assert.equal(forbidden.test('candid'), !nativeOracle, `${target} ${pkg}`);
      for (const excluded of ['ic-agent', 'ic-platform-runtime', 'unadmitted-oracle']) assert(forbidden.test(excluded));
      if (pkg === 'engine-view-consumer') {
        assert(args.includes('ic-platform-types=1'));
        assert(args.includes('ic-platform-protocol=1'));
        if (args.includes('marketplace')) assert(args.includes(`num-bigint=${nativeOracle ? 2 : 1}`));
      }
    }
  }
});

test('consolidated native CI retains Types and publication qualification on all hosts', () => {
  const workflow = readFileSync('.github/workflows/ci.yml', 'utf8');
  const native = workflow.split('  native:')[1];
  assert.match(native, /\[ubuntu-24.04, macos-15, macos-15-intel\]/);
  const makeTargets = new Set();
  for (const line of native.split('\n')) {
    const command = line.trim().match(/^(?:run:\s+)?make\s+([^#]+)/);
    if (command) {
      for (const arg of command[1].trim().split(/\s+/)) makeTargets.add(arg);
    }
  }
  for (const target of ['fmt-check', 'qualification-tooling-check', 'check-protocol-graphs',
    'check-types-graphs', 'check-runtime-consumer-graph', 'qualify-types',
    'qualify-types-consumer', 'qualify-types-auth-archive', 'qualify-runtime-consumer',
    'check-release-evidence']) {
    assert(makeTargets.has(target), `missing ${target}`);
  }
  assert.match(native, /repository: dragginzgame\/ic-auth/);
  assert.match(native, /ref: b78991e9833320692921bf766e51ad361c9c8c4d/);
  assert.match(native, /AUTH_SOURCE:.*target\/auth-source/);
  assert.match(native, /GRAPH_TARGET=.*rustc -vV/);
  const repository = workflow.split('  native:')[0];
  assert.match(repository, /fetch-depth: 0/);
  assert.match(repository, /run: make check-secrets/);
  assert.match(repository, /run: make lint-workflows/);
});
