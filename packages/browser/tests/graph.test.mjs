import assert from 'node:assert/strict';
import test from 'node:test';
import { mkdir, mkdtemp, readFile, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { checkBrowserSdkGraph } from '../../../scripts/ci/browser-sdk-graph.mjs';

// Resolve real Node packages without importing or reproducing SDK implementations.
async function graph(t) {
  const root = await mkdtemp(join(tmpdir(), 'platform-sdk-graph.'));
  t.diagnostic(`Graph refusal evidence: ${root}`);
  await mkdir(join(root, 'node_modules/@icp-sdk/core'), { recursive: true });
  await mkdir(join(root, 'node_modules/provider'), { recursive: true });
  await writeFile(join(root, 'package.json'), '{}');
  await writeFile(join(root, 'node_modules/provider/package.json'), '{"name":"provider"}');
  await writeFile(join(root, 'node_modules/@icp-sdk/core/package.json'), JSON.stringify({
    name: '@icp-sdk/core', version: '6.1.1', exports: { './agent': './agent.js' },
  }));
  await writeFile(join(root, 'node_modules/@icp-sdk/core/agent.js'), '');
  const lock = { packages: { 'node_modules/@icp-sdk/core': {
    version: '6.1.1', resolved: 'https://registry.npmjs.org/@icp-sdk/core/-/core-6.1.1.tgz',
  } } };
  await writeFile(join(root, 'package-lock.json'), JSON.stringify(lock));
  return { root, lock };
}

test('canonical lock and shared installed Agent are admitted', async t => {
  const { root } = await graph(t);
  await checkBrowserSdkGraph(root, '6.1.1', ['provider']);
});

test('duplicate or aliased locked SDK copies are refused', async t => {
  for (const path of ['node_modules/provider/node_modules/@icp-sdk/core', 'node_modules/sdk-alias']) {
    const { root, lock } = await graph(t);
    lock.packages[path] = { name: '@icp-sdk/core', version: '6.1.1' };
    await writeFile(join(root, 'package-lock.json'), JSON.stringify(lock));
    await assert.rejects(checkBrowserSdkGraph(root, '6.1.1', ['provider']), /one locked SDK/);
  }
});

test('a sole encoded-URL SDK alias is refused', async t => {
  const { root, lock } = await graph(t);
  lock.packages['node_modules/sdk-alias'] = {
    version: '6.1.1', resolved: 'https://registry.npmjs.org/%40icp-sdk%2Fcore/-/core-6.1.1.tgz',
  };
  delete lock.packages['node_modules/@icp-sdk/core'];
  await writeFile(join(root, 'package-lock.json'), JSON.stringify(lock));
  await assert.rejects(checkBrowserSdkGraph(root, '6.1.1', ['provider']), /must not be aliased/);
});

test('an unrecorded installed shadow is refused even at the same SDK version', async t => {
  const { root } = await graph(t);
  const shadow = join(root, 'node_modules/provider/node_modules/@icp-sdk/core');
  await mkdir(shadow, { recursive: true });
  await writeFile(join(shadow, 'package.json'),
    await readFile(join(root, 'node_modules/@icp-sdk/core/package.json')));
  await writeFile(join(shadow, 'agent.js'), '');
  await assert.rejects(checkBrowserSdkGraph(root, '6.1.1', ['provider']), /different SDK Core Agent/);
});

test('installed SDK version drift is refused', async t => {
  const { root } = await graph(t);
  await assert.rejects(checkBrowserSdkGraph(root, '6.1.2', ['provider']), /Locked SDK Core selection differs/);
  const manifest = join(root, 'node_modules/@icp-sdk/core/package.json');
  const data = JSON.parse(await readFile(manifest, 'utf8'));
  data.version = '5.4.0';
  await writeFile(manifest, JSON.stringify(data));
  await assert.rejects(checkBrowserSdkGraph(root, '6.1.1', ['provider']), /Installed SDK Core selection differs/);
});
