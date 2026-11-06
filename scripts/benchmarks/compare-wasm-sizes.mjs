import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const [before, after, allowance = '0'] = process.argv.slice(2);
if (!before || !after || !/^\d+$/.test(allowance) || !Number.isSafeInteger(Number(allowance))) {
  throw Error('usage: compare-wasm-sizes.mjs BEFORE AFTER [MAX_GROWTH_BYTES]');
}
for (const name of ['rustc-version.txt', 'cargo-version.txt', 'optimizer-version.txt', 'cpubits.txt', 'src/lib.rs']) {
  if (!readFileSync(join(before, name)).equals(readFileSync(join(after, name)))) {
    throw Error(`Incomparable inputs: ${name}`);
  }
}
const externalLock = directory => readFileSync(join(directory, 'Cargo.lock'), 'utf8')
  .split('\n[[package]]\n').filter(block => /^source = /m.test(block)).join('\n');
if (externalLock(before) !== externalLock(after)) throw Error('External dependency locks differ');
function rows(directory) {
  const lines = readFileSync(join(directory, 'sizes.tsv'), 'utf8').trim().split('\n');
  const expected = 'recipe\tcapability\traw_bytes\tcode_bytes\tcustom_bytes\toptimized_bytes\toptimized_code_bytes\toptimized_custom_bytes';
  if (lines.shift() !== expected || !lines.length) throw Error('Invalid size table');
  const result = new Map();
  for (const line of lines) {
    const [recipe, profile, ...sizes] = line.split('\t');
    const key = `${recipe}/${profile}`;
    if (result.has(key) || sizes.length !== 6 || sizes.some(size => !/^\d+$/.test(size) || !Number.isSafeInteger(Number(size)))) {
      throw Error('Invalid or duplicate size row');
    }
    result.set(key, sizes.map(Number));
  }
  return result;
}
const old = rows(before), current = rows(after);
if (old.size !== current.size || [...old.keys()].some(key => !current.has(key))) throw Error('Profile/recipe selections differ');
console.log('profile\traw_delta\traw_code_delta\toptimized_delta\toptimized_code_delta');
let failed = false;
for (const [key, sizes] of current) {
  const deltas = [0, 1, 3, 4].map(index => sizes[index] - old.get(key)[index]);
  console.log([key, ...deltas].join('\t'));
  if (deltas.some(delta => delta > Number(allowance))) failed = true;
}
if (failed) process.exitCode = 1;
