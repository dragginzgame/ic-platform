const fs = require('node:fs');
const [output, fixtures, typed] = process.argv.slice(2);
const corpus = [];
function add(bytes, kind = 'reserved') {
  const hex = bytes.toString('hex');
  corpus.push(`header|65536|${hex}`);
  for (const n of [0, 4, 6, 8, 16, 64, 65536]) corpus.push(`${kind}|${n}|${hex}`);
}
const seeds = [
  '4449444c0000', '4449444c00017d00', '4449444c00017e7f',
  '4449444c000168010100', '4449444c016d7b0100030080ff', '4449444c016e7f010000',
  '4449444c016a0000010101000101000178', '4449444c026a00000101690101660001010101000178',
  '4449444c016c02007b017e01008001', '4449444c017b000100',
].map(h => Buffer.from(h, 'hex'));
for (const f of fs.readdirSync(fixtures).filter(f => /^management-.*\.hex$/.test(f))) {
  seeds.push(Buffer.from(fs.readFileSync(`${fixtures}/${f}`, 'utf8').replace(/\s/g, ''), 'hex'));
}
for (const s of seeds) {
  add(s);
  for (let n = 0; n < s.length; n++) add(s.subarray(0, n));
  for (let n = 0; n < Math.min(s.length, 120); n++) {
    for (const v of [0, 1, 0x69, 0x6b, 0x6c, 0x6d, 0x6e, 0x7f, 0x80, 0xff]) {
      const b = Buffer.from(s); b[n] = v; add(b);
    }
  }
}
for (const kind of ['nat', 'principal', 'record', 'status']) for (const s of seeds) add(s, kind);
for (const table of ['6c8080808010', '6c01008080808010', '698080808080808080808001',
  '6affffffffffffffffff01', '678080808080808080808001', '6e80', 'ee7f7f', '6a000002', '6a00000104']) {
  add(Buffer.from(`4449444c01${table}0100`, 'hex'));
}
let state = 0x51cafe;
function rand() { state = (Math.imul(state, 1664525) + 1013904223) >>> 0; return state; }
for (let i = 0; i < 5000; i++) {
  const s = Buffer.from(seeds[rand() % seeds.length]); const n = rand() % s.length;
  s[n] = rand() & 255; add(s);
}
fs.writeFileSync(`${output}/corpus.txt`, `${corpus.join('\n')}\n`);
if (typed) {
  const lines = [];
  for (const s of fs.readFileSync(typed, 'utf8').trim().split('\n')) {
    const [kind, , hex] = s.split('|'), b = Buffer.from(hex, 'hex');
    const addTyped = x => { for (const n of [4, 8, 16, 64, 65536]) lines.push(`${kind}|${n}|${x.toString('hex')}`); };
    addTyped(b);
    for (let n = 0; n < b.length; n++) {
      addTyped(b.subarray(0, n));
      for (const v of [0, 0x7f, 0xff]) { const m = Buffer.from(b); m[n] = v; addTyped(m); }
    }
  }
  fs.writeFileSync(`${output}/typed-corpus.txt`, `${lines.join('\n')}\n`);
}
