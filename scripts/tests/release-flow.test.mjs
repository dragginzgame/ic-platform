import assert from 'node:assert/strict';
import {test} from 'node:test';
import {readdirSync, symlinkSync, mkdtempSync, mkdirSync, readFileSync, writeFileSync, existsSync, rmSync, appendFileSync, copyFileSync, chmodSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {tmpdir} from 'node:os';
import {join, resolve} from 'node:path';
import {spawnSync} from 'node:child_process';
import {releaseCatalog, qualifyRelease, validateQualificationReceipt} from '../qualify-release.mjs';
const source = 'a'.repeat(40);
function fixture(t, fault) {
  const root = mkdtempSync(join(tmpdir(), 'platform-release-flow-'));
  t.after(() => rmSync(root, {recursive: true, force: true}));
  mkdirSync(join(root, 'ci'));
  const profiles = {repository: ['common', 'repository'], protocol: ['protocol', 'consumer'],
    runtime: ['runtime'], platform: ['platform', 'consumer']};
  writeFileSync(join(root, 'ci/release-profiles.json'), JSON.stringify(profiles));
  writeFileSync(join(root, 'ci/types-release-profiles.json'), JSON.stringify([
    {id: 'types', owner: 'types', command: ['make', 'types', 'common']},
  ]));
  const goals = ['types', 'common', 'repository', 'protocol', 'consumer', 'runtime', 'platform'];
  writeFileSync(join(root, 'Makefile'), `.PHONY: ${goals.join(' ')} prerequisite\n` +
    'prerequisite:\n\t@echo prerequisite >> dispatch\n' + goals.map(name =>
      `${name}: prerequisite\n\t@echo ${name} >> dispatch\n${fault === name ? '\t@exit 1\n' : ''}`).join(''));
  const output = join(root, 'evidence');
  const calls = [];
  let checks = 0;
  const execute = (command, args, options) => {
    calls.push([command, args]);
    if (command === 'make') return spawnSync(command, args, options);
    if (command === 'bash') {
      if (args[0].endsWith('/check-make-execution.sh')) return spawnSync(command, [resolve(args[0])], options);
      checks++;
      return {status: fault === 'dirty' || (fault === 'changed' && checks === 2) ? 1 : 0, stdout: '', stderr: ''};
    }
    if (command === 'git') return {status: 0, stdout: source};
    if (command === 'rustc') return {status: 0, stdout: 'rustc fixture'};
    throw Error(`unexpected command ${command}`);
  };
  const audit = () => {
    if (fault === 'audit') throw Error('audit unavailable');
    return {fixture: true};
  };
  const validateAudit = value => assert.deepEqual(value, {fixture: true});
  return {root, output, execute, audit, validateAudit, calls};
}
test('complete catalog dispatch uses one serial Make graph and deduplicates prerequisites', t => {
  const f = fixture(t);
  const receipt = qualifyRelease(f);
  const lines = readFileSync(join(f.root, 'dispatch'), 'utf8').trim().split('\n');
  assert.deepEqual(lines, ['prerequisite', 'types', 'common', 'repository', 'protocol', 'consumer', 'runtime', 'platform']);
  assert.equal(f.calls.filter(([cmd]) => cmd === 'make').length, 1);
  assert.deepEqual(receipt.targets, releaseCatalog(f.root).targets);
  validateQualificationReceipt(receipt, f.output, {...f, commit: source, rust: 'rustc fixture'});
  assert(existsSync(join(f.output, 'timings.json')));
});
for (const fault of ['dirty', 'audit', 'protocol', 'changed']) {
  test(`refuses ${fault}, retains failure evidence, and a fresh run can recover`, t => {
    const f = fixture(t, fault);
    assert.throws(() => qualifyRelease(f));
    assert(!existsSync(join(f.output, 'receipt.json')));
    if (fault === 'protocol') {
      assert(existsSync(join(f.output, 'qualification.log')));
      assert(!readFileSync(join(f.root, 'dispatch'), 'utf8').includes('platform'));
    }
    const recovered = fixture(t);
    assert.equal(qualifyRelease(recovered).schema, 2);
  });
}
for (const fault of ['legacy', 'source', 'compiler', 'host', 'catalog', 'types', 'targets', 'log', 'missing-log']) {
  test(`publication receipt rejects ${fault} changes`, t => {
    const f = fixture(t);
    const receipt = qualifyRelease(f);
    if (fault === 'legacy') delete receipt.schema;
    if (fault === 'source') receipt.commit = 'b'.repeat(40);
    if (fault === 'compiler') receipt.rust = 'other';
    if (fault === 'host') receipt.host = 'other';
    if (fault === 'catalog') appendFileSync(join(f.root, 'ci/release-profiles.json'), '\n');
    if (fault === 'types') appendFileSync(join(f.root, 'ci/types-release-profiles.json'), '\n');
    if (fault === 'targets') receipt.targets.pop();
    if (fault === 'log') appendFileSync(join(f.output, 'qualification.log'), 'tampered');
    if (fault === 'missing-log') rmSync(join(f.output, 'qualification.log'));
    assert.throws(() => validateQualificationReceipt(receipt, f.output, {...f, commit: source, rust: 'rustc fixture'}));
  });
}
test('catalog rejects injected Make options and assignments before dispatch', t => {
  for (const goal of ['--eval=bad', 'PACKAGE=other', '../outside']) {
    const f = fixture(t);
    const path = join(f.root, 'ci/release-profiles.json');
    const value = JSON.parse(readFileSync(path)); value.platform = [goal];
    writeFileSync(path, JSON.stringify(value));
    assert.throws(() => qualifyRelease(f));
    assert(!f.calls.some(([command]) => command === 'make'));
  }
});
test('both native entrypoints share the unit-test prerequisite in real Make dispatch', t => {
  const root = mkdtempSync(join(tmpdir(), 'platform-native-dispatch-'));
  t.after(() => rmSync(root, {recursive: true, force: true}));
  const override = join(root, 'override.mk');
  writeFileSync(override, ['qualify-native-read', 'qualify-native-signed', 'qualification-tooling-check', 'ic-tools-check']
    .map(name => `${name}:\n\t@true\n`).join('') + 'test-native:\n\t@echo native-unit-dispatch\n');
  const result = spawnSync('make', ['--no-print-directory', '--jobs=1', '-f', 'Makefile', '-f', override,
    'qualify-native-read', 'qualify-native-signed'], {encoding: 'utf8'});
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout.match(/native-unit-dispatch/g)?.length, 1);
});

