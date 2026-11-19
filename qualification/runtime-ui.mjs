// Compile maintained fixtures against Runtime; codegen cases build actual Wasm.
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const root = fileURLToPath(new URL('..', import.meta.url));
const groups = {
  input: { command: 'check', alias: 'renamed-runtime', feature: 'call-ic0', cases: [['accepted', null], ['invalid-decoder', 'E0631'], ['borrowed-input', 'E0597'], ['borrowed-handler', 'E0373']] },
  exports: { command: 'build', alias: 'candidate-runtime', feature: 'entrypoint-macros', cases: [['accepted', null], ['invalid-option', 'manual_reply is exclusive'], ['invalid-handler', 'update requires a safe async nongeneric function'], ['invalid-decoder', 'E0631'], ['returning-error', 'E0308'], ['duplicate-export', 'already defined'], ['async-query', 'requires a safe'], ['async-lifecycle', 'requires a safe'], ['lifecycle-reply', 'lifecycle exports cannot encode a reply'], ['invalid-encoder', 'E0631'], ['query-manual', 'requires a safe']] },
};
const selection = process.argv[2];
assert.ok(Object.hasOwn(groups, selection), 'select input or exports');
assert.equal(process.argv.length, 3);
const group = groups[selection];
mkdirSync(join(root, 'target/tmp'), { recursive: true });
const scratch = mkdtempSync(join(root, 'target/tmp/runtime-ui-'));
try {
  for (const [name, expected] of group.cases) {
    const directory = join(scratch, name, 'runtime-ui-probe');
    mkdirSync(directory, { recursive: true });
    const source = selection === 'exports' && name === 'accepted'
      ? join(root, 'qualification/runtime-exports/src/exports.rs')
      : join(root, `qualification/ui/runtime-${selection}`, `${name}.rs`);
    const manifest = join(directory, 'Cargo.toml');
    writeFileSync(manifest, `[workspace]\n[package]\nname="runtime-ui-probe"\nversion="0.1.0"\nedition="2024"\npublish=false\n[lib]\n${group.command === 'build' ? 'crate-type=["cdylib"]\n' : ''}path=${JSON.stringify(source)}\n[features]\nraw=[]\n[dependencies]\n${group.alias}={package="ic-platform-runtime",path=${JSON.stringify(join(root, 'crates/ic-platform-runtime'))},default-features=false,features=["${group.feature}"]}\n[lints.rust]\nunsafe_code="deny"\nunfulfilled_lint_expectations="deny"\n`);
    if (selection === 'exports' && expected === null) {
      // Compile the generated handlers without the raw IC0 fixture on the host.
      const host = spawnSync('cargo', ['build', '--offline', '--manifest-path', manifest,
        '--target-dir', join(root, 'target/runtime-exports-ui')], {encoding: 'utf8', timeout: 60000});
      assert.ifError(host.error);
      assert.equal(host.status, 0, host.stderr);
    }
    const result = spawnSync('cargo', [group.command, '--offline', '--manifest-path', manifest, '--target', 'wasm32-unknown-unknown', '--target-dir', join(root, `target/runtime-${selection}-ui`), '--message-format=json'], { encoding: 'utf8', timeout: 60000, maxBuffer: 8 * 1024 * 1024 });
    assert.ifError(result.error);
    assert.equal(result.signal, null);
    const records = result.stdout.trim().split('\n').filter(Boolean).map(line => JSON.parse(line));
    const errors = records.filter(r => r.reason === 'compiler-message' && r.message.level === 'error');
    if (expected === null) {
      assert.equal(result.status, 0, result.stderr + JSON.stringify(errors));
      if (group.command === 'build') assert.ok(records.some(r => r.reason === 'compiler-artifact' && r.filenames.some(p => p.endsWith('.wasm'))));
    } else {
      assert.notEqual(result.status, 0, `${name} unexpectedly compiled`);
      assert.ok(errors.some(({ message }) => expected.startsWith('E')
        ? message.code?.code === expected && message.spans.some(s => s.is_primary && resolve(s.file_name) === resolve(source))
        : message.message.includes(expected)), JSON.stringify({ name, errors, stderr: result.stderr }));
    }
    assert.ok(records.some(r => r.reason === 'build-finished' && r.success === (expected === null)), result.stderr);
  }
  console.log(JSON.stringify({ admission: 'passed', selection, accepted: 1, rejected: group.cases.length - 1 }));
} finally {
  rmSync(scratch, { recursive: true, force: true });
}
