// Current candidate consumers share the repository's explicitly selected tooling.
// Historical source archives remain intact; only materialized manifests change.
import assert from 'node:assert/strict';
import {readFileSync, writeFileSync} from 'node:fs';
export const toolingVersion = readFileSync(new URL('../Cargo.toml', import.meta.url), 'utf8')
  .match(/^toko-tooling = "=([0-9.]+)"$/m)?.[1];
assert(toolingVersion, 'expected an exact repository tooling selection');
export function adoptTooling(manifest) {
  const before = readFileSync(manifest, 'utf8');
  assert(/^toko-tooling = "[^"]+"$/m.test(before), `tooling catalog missing: ${manifest}`);
  writeFileSync(manifest, before.replace(/^toko-tooling = "[^"]+"$/m, `toko-tooling = "=${toolingVersion}"`));
}
