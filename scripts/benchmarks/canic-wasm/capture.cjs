// Diagnostic source capture; never changes the inspected sibling checkout.
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const cp = require('child_process');
for (const value of [__dirname, process.cwd(), ...process.argv.slice(2)]) {
  if (/[\r\n]/.test(value)) throw Error('Source capture paths must not contain line breaks');
}
if (process.argv.length !== 4) throw Error('usage: capture.cjs INPUT OUTPUT');
const input = fs.realpathSync(process.argv[2]);
if (/[\r\n]/.test(input)) throw Error('Resolved capture path contains line breaks');
const output = path.resolve(process.argv[3]);
const source = path.join(output, 'source');
const run = (args, cwd = input) => cp.execFileSync(args[0], args.slice(1), { cwd, maxBuffer: 32 * 1024 * 1024 });
const hash = b => crypto.createHash('sha256').update(b).digest('hex');
fs.mkdirSync(output, { recursive: true });
if (fs.readdirSync(output).length) throw Error('Refuse to overwrite an existing source capture');
const files = () => [...new Set(run(['git', 'ls-files', '--cached', '--others', '--exclude-standard', '-z']).toString().split('\0').filter(Boolean))].sort();
function inventory(directory, names) {
  return Object.fromEntries(names.map(p => {
    const f = path.join(directory, p), s = fs.lstatSync(f, { throwIfNoEntry: false });
    if (!s || s.isDirectory()) return [p, null];
    if (s.isSymbolicLink()) throw Error('Source capture requires regular files, not symlinks: ' + JSON.stringify(p));
    return [p, hash(fs.readFileSync(f))];
  }));
}
const names = files();
const before = inventory(input, names);
const head = run(['git', 'rev-parse', 'HEAD']);
const status = run(['git', 'status', '--porcelain=v1', '-z', '-uall']);
fs.mkdirSync(source);
for (const p of names) {
  const from = path.join(input, p), to = path.join(source, p), s = fs.lstatSync(from, { throwIfNoEntry: false });
  if (!s || s.isDirectory()) continue;
  fs.mkdirSync(path.dirname(to), { recursive: true });
  if (!s.isFile()) throw Error('Source changed file kind during capture');
  fs.copyFileSync(from, to);
}
const copied = inventory(source, names);
if (JSON.stringify(before) !== JSON.stringify(copied)) throw Error('Copied source differs from capture inventory');
const after = inventory(input, files());
if (JSON.stringify(before) !== JSON.stringify(after)) throw Error('Canic changed during capture');
if (!head.equals(run(['git', 'rev-parse', 'HEAD'])) ||
    !status.equals(run(['git', 'status', '--porcelain=v1', '-z', '-uall']))) throw Error('Canic Git identity changed during capture');
fs.writeFileSync(path.join(output, 'source-head'), head);
fs.writeFileSync(path.join(output, 'source-status.nul'), status);
fs.writeFileSync(path.join(output, 'source-inventory.json'), JSON.stringify(before, null, 2) + '\n');
fs.writeFileSync(path.join(output, 'source-tar-provenance.json'), JSON.stringify({ input, output, head: head.toString().trim(), inventorySha256: hash(JSON.stringify(before)) }, null, 2) + '\n');
console.log(source);
