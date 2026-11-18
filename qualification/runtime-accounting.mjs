// Build an observation-only variant from this working tree. Production Rust and
// the normal rollback Wasm have no callback registry inspection interface.
import {execFileSync} from 'node:child_process';
import {appendFileSync, copyFileSync, mkdirSync, mkdtempSync} from 'node:fs';
import {dirname, join, resolve} from 'node:path';

const target = resolve('target/runtime-accounting');
mkdirSync(target, {recursive: true});
const source = mkdtempSync(join(target, 'source-'));
const files = execFileSync('git', ['ls-files', '--cached', '--others', '--exclude-standard', '-z', '--', 'Cargo.toml', 'Cargo.lock',
  'LICENSE.md', 'crates', 'qualification'], {encoding: 'utf8'}).split('\0').filter(Boolean);
for (const file of files) {
  const path = join(source, file);
  mkdirSync(dirname(path), {recursive: true});
  copyFileSync(file, path);
}
const call = join(source, 'crates/ic-platform-runtime/src/call');
appendFileSync(join(call, 'registry/mod.rs'), `
impl<T> Registry<T> {
    pub(super) fn observed_count(&self) -> usize { self.entries.len() }
}
`);
appendFileSync(join(call, 'future.rs'), `
pub(super) fn observed_count() -> usize {
    CALLBACKS.with(|entries| entries.borrow().observed_count())
}
`);
appendFileSync(join(call, 'mod.rs'), `
/// Count occupied callback registrations in the private instrumented fixture.
pub fn observed_count() -> usize { future::observed_count() }
`);
appendFileSync(join(source, 'crates/ic-platform-runtime/src/executor/mod.rs'), `
/// Count parked executor tasks in the private instrumented fixture.
pub fn observed_tasks() -> usize { EXECUTOR.with(|executor| executor.borrow().tasks.len()) }
`);
appendFileSync(join(source, 'qualification/runtime-rollback/src/timer.rs'), `
#[candidate_runtime::query(runtime = candidate_runtime, name = "registry_size", guard = admit,
    decode = empty, max_bytes = 0, encode = encode, on_error = fail)]
fn registry_size((): ()) -> Vec<u8> {
    let mut counts = candidate_runtime::call::observed_count().to_le_bytes().to_vec();
    counts.extend_from_slice(&candidate_runtime::executor::observed_tasks().to_le_bytes());
    counts
}
`);
execFileSync('cargo', ['build', '--offline', '--locked', '--manifest-path',
  join(source, 'qualification/Cargo.toml'), '-p', 'runtime-rollback', '--release',
  '--target', 'wasm32-unknown-unknown', '--target-dir', target], {stdio: 'inherit'});