function publicationFixture(t, fault) {
  const f = fixture(t);
  const digest = bytes => createHash('sha256').update(bytes).digest('hex');
  const audit = output => {
    mkdirSync(join(f.root, 'qualification'));
    mkdirSync(join(output, 'rustsec'));
    writeFileSync(join(output, 'rustsec/revision'), source);
    writeFileSync(join(output, 'rustsec/prepare.log'), 'fixture database');
    const reports = ['Cargo.lock', 'qualification/Cargo.lock'].map((lock, index) => {
      writeFileSync(join(f.root, lock), 'fixture lock');
      const report = index ? 'qualification-audit.json' : 'audit.json';
      const bytes = JSON.stringify({database: {'last-commit': null, 'advisory-count': 1},
        vulnerabilities: {found: false, count: 0, list: []}});
      writeFileSync(join(output, report), bytes);
      return {lock, report, lock_sha256: digest('fixture lock'), sha256: digest(bytes)};
    });
    return {schema: 1, source: 'https://github.com/RustSec/advisory-db.git', revision: source,
      observed_at: new Date().toISOString(), preparation_sha256: digest('fixture database'), reports};
  };
  qualifyRelease({...f, audit, validateAudit: () => {}});
  mkdirSync(join(f.root, 'scripts'));
  for (const name of ['qualify-release.mjs', 'release-audit.mjs', 'release-publication-admission.mjs'])
    copyFileSync(join('scripts', name), join(f.root, 'scripts', name));
  writeFileSync(join(f.root, 'ci/cargo-release-groups.json'), JSON.stringify({schema: 1,
    groups: [{name: 'platform', members: ['ic-platform']}]}));
  const vendor = join(f.root, 'vendor/toko-tooling/scripts/ci'); mkdirSync(vendor, {recursive: true});
  writeFileSync(join(vendor, 'check-release-source.sh'), fault === 'dirty' ? 'exit 1\n' : 'exit 0\n');
  writeFileSync(join(vendor, 'check-cargo-package-provenance.sh'), fault === 'provenance' ? 'exit 1\n' : 'echo provenance >> effects\n');
  const bin = join(f.root, 'bin'); mkdirSync(bin);
  // Simulate external effects only: execute the real consumer admission process.
  const tool = `#!/usr/bin/env node
import {appendFileSync, writeFileSync, mkdirSync} from 'node:fs';
import {basename, join} from 'node:path';
const tool=basename(process.argv[1]); const args=process.argv.slice(2);
appendFileSync('effects', JSON.stringify([tool,...args])+'\\n');
const fault=process.env.FIXTURE_FAULT;
if(tool==='git') {
 if(args[0]==='remote') console.log(fault==='repository'?'https://example.invalid/repo':'https://github.com/dragginzgame/ic-platform.git');
 else if(args[0]==='cat-file') console.log(fault==='lightweight'?'commit':'tag');
 else if(args[0]==='ls-remote') console.log((fault==='remote-tag'?'b'.repeat(40):'${source}')+' refs/tags/platform-v0.34.15^{}');
 else console.log('${source}');
} else if(tool==='rustc') console.log('rustc fixture');
else if(tool==='cargo') {
 if(args[0]==='metadata') console.log(JSON.stringify({workspace_members:['local'],packages:[{id:'local',name:'ic-platform',version:'0.34.15',publish:fault==='ineligible'?[]:['crates-io'],features:{default:fault==='defaults'?['extra']:[]},repository:'https://github.com/dragginzgame/ic-platform'}]}));
 if(args[0]===fault) process.exit(1);
} else if(tool==='curl') {
 writeFileSync(args[args.indexOf('--output')+1],JSON.stringify({vers:fault==='published'?'0.34.15':'0.34.14'})+'\\n');
 process.stdout.write(fault==='registry'?'503':'200');
} else if(tool==='tar') {
 const dir=join(args[args.indexOf('-C')+1],'ic-platform-0.34.15');mkdirSync(dir);writeFileSync(join(dir,'Cargo.toml'),'[package]\\nname="fixture"\\nversion="0.1.0"\\n');
}
`;
  for (const name of ['cargo', 'git', 'rustc', 'curl', 'tar']) {
    const file = join(bin, name); writeFileSync(file, tool); chmodSync(file, 0o755);
  }
  if (fault === 'tamper') appendFileSync(join(f.output, 'qualification.log'), 'changed');
  if (fault === 'lock') appendFileSync(join(f.root, 'Cargo.lock'), 'changed');
  return {...f, run: () => spawnSync(process.execPath, ['scripts/release-publication-admission.mjs'], {
    cwd: f.root, encoding: 'utf8', env: {...process.env, PATH: `${bin}:${process.env.PATH}`,
      RELEASE_RECEIPT: join(f.output, 'receipt.json'), RELEASE_GROUP: fault === 'group' ? 'runtime' : 'platform',
      CARGO_PUBLISH_SELECTION: fault === 'roster' ? 'ic-platform ic-platform-types' : 'ic-platform', FIXTURE_FAULT: fault ?? ''},
  })};
}
test('actual publication admission dispatch preserves archive target/features and provenance', t => {
  const f = publicationFixture(t);
  const result = f.run(); assert.equal(result.status, 0, result.stderr);
  const lines = readFileSync(join(f.root, 'effects'), 'utf8').trim().split('\n');
  assert.equal(lines.at(-1), 'provenance');
  const calls = lines.slice(0, -1).map(JSON.parse);
  assert.deepEqual(calls.filter(c => c[0] === 'cargo').map(c => c[1]), ['metadata', 'package', 'clippy', 'clippy', 'test']);
  const clippy = calls.filter(c => c[1] === 'clippy');
  assert(clippy.every(c => c.includes('--all-features') && c.includes('--all-targets') && c.includes('--locked')));
  assert(clippy[1].includes('wasm32-unknown-unknown'));
  assert(!calls.some(c => c[1] === 'publish'));
});
for (const fault of ['dirty', 'tamper', 'lock', 'group', 'roster', 'repository', 'lightweight', 'remote-tag', 'ineligible', 'defaults', 'registry', 'published', 'package', 'clippy', 'test', 'provenance']) {
  test(`actual publication admission refuses ${fault} before downstream effects`, t => {
    const f = publicationFixture(t, fault);
    const result = f.run(); assert.notEqual(result.status, 0);
    const effects = existsSync(join(f.root, 'effects')) ? readFileSync(join(f.root, 'effects'), 'utf8') : '';
    assert(!effects.includes('provenance'));
    if (!['package', 'clippy', 'test', 'provenance'].includes(fault)) assert(!effects.includes('"package"'));
    assert(!effects.includes('"publish"'));
  });
}


