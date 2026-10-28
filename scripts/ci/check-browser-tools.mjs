import assert from 'node:assert/strict';
import { readFile, lstat } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { checkBrowserSdkGraph } from './browser-sdk-graph.mjs';

const json = async path => JSON.parse(await readFile(new URL(path, import.meta.url), 'utf8'));
const tools = await json('../fixtures/wire-schema/package.json');
const manifest = await json('../../packages/browser/package.json');
const floor = await json('../fixtures/browser-compiler-floor/package.json');
assert.deepEqual(floor.engines, tools.engines, 'Consumer compiler preparation uses the shared Node/npm selection');
assert.equal((await json('../fixtures/browser-compiler-floor/node_modules/typescript/package.json')).version,
  floor.devDependencies.typescript, 'Prepare the consumer compiler floor with make install-browser-tools');
assert.equal(process.versions.node, tools.engines.node, 'Use the pinned contributor Node version');
assert.equal(execFileSync('npm', ['--version'], { encoding: 'utf8' }).trim(), tools.engines.npm);
assert.deepEqual(manifest.dependencies, { '@icp-sdk/core': '6.1.1' });
assert.equal(manifest.peerDependencies, undefined);
for (const [name, version] of Object.entries({ ...manifest.dependencies, ...manifest.devDependencies })) {
  assert.equal((await json(`../../packages/browser/node_modules/${name}/package.json`)).version, version, `Prepare ${name} with make install-browser-tools`);
}
await checkBrowserSdkGraph(fileURLToPath(new URL('../../packages/browser', import.meta.url)),
  manifest.dependencies['@icp-sdk/core'], ['@caffeineai/object-storage', '@icp-sdk/auth', '@icp-sdk/signer']);
const license = new URL('../../packages/browser/LICENSE', import.meta.url);
assert((await lstat(license)).isFile(), 'Browser LICENSE must be a regular file');
assert.equal(await readFile(license, 'utf8'), await readFile(new URL('../../LICENSE', import.meta.url), 'utf8'),
  'Browser LICENSE must match the workspace MIT license');
console.log('Browser SDK and test dependencies match their exact selections');
