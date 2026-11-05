// Final, uncompressed modules only. Diagnostic symbol companions are separate.
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const base = path.resolve(process.argv[2]);
const arms = process.argv.slice(3);
if (!arms.length) throw Error('usage: report.cjs BASE ARM...');
const sha256 = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
function inspect(file) {
  const bytes = fs.readFileSync(file);
  if (!bytes.subarray(0, 8).equals(Buffer.from([0, 97, 115, 109, 1, 0, 0, 0]))) throw Error('Not Wasm v1');
  let offset = 8;
  const leb = () => {
    let value = 0, shift = 0, byte;
    do { byte = bytes[offset++]; value += (byte & 127) * 2 ** shift; shift += 7; } while (byte & 128);
    return value;
  };
  const sections = [];
  while (offset < bytes.length) {
    const start = offset, id = bytes[offset++], size = leb();
    const payload = bytes.subarray(offset, offset + size);
    sections.push({ id, payloadBytes: size, framedBytes: offset - start + size, sha256: sha256(payload) });
    offset += size;
  }
  if (offset !== bytes.length) throw Error('Bad section length');
  const code = sections.filter(s => s.id === 10);
  if (code.length !== 1) throw Error('Expected one code section');
  const module = new WebAssembly.Module(bytes);
  return {
    totalBytes: bytes.length, codePayloadBytes: code[0].payloadBytes,
    codeFramedBytes: code[0].framedBytes, sha256: sha256(bytes), sections,
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
