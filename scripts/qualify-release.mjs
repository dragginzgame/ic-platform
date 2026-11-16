// Retain complete local qualification bound to clean source and exact log bytes.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { createHash } from 'node:crypto';
const run = (cmd, args) => {
  const r = spawnSync(cmd, args, { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });
  if (r.error) throw r.error;
  assert.equal(r.status, 0, `${cmd}: ${r.stderr}`);
  return r.stdout;
};
run('bash', ['vendor/toko-tooling/scripts/ci/check-release-source.sh']);
const commit = run('git', ['rev-parse', 'HEAD']).trim();
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const catalog = readFileSync('ci/release-profiles.json');
const typesCatalog = readFileSync('ci/types-release-profiles.json');
const typesProfiles = JSON.parse(typesCatalog);
assert.equal(new Set(typesProfiles.map(p => p.id)).size, typesProfiles.length);
assert(typesProfiles.length > 0);
for (const profile of typesProfiles) {
  assert.equal(profile.owner, 'types');
  assert.equal(profile.command[0], 'make');
}
const profiles = {types: typesProfiles.flatMap(p => p.command.slice(1)), ...JSON.parse(catalog)};
const output = resolve(`target/release-qualification/${Date.now()}-${process.pid}`);
mkdirSync(output, {recursive:true});
const receipt = { commit, catalog: hash(catalog), types_catalog: hash(typesCatalog), rust: run('rustc', ['-V']).trim(), profiles: {} };
console.log(`Release qualification: ${output}`);
for (const [group, targets] of Object.entries(profiles)) {
  console.log(`Qualifying ${group}`);
  const result = spawnSync('make', targets, { encoding:'utf8', maxBuffer:64*1024*1024 });
  const log = `${result.stdout ?? ''}\n${result.stderr ?? ''}`;
  writeFileSync(`${output}/${group}.log`, log);
  if (result.error) throw result.error;
  assert.equal(result.status, 0, `${group} failed; inspect ${output}/${group}.log`);
  receipt.profiles[group] = hash(log);
}
writeFileSync(`${output}/audit.json`, run('cargo', ['audit', '--no-fetch', '--file', 'Cargo.lock', '--json']));
run('bash', ['vendor/toko-tooling/scripts/ci/check-release-source.sh']);
assert.equal(run('git', ['rev-parse','HEAD']).trim(),commit);
writeFileSync(`${output}/receipt.json`, JSON.stringify(receipt,null,2)+'\n');
console.log(`RELEASE_RECEIPT=${output}/receipt.json`);
