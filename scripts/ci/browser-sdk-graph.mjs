import assert from 'node:assert/strict';
import { readFile, realpath } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { resolve } from 'node:path';

/** Admit one locked SDK and one installed Agent identity for the actual owners. */
export async function checkBrowserSdkGraph(root, version, owners) {
  const lock = JSON.parse(await readFile(resolve(root, 'package-lock.json'), 'utf8'));
  const sdkPackages = Object.entries(lock.packages).filter(([path, entry]) =>
    path.endsWith('node_modules/@icp-sdk/core') || entry.name === '@icp-sdk/core' ||
    decodeURIComponent(entry.resolved ?? '').includes('/@icp-sdk/core/'));
  assert.equal(sdkPackages.length, 1, 'Expected one locked SDK Core package');
  assert.equal(sdkPackages[0][0], 'node_modules/@icp-sdk/core', 'SDK Core must not be aliased');
  assert.equal(sdkPackages[0][1].version, version, 'Locked SDK Core selection differs');
  const installed = JSON.parse(await readFile(resolve(root, 'node_modules/@icp-sdk/core/package.json'), 'utf8'));
  assert.equal(installed.name, '@icp-sdk/core');
  assert.equal(installed.version, version, 'Installed SDK Core selection differs');
  const consumer = createRequire(resolve(root, 'package.json'));
  const canonical = await realpath(consumer.resolve('@icp-sdk/core/agent'));
  for (const owner of owners) {
    const manifest = await realpath(resolve(root, 'node_modules', owner, 'package.json'));
    const supplied = createRequire(manifest);
    assert.equal(await realpath(supplied.resolve('@icp-sdk/core/agent')), canonical,
      `${owner} resolves a different SDK Core Agent`);
  }
}