test('an occupied evidence directory is never overwritten or treated as a new run', t => {
  const f = fixture(t);
  qualifyRelease(f);
  const receipt = readFileSync(join(f.output, 'receipt.json'));
  const log = readFileSync(join(f.output, 'qualification.log'));
  assert.throws(() => qualifyRelease(f));
  assert.deepEqual(readFileSync(join(f.output, 'receipt.json')), receipt);
  assert.deepEqual(readFileSync(join(f.output, 'qualification.log')), log);
});

for (const mode of ['-n', '-i']) {
  test(`collector reaches the shared execution guard before dispatch under ${mode}`, t => {
    const f = fixture(t);
    assert.throws(() => qualifyRelease({...f, execute: (command, args, options) =>
      f.execute(command, args, {...options, env: {...process.env, MAKEFLAGS: mode, MFLAGS: mode}})}));
    assert(!f.calls.some(([command]) => command === 'make'));
    assert(!existsSync(join(f.output, 'receipt.json')));
  });
}

test('isolated export uses the shared runner and a mixed release range falls back', t => {
  const f = fixture(t);
  const {root} = f;
  function put(path, value) {
    mkdirSync(join(root, path, '..'), {recursive: true});
    writeFileSync(join(root, path), value);
  }
  function git(...args) {
    const result = spawnSync('git', args, {cwd: root, encoding: 'utf8'});
    assert.equal(result.status, 0, result.stderr);
    return result.stdout.trim();
  }
  const catalog = JSON.parse(readFileSync(join(root, 'ci/release-profiles.json')));
  catalog.platform.push('qualify-stable-structures');
  put('ci/release-profiles.json', JSON.stringify(catalog));
  appendFileSync(join(root, 'Makefile'), '\nqualify-stable-structures:\n\t@echo stable >> dispatch\n');
  for (const file of ['Cargo.lock','qualification/Cargo.lock','scripts/qualify-release.mjs',
    'crates/ic-platform/tests/stable_structures.rs']) put(file, 'fixture\n');
  put('crates/ic-platform/Cargo.toml', 'version = "0.34.17"\n');
  const lib = 'crates/ic-platform/src/lib.rs';
  const alias = '#[cfg(feature = "stable-structures")]\npub use ic_stable_structures as stable_structures;\n';
  put(lib, '\n' + alias);
  const vendor = 'vendor/toko-tooling/scripts/ci';
  for (const name of ['run-validation-targets.sh','check-make-execution.sh','verify-file-checksum.sh',
    'validation-input-checkpoint.pl','release-validation-plan.pl','ValidationPlan.pm']) {
    put(vendor + '/' + name, readFileSync(join(vendor, name)));
  }
  git('init', '-q');
  git('config', 'user.email', 'fixture@example.invalid');
  git('config', 'user.name', 'Fixture');
  git('add', '.');
  git('commit', '-qm', 'baseline');
  git('tag', '-a', 'platform-v0.34.16', '-m', 'qualified baseline');
  put(lib, '\n/// Storage collections.\n' + alias);
  git('commit', '-qam', 'document isolated export');
  const commit = git('rev-parse', 'HEAD');
  const execute = (cmd, args, opts) => {
    if (cmd === 'git' || args[0]?.endsWith('run-validation-targets.sh')) return spawnSync(cmd, args, opts);
    return f.execute(cmd, args, opts);
  };
  const options = {...f, execute, group: 'platform'};
  const receipt = qualifyRelease(options);
  assert.equal(receipt.schema, 3);
  assert.deepEqual(receipt.targets, ['qualify-stable-structures']);
  assert.equal(readFileSync(join(root, 'dispatch'), 'utf8'), 'stable\n');
  git('tag', '-a', 'platform-v0.34.17', '-m', 'candidate');
  validateQualificationReceipt(receipt, f.output, {...f, commit, rust: receipt.rust});
  const aliasOutput = join(root, 'evidence-alias');
  symlinkSync(f.output, aliasOutput, 'dir');
  const validate = () => validateQualificationReceipt(receipt, aliasOutput, {...f, commit, rust: receipt.rust});
  validate();
  const run = readdirSync(f.output).find(name => name.startsWith('validation.'));
  const timings = join(f.output, run, 'timings.tsv');
  const original = readFileSync(timings, 'utf8');
  const log = original.trimEnd().split('\t').at(-1);
  const outside = join(f.output, 'outside.log');
  copyFileSync(log, outside);
  writeFileSync(timings, original.replace(log, outside));
  assert.throws(validate, /outside the selected evidence directory/);
  writeFileSync(timings, original);
  appendFileSync(join(f.output, 'plan.json'), ' ');
  assert.throws(() => validateQualificationReceipt(receipt, f.output, {...f, commit, rust: receipt.rust}));
  put('shared-input', 'unmapped');
  git('add', 'shared-input');
  git('commit', '-qm', 'shared change');
  const full = qualifyRelease({...options, output: join(root, 'mixed')});
  assert.deepEqual(full.targets, releaseCatalog(root).targets);
});
