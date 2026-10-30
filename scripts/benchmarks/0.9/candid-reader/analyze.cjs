const fs = require('node:fs'), crypto = require('node:crypto');
const root = process.argv[2];
const sha = p => crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex');
function size(name) {
  const file = `${root}/${name}.wasm`, b = fs.readFileSync(file);
  let pos = 8, stripped = 8;
  while (pos < b.length) {
    const start = pos, id = b[pos++]; let len = 0, shift = 0, x;
    do { x = b[pos++]; len += (x & 127) * 2 ** shift; shift += 7; } while (x & 128);
    pos += len; if (pos > b.length) throw Error('Invalid Wasm section');
    if (id !== 0) stripped += pos - start;
  }
  return { raw_bytes: b.length, stripped_bytes: stripped, sha256: sha(file) };
}
const sizes = Object.fromEntries(['baseline', 'reader', 'compact'].map(n => [n, size(n)]));
for (const n of ['reader', 'compact']) sizes[n].saved_bytes = sizes.baseline.stripped_bytes - sizes[n].stripped_bytes;
const results = {};
for (const corpus of ['results', 'typed-results']) {
  const baseline = fs.readFileSync(`${root}/baseline-${corpus}.txt`, 'utf8');
  for (const name of ['reader', 'compact']) {
    const file = `${root}/${name}-${corpus}.txt`;
    if (!fs.existsSync(file)) continue;
    const value = fs.readFileSync(file, 'utf8');
    if (baseline !== value) throw Error(`Differential mismatch: ${file}`);
    results[`${name}-${corpus}`] = { cases: baseline.trim().split('\n').length, sha256: sha(file) };
  }
}
const report = { scope: 'Isolated Candid 0.10.39 reader experiment; no production codec adoption', sizes, results,
  limits: ['Status-codec fixture; not a production application size measurement', 'Diagnostic text and public binrw traits are not preserved', 'Corpus comparison is not exhaustive protocol or fuzz qualification', 'Cargo patches are experiment supply only, not downstream distribution'] };
fs.writeFileSync(`${root}/results.json`, `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify(report, null, 2));
