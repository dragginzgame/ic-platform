// Final, uncompressed modules only. Diagnostic symbol companions are separate.
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const cp = require('child_process');
const base = path.resolve(process.argv[2]);
const arms = process.argv.slice(3);
if (!arms.length) throw Error('usage: report.cjs BASE ARM...');
const sha256 = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
function inspect(file) {
  const bytes = fs.readFileSync(file);
  const measured = JSON.parse(cp.execFileSync('perl', [path.join(__dirname, '../wasm-sections.pl'), '--json', '-'],
    { input: bytes, encoding: 'utf8', timeout: 30000, maxBuffer: 32 * 1024 * 1024 }));
  if (measured.sections.filter(s => s.id === 10).length !== 1) throw Error('Expected one code section');
  const module = new WebAssembly.Module(bytes);
  return {
    totalBytes: measured.totalBytes, codePayloadBytes: measured.codePayloadBytes,
    codeFramedBytes: measured.codeFramedBytes, sha256: measured.sha256, sections: measured.sections,
    imports: WebAssembly.Module.imports(module), exports: WebAssembly.Module.exports(module),
  };
}
const measurements = [];
for (const role of ['issuer', 'receiver']) {
  let reference;
  for (const arm of arms) {
    const directory = path.join(base, arm);
    const item = { arm, role, ...inspect(path.join(directory, role + '.wasm')) };
    item.contractHashes = Object.fromEntries(['did', 'wire.json', 'wire.ts'].map(extension =>
      [extension, sha256(fs.readFileSync(path.join(directory, role + '.' + extension)))]));
    item.lockSha256 = sha256(fs.readFileSync(path.join(directory, 'Cargo.lock')));
    if (reference) {
      for (const key of ['imports', 'exports', 'contractHashes']) {
        if (JSON.stringify(reference[key]) !== JSON.stringify(item[key])) throw Error(`${role}/${arm}: changed ${key}`);
      }
      item.deltaTotalBytes = item.totalBytes - reference.totalBytes;
      item.deltaCodePayloadBytes = item.codePayloadBytes - reference.codePayloadBytes;
    } else reference = item;
    measurements.push(item);
    console.log(`${arm}\t${role}\t${item.totalBytes}\t${item.codePayloadBytes}`);
  }
}
fs.writeFileSync(path.join(base, 'measurements.json'), JSON.stringify(measurements, null, 2) + '\n');
