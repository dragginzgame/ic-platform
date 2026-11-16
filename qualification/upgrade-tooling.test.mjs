import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, readFileSync, rmSync, chmodSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const script = fileURLToPath(new URL('../scripts/upgrade-canister.mjs', import.meta.url));
for (const scenario of ['preview', 'success', 'stopped', 'unknown', 'public-only', 'wrong-target', 'stop-fails', 'stopping', 'timeout', 'install-fails', 'wrong-hash', 'start-fails']) {
  test(`upgrade operator workflow: ${scenario}`, () => {
    const dir = mkdtempSync(join(tmpdir(), 'ic-upgrade-tool-'));
    try {
      const wasm = join(dir, 'module with spaces.wasm');
      writeFileSync(wasm, Buffer.from([0, 97, 115, 109, 1, 0, 0, 0]));
      const state = join(dir, 'state.json');
      writeFileSync(state, JSON.stringify({ status: scenario === 'stopped' ? 'Stopped' : 'Running', hash: 'old', calls: [] }));
      const fake = join(dir, 'icp');
      writeFileSync(fake, `#!${process.execPath}
const fs = require('node:fs'), crypto = require('node:crypto');
const path = process.env.UPGRADE_TEST_STATE, mode = process.env.UPGRADE_TEST_MODE;
const args = process.argv.slice(2), verb = args[1];
const state = JSON.parse(fs.readFileSync(path)); state.calls.push(args);
const save = () => fs.writeFileSync(path, JSON.stringify(state)); save();
if (verb === 'status') console.log(mode === 'unknown' ? 'unrecognized' : JSON.stringify({ id: mode === 'wrong-target' ? '2vxsx-fae' : args[2], status: mode === 'public-only' ? undefined : state.status, module_hash: '0x' + state.hash }));
else if (verb === 'stop') {
  if (mode === 'stop-fails') process.exit(1);
  state.status = mode === 'stopping' || mode === 'timeout' ? 'Stopping' : 'Stopped'; save();
  if (mode === 'timeout') setInterval(() => {}, 1000);
} else if (verb === 'install') {
  if (mode === 'install-fails') process.exit(1);
  state.hash = mode === 'wrong-hash' ? 'wrong' : crypto.createHash('sha256').update(fs.readFileSync(args[args.indexOf('--wasm') + 1])).digest('hex'); save();
} else if (verb === 'start') {
  if (mode === 'start-fails') process.exit(1);
  state.status = 'Running'; save();
} else process.exit(2);
`);
      chmodSync(fake, 0o755);
      const args = [script, '--network', 'local', '--identity', 'operator', '--canister', 'aaaaa-aa', '--wasm', wasm, '--timeout-seconds', '1'];
      if (scenario !== 'preview') args.push('--execute');
      const result = spawnSync(process.execPath, args, { encoding: 'utf8', timeout: 8000,
        env: { ...process.env, PATH: `${dir}:${process.env.PATH}`, UPGRADE_TEST_STATE: state, UPGRADE_TEST_MODE: scenario } });
      assert.ifError(result.error);
      const calls = JSON.parse(readFileSync(state)).calls;
      const verbs = calls.map(args => args[1]);
      if (scenario === 'preview') {
        assert.equal(result.status, 0); assert.deepEqual(calls, []);
      } else if (scenario === 'success' || scenario === 'stopped') {
        assert.equal(result.status, 0, result.stderr);
        assert.deepEqual(verbs, scenario === 'success' ? ['status', 'stop', 'status', 'install', 'status', 'start', 'status'] : ['status', 'status', 'install', 'status', 'start', 'status']);
        const install = calls.find(args => args[1] === 'install');
        assert.equal(install[install.indexOf('--mode') + 1], 'upgrade');
        assert.equal(install[install.indexOf('--wasm') + 1], wasm);
        for (const args of calls) {
          assert.equal(args[0], 'canister');
          assert.deepEqual(args.slice(2, 7), ['aaaaa-aa', '--identity', 'operator', '--network', 'local']);
          if (args[1] === 'status') assert.ok(args.includes('--json'));
        }
        assert.equal(install[install.indexOf('--args-format') + 1], 'hex');
        assert.equal(install[install.indexOf('--args') + 1], '');
      } else {
        assert.notEqual(result.status, 0);
        assert.match(result.stderr, /Upgrade halted during/);
        if (['unknown', 'public-only', 'wrong-target', 'stop-fails', 'stopping', 'timeout'].includes(scenario)) assert.ok(!verbs.includes('install'));
        if (scenario !== 'start-fails') assert.ok(!verbs.includes('start'));
      }
    } finally { rmSync(dir, { recursive: true, force: true }); }
  });
}
