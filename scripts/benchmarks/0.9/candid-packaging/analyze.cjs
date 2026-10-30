const fs = require('fs'), crypto = require('crypto');
const root = process.argv[2];
const hash = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
function size(name) {
  const file = `${root}/${name}.wasm`, bytes = fs.readFileSync(file);
  let pos = 8, stripped = 8;
  while (pos < bytes.length) {
    const start = pos, id = bytes[pos++];
    let length = 0, shift = 0, x;
    do { x = bytes[pos++]; length += (x & 127) * 2 ** shift; shift += 7; } while (x & 128);
    pos += length;
    if (pos > bytes.length) throw Error('Invalid Wasm section');
    if (id !== 0) stripped += pos - start;
  }
  return { raw_bytes: bytes.length, stripped_bytes: stripped, sha256: hash(file) };
}
const sizes = Object.fromEntries(['baseline', 'owned', 'baseline-two', 'mixed'].map(n => [n, size(n)]));
const cases = {};
for (const corpus of ['corpus', 'typed-corpus']) {
  const baseline = fs.readFileSync(`${root}/baseline-${corpus}.txt`, 'utf8');
  if (baseline !== fs.readFileSync(`${root}/owned-${corpus}.txt`, 'utf8')) throw Error(`Differential mismatch: ${corpus}`);
  cases[corpus] = { count: baseline.trim().split('\n').length, sha256: hash(`${root}/owned-${corpus}.txt`) };
}
const metadata = JSON.parse(fs.readFileSync(`${root}/metadata.json`));
const candid = metadata.packages.filter(p => p.name === 'candid');
if (candid.length !== 1 || candid[0].source !== 'registry+https://github.com/rust-lang/crates.io-index') throw Error('Expected one unpatched registry Candid');
const graph = fs.readFileSync(`${root}/owned.graph`, 'utf8');
const retained = ['binrw', 'binrw_derive', 'array-init', 'bytemuck', 'byteorder'].filter(name => new RegExp(`^${name} `, 'm').test(graph));
if (retained.length !== 5) throw Error('Review changed registry dependency graph');
const sourcePaths = ['src/lib.rs', ...['de', 'ser', 'binary_parser', 'compat'].map(m => `src/${m}/mod.rs`)];
const source = Object.fromEntries(sourcePaths.map(p => [p, hash(`${root}/codec/${p}`)]));
const result = {
  scope: 'Unpublished codec packaging investigation using registry Candid types; no production adoption',
  registry_candid: {version: candid[0].version, source: candid[0].source},
  sizes,
  owned_saved_bytes: sizes.baseline.stripped_bytes - sizes.owned.stripped_bytes,
  mixed_added_bytes: sizes.mixed.stripped_bytes - sizes['baseline-two'].stripped_bytes,
  cases,
  retained_cargo_dependencies: retained,
  maintained_source_lines_including_comments: sourcePaths.reduce((n, p) => n + fs.readFileSync(`${root}/codec/${p}`, 'utf8').trimEnd().split('\n').length, 0),
  source_sha256: source,
  consumer_lock_sha256: hash(`${root}/consumer/Cargo.lock`),
  validation: ['Owner and codec/consumer strict Clippy', 'Registry DTO round trips and Error identity', 'Three expected concrete API identity refusals', 'Rust 1.91 Wasm check'],
  limits: ['One status-codec fixture, not application Wasm', 'Configuration/encoder/decoder and argument-trait identities are not interchangeable', 'Parser errors retain registry Error identity but use Custom instead of Binread', 'Public binrw parser API is not preserved', 'No new installed, fuzz or production qualification', 'Registry Candid dependencies remain in Cargo; savings require the original codec to be unused'],
};
fs.writeFileSync(`${root}/results.json`, JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify(result, null, 2));
