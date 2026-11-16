// Qualification-owned asset admission and invocation; no product dependency.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {constants, openSync, closeSync, fstatSync, lstatSync, fsyncSync, readFileSync, mkdirSync, mkdtempSync, writeFileSync, chmodSync, renameSync, rmSync} from 'node:fs';
import {resolve, join} from 'node:path';
import {gunzipSync} from 'node:zlib';
import {spawnSync} from 'node:child_process';
const version = '16.1.0';
const assets = {
  'x64-linux': ['x86_64-linux', '131219d90dcf9bf6f3ed8ee02d8f55504ea24a6db382f5680675b7bd6b7ed3bf'],
  'x64-darwin': ['x86_64-darwin', 'af9ad2d781530a43556ef2d1c8f93db99a2425c6cabdc78520f61922399ed530'],
  'arm64-darwin': ['arm64-darwin', '9ae843fbb7ae6c6eb30137671c8629a80ef53b3a3652eb85b344847ac39f9fcd'],
};
const selected = assets[`${process.arch}-${process.platform}`];
assert(selected, 'unsupported qualification host');
const [platform, digest] = selected;
const parent = resolve('.tools/ic-testkit-server');
const directory = join(parent, `${version}-${platform}`);
const binary = join(directory, 'pocket-ic');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
function boundedFile(path, limit) {
  const fd = openSync(path, constants.O_RDONLY | constants.O_NOFOLLOW);
  try {
    const stat = fstatSync(fd);
    assert(stat.isFile() && stat.size <= limit, `invalid asset: ${path}`);
    return readFileSync(fd);
  } finally { closeSync(fd); }
}
function archiveBytes(bytes) {
  assert(bytes.length <= 80 * 1024 * 1024, 'archive limit');
  assert.equal(hash(bytes), digest, 'official archive checksum');
  return gunzipSync(bytes, {maxOutputLength: 512 * 1024 * 1024});
}
function directories() {
  for (const path of [resolve(".tools"), parent, directory]) {
    const stat = lstatSync(path); assert(stat.isDirectory() && !stat.isSymbolicLink(), `invalid asset directory: ${path}`);
  }
}
function check() {
  directories();
  const expected = archiveBytes(boundedFile(join(directory, 'archive.gz'), 80 * 1024 * 1024));
  assert.equal(hash(boundedFile(binary, 512 * 1024 * 1024)), hash(expected), 'executable differs from authenticated archive');
  const result = spawnSync(binary, ['--version'], {encoding:'utf8',timeout:10_000,maxBuffer:4096});
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout.trim(), `pocket-ic-server ${version}`, 'server version');
}
const [mode, ...command] = process.argv.slice(2);
if (mode === 'setup') {
  mkdirSync(parent, {recursive:true});
  for (const path of [resolve('.tools'), parent]) assert(lstatSync(path).isDirectory() && !lstatSync(path).isSymbolicLink(), 'redirected asset ancestor');
  try { lstatSync(directory); check(); process.exit(0); } catch (error) { if (error.code !== 'ENOENT') throw error; }
  const temporary = mkdtempSync(join(parent, '.prepare-'));
  try {
    const url = `https://github.com/dfinity/pocketic/releases/download/${version}/pocket-ic-${platform}.gz`;
    const response = await fetch(url, {signal:AbortSignal.timeout(120_000)});
    assert(response.ok, `download failed: ${response.status}`);
    const chunks = []; let count = 0;
    for await (const chunk of response.body) {
      count += chunk.length; assert(count <= 80 * 1024 * 1024, 'archive limit'); chunks.push(chunk);
    }
    const bytes = Buffer.concat(chunks);
    const executable = archiveBytes(bytes);
    writeFileSync(join(temporary, 'archive.gz'), bytes, {flag:'wx'});
    writeFileSync(join(temporary, 'pocket-ic'), executable, {flag:'wx',mode:0o755});
    chmodSync(join(temporary, 'pocket-ic'), 0o755);
    for (const file of ['archive.gz', 'pocket-ic']) { const fd = openSync(join(temporary,file),constants.O_RDONLY | constants.O_NOFOLLOW); try { fsyncSync(fd); } finally { closeSync(fd); } }
    // Refuse replacing an existing prepared bundle; check it separately.
    renameSync(temporary, directory);
    const fd = openSync(parent, constants.O_RDONLY | constants.O_DIRECTORY | constants.O_NOFOLLOW); try { fsyncSync(fd); } finally { closeSync(fd); }
  } finally { rmSync(temporary, {recursive:true,force:true}); }
  check();
} else if (mode === 'check' || mode === 'run') {
  check();
  if (mode === 'run') {
    assert(command.length, 'missing qualification command');
    const result = spawnSync('cargo', ['run','--offline','--locked','--manifest-path','qualification/Cargo.toml','-p','platform-qualification-server','--',binary,...command], {stdio:'inherit'});
    assert.ifError(result.error);
    assert.equal(result.status, 0, 'qualification supervisor failed');
  }
} else { throw new Error('expected setup, check, or run COMMAND [ARGS]'); }
